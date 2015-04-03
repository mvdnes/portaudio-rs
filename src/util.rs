use pa::{PaResult, PaError};
use ll;
use std::num::FromPrimitive;

pub use self::duration::Duration;

pub fn to_pa_result(code: i32) -> PaResult
{
    if code == ll::paNoError
    {
        return Ok(());
    }
    let error: Option<PaError> = FromPrimitive::from_i32(code);
    match error
    {
        Some(value) => Err(value),
        None => Err(PaError::UnknownError),
    }
}

pub fn pa_time_to_duration(seconds: f64) -> Duration
{
    Duration::milliseconds((seconds * 1000.0) as i64)
}

pub fn duration_to_pa_time(duration: Duration) -> f64
{
    duration.num_milliseconds() as f64 / 1000.0
}

mod duration {
    const NANOS_PER_MILLI: i32 = 1000_000;
    const MILLIS_PER_SEC: i64 = 1000;
    const NANOS_PER_SEC: i32 = 1_000_000_000;

    #[derive(Copy, Clone)]
    pub struct Duration {
        secs: i64,
        nanos: i32, // Always 0 <= nanos < NANOS_PER_SEC
    }

    impl Duration {
        pub fn num_milliseconds(&self) -> i64 {
            let secs_part = self.num_seconds() * MILLIS_PER_SEC;
            let nanos_part = self.nanos_mod_sec() / NANOS_PER_MILLI;
            secs_part + nanos_part as i64
        }

        pub fn milliseconds(milliseconds: i64) -> Duration {
            let (secs, millis) = div_mod_floor_64(milliseconds, MILLIS_PER_SEC);
            let nanos = millis as i32 * NANOS_PER_MILLI;
            Duration { secs: secs, nanos: nanos }
        }

        fn nanos_mod_sec(&self) -> i32 {
            if self.secs < 0 && self.nanos > 0 {
                self.nanos - NANOS_PER_SEC
            } else {
                self.nanos
            }
        }

        fn num_seconds(&self) -> i64 {
            // If secs is negative, nanos should be subtracted from the duration.
            if self.secs < 0 && self.nanos > 0 {
                self.secs + 1
            } else {
                self.secs
            }
        }
    }

    // Copied from libnum
    #[inline]
    fn div_mod_floor_64(this: i64, other: i64) -> (i64, i64) {
        (div_floor_64(this, other), mod_floor_64(this, other))
    }

    #[inline]
    fn div_floor_64(this: i64, other: i64) -> i64 {
        match div_rem_64(this, other) {
            (d, r) if (r > 0 && other < 0)
                || (r < 0 && other > 0) => d - 1,
                (d, _)                         => d,
        }
    }

    #[inline]
    fn mod_floor_64(this: i64, other: i64) -> i64 {
        match this % other {
            r if (r > 0 && other < 0)
                || (r < 0 && other > 0) => r + other,
                r                         => r,
        }
    }

    #[inline]
    fn div_rem_64(this: i64, other: i64) -> (i64, i64) {
        (this / other, this % other)
    }
}
