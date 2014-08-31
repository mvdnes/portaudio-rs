use pa::{PaResult, PaError, UnknownError};
use ll;
use std::time::duration::Duration;

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
        None => Err(UnknownError),
    }
}

pub fn pa_time_to_duration(seconds: f64) -> Duration
{
    Duration::milliseconds((seconds * 1000.0) as i64)
}
