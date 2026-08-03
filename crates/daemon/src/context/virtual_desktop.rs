//! Virtual-desktop membership via `IVirtualDesktopManager`.
//! **This is the only COM integration in Wira Desk.** `windows-sys` 0.52 ships no
//! binding for `IVirtualDesktopManager`, so the vtable is declared by hand
//! against the documented interface layout. Everything is kept minimal and
//! explicit, which is the documented exception permits.
//! Threading: the apartment is initialized and the interface created, used, and
//! released on **one** thread — the Worker actor. [`VirtualDesktopManager`]
//! holds a raw pointer, which makes it `!Send` and `!Sync` automatically, so
//! the compiler enforces that ownership rather than a comment.
//! Failure policy: every path that cannot *prove* membership returns `None`,
//! and the frozen contract turns `None` into an ineligible decision.
//! The adapter therefore fails closed without deciding that itself.
//! # Unverified
//! The COM path in this file has never been executed. It is written against the
//! documented interface layout and reviewed for lifecycle correctness, but only
//! a live elevated desktop can confirm it.

use std::ffi::c_void;
use std::ptr;

use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, S_OK};
use windows_sys::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};

use crate::cycling::WindowId;

use super::VirtualDesktopSource;

/// `CLSID_VirtualDesktopManager` — `{aa509086-5ca9-4c25-8f95-589d3c07b48a}`.
const CLSID_VIRTUAL_DESKTOP_MANAGER: GUID = GUID {
    data1: 0xaa50_9086,
    data2: 0x5ca9,
    data3: 0x4c25,
    data4: [0x8f, 0x95, 0x58, 0x9d, 0x3c, 0x07, 0xb4, 0x8a],
};

/// `IID_IVirtualDesktopManager` — `{a5cd92ff-29be-454c-8d04-d82879fb3f1b}`.
const IID_IVIRTUAL_DESKTOP_MANAGER: GUID = GUID {
    data1: 0xa5cd_92ff,
    data2: 0x29be,
    data3: 0x454c,
    data4: [0x8d, 0x04, 0xd8, 0x28, 0x79, 0xfb, 0x3f, 0x1b],
};

/// `RPC_E_CHANGED_MODE` — the apartment already exists in another mode. We may
/// still use it, but we must **not** balance it with `CoUninitialize`.
const RPC_E_CHANGED_MODE: HRESULT = -2_147_417_850; // 0x80010106

/// Hand-declared vtable. Order matters and must match the documented interface:
/// three `IUnknown` slots followed by the three `IVirtualDesktopManager` slots.
#[repr(C)]
struct IVirtualDesktopManagerVtbl {
    query_interface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    is_window_on_current_virtual_desktop:
        unsafe extern "system" fn(*mut c_void, HWND, *mut BOOL) -> HRESULT,
    get_window_desktop_id: unsafe extern "system" fn(*mut c_void, HWND, *mut GUID) -> HRESULT,
    move_window_to_desktop: unsafe extern "system" fn(*mut c_void, HWND, *const GUID) -> HRESULT,
}

#[repr(C)]
struct IVirtualDesktopManagerRaw {
    vtbl: *const IVirtualDesktopManagerVtbl,
}

/// Worker-thread-owned virtual-desktop adapter.
/// Construct with [`VirtualDesktopManager::create`], drop to release. The raw
/// pointer makes this type `!Send`/`!Sync`, which is the enforcement mechanism
/// for "COM ownership remains on the Worker actor's thread".
pub struct VirtualDesktopManager {
    manager: *mut IVirtualDesktopManagerRaw,
    /// Whether *we* initialized the apartment and therefore owe a
    /// `CoUninitialize`. False when we joined an existing apartment.
    owns_apartment: bool,
}

impl VirtualDesktopManager {
    /// Initialize the apartment and create the interface.
    /// Returns `None` if either step fails; the caller then treats every
    /// membership question as unknown, which fails closed.
    pub fn create() -> Option<VirtualDesktopManager> {
        // windows-sys 0.52 types the constant as `COINIT` (i32) but the
        // function parameter as `u32`, so the cast is required.
        // SAFETY: the first parameter is documented as reserved and must be null, which it is.
        // The obligation this call creates is a reference count, not a pointer: every return
        // that indicates the apartment was entered — `S_OK` or `S_FALSE`, both `>= 0` — owes
        // exactly one `CoUninitialize` **on this same thread**. `RPC_E_CHANGED_MODE` is the
        // one success-shaped failure: the apartment already exists in another mode, so it is
        // usable but was not entered by us and must not be balanced. `owns_apartment` records
        // which of those two happened, and it is the only thing standing between this and an
        // apartment refcount driven negative by a `CoUninitialize` we never earned.
        //
        // Thread affinity is not checked here because it is enforced by the type: the raw
        // pointer field makes `VirtualDesktopManager` `!Send`/`!Sync`, so the value cannot
        // leave the thread that ran this call.
        let hr = unsafe { CoInitializeEx(ptr::null(), COINIT_APARTMENTTHREADED as u32) };
        // S_OK and S_FALSE both mean usable. S_FALSE means this thread was
        // already initialized in the same mode — still balanced by one
        // CoUninitialize, per the documented reference counting.
        let owns_apartment = if hr >= 0 {
            true
        } else if hr == RPC_E_CHANGED_MODE {
            // Someone else owns the apartment in another mode. Usable, but not
            // ours to tear down.
            false
        } else {
            return None;
        };

        let mut raw: *mut c_void = ptr::null_mut();
        // SAFETY: both GUIDs are `'static` consts, so the pointers are valid for the call, and
        // `&mut raw` is a live out-param of exactly the pointer width COM writes.
        //
        // The load-bearing precondition is the agreement between the IID and the type this
        // pointer is cast to below. COM hands back whatever interface `IID_IVIRTUAL_DESKTOP_-
        // MANAGER` names, and we then reinterpret it as `IVirtualDesktopManagerRaw` and index
        // its vtable by slot number — so a mistranscribed IID or a mis-ordered vtable would
        // not fail loudly, it would call a different function through a different signature.
        // Neither is left to review alone: `guid_constants_match_the_documented_values` pins
        // every nibble of both GUIDs, and `vtable_slots_sit_at_the_documented_offsets` pins
        // each of the six slots individually plus the total size, because swapping two
        // pointer-sized fields leaves `size_of` unchanged.
        //
        // The apartment is already initialised at this point, which `CoCreateInstance`
        // requires, and the failure path below releases it iff we entered it.
        let hr = unsafe {
            CoCreateInstance(
                &CLSID_VIRTUAL_DESKTOP_MANAGER,
                ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IVIRTUAL_DESKTOP_MANAGER,
                &mut raw,
            )
        };

        if hr != S_OK || raw.is_null() {
            if owns_apartment {
                // SAFETY: balances the one `CoInitializeEx` above that this thread earned,
                // and runs on that same thread. Guarded by `owns_apartment`, so the
                // `RPC_E_CHANGED_MODE` case — an apartment someone else owns — is left
                // alone. No interface was obtained, so there is nothing to release first.
                unsafe { CoUninitialize() };
            }
            return None;
        }

        Some(VirtualDesktopManager {
            manager: raw as *mut IVirtualDesktopManagerRaw,
            owns_apartment,
        })
    }

    fn query_membership(&self, window: HWND) -> Option<bool> {
        if self.manager.is_null() || window == 0 {
            return None;
        }
        let mut on_current: BOOL = FALSE;
        // SAFETY: `self.manager` is non-null (checked above) and, since `create` returned it,
        // points to a live COM object this instance still holds a reference to — `Drop` is the
        // only place that releases it, and it nulls the field.
        //
        // Reading `(*self.manager).vtbl` is sound because the COM ABI guarantees that any
        // interface pointer begins with a pointer to its vtable, which is exactly the layout
        // `IVirtualDesktopManagerRaw` declares as `#[repr(C)]`. Indexing the fourth slot is
        // sound because our vtable declaration matches the documented one — three `IUnknown`
        // slots then three interface slots — which the offset test pins field by field. The
        // signature is likewise ours to get right: `extern "system"` is COM's calling
        // convention, and the parameter list matches the documented method.
        //
        // `self.manager as *mut c_void` is the `this` pointer the method expects, and
        // `&mut on_current` is a live out-param of the right width. `window` needs no
        // validity proof: an unusable handle produces a failing `HRESULT`, which is checked
        // below and mapped to `None` rather than to `false`. Called on the owning thread by
        // construction — `&self` on a `!Send`/`!Sync` type.
        let hr = unsafe {
            let vtbl = (*self.manager).vtbl;
            ((*vtbl).is_window_on_current_virtual_desktop)(
                self.manager as *mut c_void,
                window,
                &mut on_current,
            )
        };
        if hr != S_OK {
            // The window vanished, is not a top-level window, or the shell is
            // unavailable. Unknown, not "false".
            return None;
        }
        Some(on_current != FALSE)
    }
}

impl Drop for VirtualDesktopManager {
    fn drop(&mut self) {
        if !self.manager.is_null() {
            // SAFETY: same layout and slot reasoning as `query_membership`, on slot 2
            // (`IUnknown::Release`). This releases the single reference `CoCreateInstance`
            // handed over, exactly once: `Drop` runs at most once per value, and the field is
            // nulled immediately afterwards so any further path sees no reference to release.
            unsafe {
                let vtbl = (*self.manager).vtbl;
                ((*vtbl).release)(self.manager as *mut c_void);
            }
            self.manager = ptr::null_mut();
        }
        if self.owns_apartment {
            // SAFETY: balances the `CoInitializeEx` this thread earned, on that thread, and
            // only when we entered the apartment ourselves. The ordering is the precondition
            // that matters here — `Release` above happens *before* this, because releasing an
            // interface after its apartment has been torn down is use of a dead apartment.
            unsafe { CoUninitialize() };
        }
    }
}

impl VirtualDesktopSource for VirtualDesktopManager {
    fn is_on_current_desktop(&self, window: WindowId) -> Option<bool> {
        self.query_membership(window.0)
    }
}

/// Adapter used when COM is unavailable.
/// Every answer is "unknown", so the frozen contract rejects every candidate.
/// That is the intended behaviour: without virtual-desktop information Wira Desk
/// must not risk activating a window on another desktop.
pub struct UnavailableVirtualDesktops;

impl VirtualDesktopSource for UnavailableVirtualDesktops {
    fn is_on_current_desktop(&self, _window: WindowId) -> Option<bool> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::super::{
        evaluate_spatial, MonitorId, SpatialContext, SpatialDecision, SpatialFacts,
        SpatialRejection,
    };
    use super::*;

    #[test]
    fn unavailable_source_reports_unknown() {
        assert_eq!(
            UnavailableVirtualDesktops.is_on_current_desktop(WindowId(1)),
            None
        );
    }

    #[test]
    fn unknown_membership_fails_closed_through_the_contract() {
        let ctx = SpatialContext {
            origin_monitor: Some(MonitorId(1)),
        };
        let facts = SpatialFacts {
            candidate_monitor: Some(MonitorId(1)),
            on_current_virtual_desktop: UnavailableVirtualDesktops
                .is_on_current_desktop(WindowId(1)),
        };
        assert_eq!(
            evaluate_spatial(&ctx, &facts),
            SpatialDecision::Ineligible(SpatialRejection::VirtualDesktopUnavailable)
        );
    }

    #[test]
    fn guid_constants_match_the_documented_values() {
        // Transcription guard: a wrong nibble here would fail at runtime with
        // an opaque REGDB_E_CLASSNOTREG, so it is worth pinning literally.
        assert_eq!(CLSID_VIRTUAL_DESKTOP_MANAGER.data1, 0xaa50_9086);
        assert_eq!(CLSID_VIRTUAL_DESKTOP_MANAGER.data2, 0x5ca9);
        assert_eq!(CLSID_VIRTUAL_DESKTOP_MANAGER.data3, 0x4c25);
        assert_eq!(
            CLSID_VIRTUAL_DESKTOP_MANAGER.data4,
            [0x8f, 0x95, 0x58, 0x9d, 0x3c, 0x07, 0xb4, 0x8a]
        );
        assert_eq!(IID_IVIRTUAL_DESKTOP_MANAGER.data1, 0xa5cd_92ff);
        assert_eq!(IID_IVIRTUAL_DESKTOP_MANAGER.data2, 0x29be);
        assert_eq!(IID_IVIRTUAL_DESKTOP_MANAGER.data3, 0x454c);
        assert_eq!(
            IID_IVIRTUAL_DESKTOP_MANAGER.data4,
            [0x8d, 0x04, 0xd8, 0x28, 0x79, 0xfb, 0x3f, 0x1b]
        );
    }

    #[test]
    fn vtable_slots_sit_at_the_documented_offsets() {
        // A COM call dispatches purely on slot offset, so a reordered field
        // would silently call the wrong function with the wrong signature.
        // `size_of` alone cannot catch that — swapping two pointer-sized fields
        // leaves the total unchanged — so each slot is pinned individually.
        // IUnknown's three slots must come first, then the interface's three.
        let slot = size_of::<usize>();
        assert_eq!(offset_of!(IVirtualDesktopManagerVtbl, query_interface), 0);
        assert_eq!(offset_of!(IVirtualDesktopManagerVtbl, add_ref), slot);
        assert_eq!(offset_of!(IVirtualDesktopManagerVtbl, release), 2 * slot);
        assert_eq!(
            offset_of!(
                IVirtualDesktopManagerVtbl,
                is_window_on_current_virtual_desktop
            ),
            3 * slot
        );
        assert_eq!(
            offset_of!(IVirtualDesktopManagerVtbl, get_window_desktop_id),
            4 * slot
        );
        assert_eq!(
            offset_of!(IVirtualDesktopManagerVtbl, move_window_to_desktop),
            5 * slot
        );
        // No trailing padding, so the layout is exactly the six documented slots.
        assert_eq!(size_of::<IVirtualDesktopManagerVtbl>(), 6 * slot);
    }

    // Compile-time proof that a type implements neither `Send` nor `Sync`.
    //
    // Both traits carry two overlapping blanket impls: one for every type, one
    // gated on the auto trait. For a type that does *not* implement the auto
    // trait only the first applies, so inference resolves `A` and the call
    // compiles. For a type that does, both apply, `A` becomes ambiguous, and
    // compilation fails. That inverted-ambiguity trick is what makes this a real
    // assertion — an ordinary generic function with no bounds accepts every type
    // and therefore proves nothing.
    trait NotSend<A> {
        fn assert() {}
    }
    impl<T: ?Sized> NotSend<()> for T {}
    impl<T: ?Sized + Send> NotSend<u8> for T {}

    trait NotSync<A> {
        fn assert() {}
    }
    impl<T: ?Sized> NotSync<()> for T {}
    impl<T: ?Sized + Sync> NotSync<u8> for T {}

    #[test]
    fn manager_is_neither_send_nor_sync() {
        // The COM apartment is per-thread: the interface is created, used, and
        // released on the Worker actor's thread, and the raw pointer field is
        // what enforces that. Adding `unsafe impl Send`/`Sync` to
        // `VirtualDesktopManager` would break `CoUninitialize` thread affinity
        // and make this stop compiling.
        <VirtualDesktopManager as NotSend<_>>::assert();
        <VirtualDesktopManager as NotSync<_>>::assert();
    }
}
