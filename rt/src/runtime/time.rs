use crate::mem::Float;
use crate::state::State;
use rustix::time;
use std::mem::MaybeUninit;

fn utc() -> f64 {
    let ts = time::clock_gettime(time::ClockId::Realtime);
    ts.tv_sec as f64 + (ts.tv_nsec as f64 / 1_000_000_000.0)
}

fn offset() -> i64 {
    unsafe {
        extern "C" {
            fn tzset();
        }

        let ts = time::clock_gettime(time::ClockId::Realtime);
        let mut tm = MaybeUninit::uninit();

        // localtime_r() doesn't necessarily call tzset() for us.
        tzset();

        // While localtime_r() may call setenv() internally, this is not a
        // problem as Inko caches environment variables upon startup. If an FFI
        // call ends up racing with the setenv() call, that's a problem for the
        // FFI code.
        if libc::localtime_r(&ts.tv_sec, tm.as_mut_ptr()).is_null() {
            panic!("localtime_r() failed");
        }

        tm.assume_init().tm_gmtoff
    }
}

#[no_mangle]
pub unsafe extern "system" fn inko_time_monotonic(state: *const State) -> i64 {
    // An i64 gives us roughly 292 years of time. That should be more than
    // enough for a monotonic clock, as an Inko program is unlikely to run for
    // that long.
    let state = &*state;

    state.start_time.elapsed().as_nanos() as i64
}

#[no_mangle]
pub unsafe extern "system" fn inko_time_system(
    state: *const State,
) -> *const Float {
    Float::alloc((*state).float_class, utc())
}

#[no_mangle]
pub unsafe extern "system" fn inko_time_system_offset() -> i64 {
    offset()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_utc() {
        let expected =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let given = utc();

        // We can't assert equality, for there may be time between the two
        // function calls. We also can't assert the utc() time is greater in the
        // event of clock changes. Instead we just assert the two times are
        // within 5 seconds of each other, which should be sufficient.
        assert!((given - expected).abs() < 5.0);
    }
}
