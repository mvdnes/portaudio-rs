use pa::{PaResult, PaError};
use ll;

pub use self::duration::Duration;

pub fn to_pa_result(code: i32) -> PaResult
{
    if code == ll::paNoError
    {
        return Ok(());
    }
    Err(PaError::from_i32(code))
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
    #[derive(Copy, Clone)]
    pub struct Duration {
        ms: i64,
    }

    impl Duration {
        pub fn num_milliseconds(&self) -> i64 {
            self.ms
        }

        pub fn milliseconds(milliseconds: i64) -> Duration {
            Duration { ms: milliseconds }
        }
    }
}
