use pa::{PaResult, PaError};
use ll;

use std::time::Duration;

pub fn to_pa_result(code: i32) -> PaResult
{
    if code == ll::paNoError
    {
        return Ok(());
    }
    Err(PaError::from_i32(code))
}

pub fn pa_time_to_duration(input: f64) -> Duration
{
    assert!(input >= 0.0);
    let secs = input.floor();
    let nanos = (input - secs) * 1e9;
    Duration::new(secs as u64, nanos as u32)
}

pub fn duration_to_pa_time(duration: Duration) -> f64
{
    duration.as_secs() as f64 + (duration.subsec_nanos() as f64 * 1e-9)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_conversion() {
        let seconds = 2.512389131321938123681627;
        let duration = super::pa_time_to_duration(seconds);
        let seconds2 = super::duration_to_pa_time(duration);

        println!("{}", (seconds - seconds2).abs());
        assert!((seconds - seconds2).abs() <= 1e-8);
    }
}
