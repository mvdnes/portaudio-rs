//! Info about connected audio devices

use ll;
use util::{to_pa_result, pa_time_to_duration};
use hostapi::HostApiIndex;
use pa::PaError;
use std::time::duration::Duration;
use std::ffi::CStr;

/// Index of a Device
pub type DeviceIndex = u32;

/// Information for a specific device
pub struct DeviceInfo
{
    /// Human readable name
    pub name: String,

    /// Index of the host API this device belongs to
    pub host_api: HostApiIndex,

    /// Maximal number of input channels that can be used
    pub max_input_channels: u32,

    /// Maximal number of ouput channels that can be used
    pub max_output_channels: u32,

    /// Default input latency for interactive performance
    pub default_low_input_latency: Duration,

    /// Default output latency for interactive performance
    pub default_low_output_latency: Duration,

    /// Default input latency for robust non-interactive applications
    pub default_high_input_latency: Duration,

    /// Default output latency for robust non-interactive applications
    pub default_high_output_latency: Duration,

    /// Default sample rate
    pub default_sample_rate: f64,
}

impl DeviceInfo
{
    fn from_ll(input: &ll::PaDeviceInfo) -> DeviceInfo
    {
        DeviceInfo
        {
            name: String::from_utf8_lossy(unsafe { CStr::from_ptr(input.name).to_bytes() }).into_owned(),
            host_api: input.hostApi as HostApiIndex,
            max_input_channels: input.maxInputChannels as u32,
            max_output_channels: input.maxOutputChannels as u32,
            default_low_input_latency: pa_time_to_duration(input.defaultLowInputLatency),
            default_low_output_latency: pa_time_to_duration(input.defaultLowOutputLatency),
            default_high_input_latency: pa_time_to_duration(input.defaultHighInputLatency),
            default_high_output_latency: pa_time_to_duration(input.defaultHighOutputLatency),
            default_sample_rate: input.defaultSampleRate,
        }
    }
}

/// Retrieve the number of available devices.
pub fn get_count() -> Result<u32, PaError>
{
    match unsafe { ll::Pa_GetDeviceCount() }
    {
        n if n >= 0 => Ok(n as u32),
        m => to_pa_result(m).map(|_| 0),
    }
}

/// Retrieve the index of the default input device
///
/// Will return Err(NoDevice) when non are available.
pub fn get_default_input_index() -> Result<DeviceIndex, PaError>
{
    match unsafe { ll::Pa_GetDefaultInputDevice() }
    {
        n if n >= 0 => Ok(n as u32),
        m => to_pa_result(m).map(|_| 0),
    }
}

/// Retrieve the index of the default output device
///
/// Will return Err(NoDevice) when non are available.
pub fn get_default_output_index() -> Result<DeviceIndex, PaError>
{
    match unsafe { ll::Pa_GetDefaultOutputDevice() }
    {
        n if n >= 0 => Ok(n as u32),
        m => to_pa_result(m).map(|_| 0),
    }
}

/// Get info about a particular device
///
/// Returns None when the index is out of range.
pub fn get_info(index: DeviceIndex) -> Option<DeviceInfo>
{
    unsafe
    {
        ll::Pa_GetDeviceInfo(index as i32)
            .as_ref()
            .map(|s| DeviceInfo::from_ll(s))
    }
}

/// Converts a device index from a specific host API to a global device index
///
/// Returns Err(InvalidHostApi) when the host_api is out of range, and Err(InvalidDevice) when
/// host_api_device_index is out of range.
///
/// ```
/// // We retrieve the index of device 3 of api 1
/// let device_index = match portaudio::device::get_from_host_api_device_index(1, 3)
/// {
///     Ok(n) => n,
///     Err(e) => { println!("Error: {:?}", e); return },
/// };
/// ```
pub fn get_from_host_api_device_index(host_api: HostApiIndex, host_api_device_index: u32) -> Result<DeviceIndex, PaError>
{
    match unsafe { ll::Pa_HostApiDeviceIndexToDeviceIndex(host_api as i32, host_api_device_index as i32) }
    {
        n if n >= 0 => Ok(n as DeviceIndex),
        m => to_pa_result(m).map(|_| 0),
    }
}
