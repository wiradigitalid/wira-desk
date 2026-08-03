//! Static single-producer single-consumer ring (Hook → Worker).
//! Sixteen FIFO slots, `u8` primitives only, no heap.

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

use shared::constants::RING_BUFFER_CAPACITY;

const CAP: u32 = RING_BUFFER_CAPACITY as u32;

static SLOTS: [AtomicU8; RING_BUFFER_CAPACITY] = [
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
    AtomicU8::new(0),
];

static HEAD: AtomicU32 = AtomicU32::new(0);
static TAIL: AtomicU32 = AtomicU32::new(0);

#[inline]
fn slot_index(seq: u32) -> usize {
    (seq % CAP) as usize
}

/// Producer (Hook Thread): push one command byte. Returns `false` when full.
pub fn push(value: u8) -> bool {
    let head = HEAD.load(Ordering::Acquire);
    let tail = TAIL.load(Ordering::Acquire);
    if head.wrapping_sub(tail) >= CAP {
        return false;
    }
    let idx = slot_index(head);
    SLOTS[idx].store(value, Ordering::Release);
    HEAD.store(head.wrapping_add(1), Ordering::Release);
    true
}

/// Consumer (Worker): pop one command byte, FIFO order.
pub fn pop() -> Option<u8> {
    let tail = TAIL.load(Ordering::Acquire);
    let head = HEAD.load(Ordering::Acquire);
    if tail == head {
        return None;
    }
    let idx = slot_index(tail);
    let value = SLOTS[idx].load(Ordering::Acquire);
    TAIL.store(tail.wrapping_add(1), Ordering::Release);
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    /// Serializes every test in this module. The ring uses process-wide static
    /// state, so parallel `cargo test` runs would otherwise race on HEAD/TAIL.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset() {
        while pop().is_some() {}
        HEAD.store(0, Ordering::Release);
        TAIL.store(0, Ordering::Release);
    }

    #[test]
    fn empty_pop_is_none() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        assert!(pop().is_none());
    }

    #[test]
    fn fifo_single_push_pop() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        assert!(push(1));
        assert_eq!(pop(), Some(1));
        assert!(pop().is_none());
    }

    #[test]
    fn fifo_order_and_wrap() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        for v in 0..16u8 {
            assert!(push(v));
        }
        for v in 0..16u8 {
            assert_eq!(pop(), Some(v));
        }
        assert!(push(42));
        assert_eq!(pop(), Some(42));
    }

    #[test]
    fn sixteenth_push_ok_seventeenth_rejected() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        for _ in 0..16 {
            assert!(push(1));
        }
        assert!(!push(2));
        assert_eq!(pop(), Some(1));
        assert!(push(2));
    }

    #[test]
    fn full_does_not_overwrite_unread() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        for v in 10..26u8 {
            assert!(push(v));
        }
        assert!(!push(99));
        assert_eq!(pop(), Some(10));
        assert_eq!(pop(), Some(11));
    }

    #[test]
    fn spsc_stress_producer_consumer() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset();
        let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let done_c = Arc::clone(&done);
        let producer = thread::spawn(move || {
            let mut seq: u8 = 0;
            while !done_c.load(Ordering::Acquire) {
                if push(seq) {
                    seq = seq.wrapping_add(1);
                } else {
                    thread::yield_now();
                }
            }
        });
        let mut last: Option<u8> = None;
        let mut count = 0usize;
        while count < 10_000 {
            if let Some(v) = pop() {
                if let Some(prev) = last {
                    assert_eq!(v, prev.wrapping_add(1));
                }
                last = Some(v);
                count += 1;
            } else {
                thread::yield_now();
            }
        }
        done.store(true, Ordering::Release);
        producer.join().unwrap();
    }
}
