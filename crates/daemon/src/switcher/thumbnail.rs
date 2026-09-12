//! DWM thumbnail abstraction and RAII handle lifecycle.

use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::{DwmRegisterThumbnail, DwmUnregisterThumbnail};

/// Trait for thumbnail registration and unregistration, enabling fake counting tests.
pub trait ThumbnailSink: Send + Sync {
    fn register(&mut self, destination: HWND, source: HWND) -> Result<isize, i32>;
    fn unregister(&mut self, handle: isize);
}

/// Win32 DWM thumbnail sink.
#[derive(Default, Clone)]
pub struct DwmThumbnailSink;

impl ThumbnailSink for DwmThumbnailSink {
    fn register(&mut self, destination: HWND, source: HWND) -> Result<isize, i32> {
        let mut handle: isize = 0;
        // SAFETY: `DwmRegisterThumbnail` is called with valid destination and source window handles.
        // `&mut handle` is a live local stack pointer initialized to 0.
        let hr = unsafe { DwmRegisterThumbnail(destination, source, &mut handle) };
        if hr == 0 {
            Ok(handle)
        } else {
            Err(hr)
        }
    }

    fn unregister(&mut self, handle: isize) {
        if handle != 0 {
            // SAFETY: `DwmUnregisterThumbnail` is called with a non-zero thumbnail handle returned by a prior register.
            unsafe {
                let _ = DwmUnregisterThumbnail(handle);
            }
        }
    }
}

/// RAII wrapper around a registered thumbnail handle.
pub struct DwmThumbnailHandle {
    pub handle: isize,
    pub sink: Arc<Mutex<dyn ThumbnailSink>>,
}

impl DwmThumbnailHandle {
    pub fn new(handle: isize, sink: Arc<Mutex<dyn ThumbnailSink>>) -> Self {
        Self { handle, sink }
    }
}

impl Drop for DwmThumbnailHandle {
    fn drop(&mut self) {
        if self.handle != 0 {
            if let Ok(mut s) = self.sink.lock() {
                s.unregister(self.handle);
            }
            self.handle = 0;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Default, Clone)]
    pub struct FakeThumbnailSink {
        pub counts: Arc<Mutex<(usize, usize, usize)>>, // (active, registered, unregistered)
    }

    impl ThumbnailSink for FakeThumbnailSink {
        fn register(&mut self, _destination: HWND, _source: HWND) -> Result<isize, i32> {
            let mut c = self.counts.lock().unwrap();
            c.0 += 1;
            c.1 += 1;
            Ok(c.1 as isize)
        }

        fn unregister(&mut self, _handle: isize) {
            let mut c = self.counts.lock().unwrap();
            c.0 = c.0.saturating_sub(1);
            c.2 += 1;
        }
    }

    #[test]
    fn every_registration_is_unregistered_on_dismissal() {
        let counts = Arc::new(Mutex::new((0usize, 0usize, 0usize)));
        let fake = FakeThumbnailSink {
            counts: Arc::clone(&counts),
        };
        let sink: Arc<Mutex<dyn ThumbnailSink>> = Arc::new(Mutex::new(fake));
        {
            let mut handles = Vec::new();
            for i in 1..=5 {
                let handle_val = sink.lock().unwrap().register(100, i).unwrap();
                handles.push(DwmThumbnailHandle::new(handle_val, Arc::clone(&sink)));
            }
            assert_eq!(counts.lock().unwrap().0, 5);
        }
        // At this point, `handles` have all been dropped on dismissal
        let c = counts.lock().unwrap();
        assert_eq!(c.0, 0);
        assert_eq!(c.1, 5);
        assert_eq!(c.2, 5);
    }
}
