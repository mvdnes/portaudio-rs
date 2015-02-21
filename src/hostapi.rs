//! Info module for available audio host API's

use ll;
use pa::PaError;
use std::ffi::CStr;
use std::num::FromPrimitive;
use util::to_pa_result;

/// Index number of a Host API
pub type HostApiIndex = u32;

/// Possible Host API types
#[repr(u32)]
#[derive(FromPrimitive, Copy)]
#[allow(missing_docs)]
pub enum HostApiType
{
    InDevelopment = ll::paInDevelopment,
    DirectSound = ll::paDirectSound,
    MME = ll::paMME,
    ASIO = ll::paASIO,
    SoundManager = ll::paSoundManager,
    CoreAudio = ll::paCoreAudio,
    OSS = ll::paOSS,
    ALSA = ll::paALSA,
    AL = ll::paAL,
    BeOS = ll::paBeOS,
    WDMKS = ll::paWDMKS,
    JACK = ll::paJACK,
    WASAPI = ll::paWASAPI,
    AudioScienceHPI = ll::paAudioScienceHPI,

    /// Added for when FromPrimitive returns None
    Unknown,
}

impl HostApiType
{
    /// Convert a static host API unique identifier, into a runtime host API index.
    pub fn to_api_index(self) -> Result<HostApiIndex, PaError>
    {
        match unsafe { ll::Pa_HostApiTypeIdToHostApiIndex(self as u32) }
        {
            n if n >= 0 => Ok(n as HostApiIndex),
            m => to_pa_result(m).map(|_| 0),
        }
    }
}

/// Information about a specific host API
pub struct HostApiInfo
{
    /// The type of the API
    pub api_type: HostApiType,

    /// Human-readable name of the API
    pub name: String,

    /// Number of devices this API has
    pub device_count: u32,

    /// Default input device of the API. Is None if there is no input device available.
    pub default_input: Option<u32>,

    /// Default output device of the API. Is None if there is no output device available.
    pub default_output: Option<u32>,
}

impl HostApiInfo
{
    fn from_ll(input: &ll::PaHostApiInfo) -> HostApiInfo
    {
        HostApiInfo
        {
            api_type: FromPrimitive::from_u32(input._type).unwrap_or(HostApiType::Unknown),
            name: String::from_utf8_lossy(unsafe { CStr::from_ptr(input.name).to_bytes() }).into_owned(),
            device_count: input.deviceCount as u32,
            default_input: match input.defaultInputDevice { n if n >= 0 => Some(n as u32), _ => None },
            default_output: match input.defaultOutputDevice { n if n >= 0 => Some(n as u32), _ => None },
        }
    }
}

/// Error info obtained by get_last_error
pub struct HostErrorInfo
{
    /// The error code given
    pub code: i32,

    /// A human readable error message
    pub text: String,

    /// The type of the API that produced the error
    pub api_type: HostApiType,
}

impl HostErrorInfo
{
    fn from_ll(input: &ll::PaHostErrorInfo) -> HostErrorInfo
    {
        HostErrorInfo
        {
            code: input.errorCode as i32,
            text: String::from_utf8_lossy(unsafe { CStr::from_ptr(input.errorText).to_bytes() }).into_owned(),
            api_type: FromPrimitive::from_u32(input.hostApiType).unwrap_or(HostApiType::Unknown),
        }
    }
}

/// Return information about the last host error encountered.
///
/// The values in this structure will only be valid if a PortAudio function has previously returned
/// the UnanticipatedHostError error code.
pub fn get_last_error() -> Option<HostErrorInfo>
{
    unsafe
    {
        ll::Pa_GetLastHostErrorInfo()
            .as_ref()
            .map(|s| HostErrorInfo::from_ll(s))
    }
}

/// Get the number of host API's available
pub fn get_count() -> Result<u32, PaError>
{
    match unsafe { ll::Pa_GetHostApiCount() }
    {
        n if n >= 0 => Ok(n as HostApiIndex),
        m => to_pa_result(m).map(|_| 0),
    }
}

/// Get the default Host API
pub fn get_default_index() -> Result<HostApiIndex, PaError>
{
    match unsafe { ll::Pa_GetDefaultHostApi() }
    {
        n if n >= 0 => Ok(n as HostApiIndex),
        m => to_pa_result(m).map(|_| 0),
    }
}

/// Get information about a specific Host API
///
/// Returns None when an invalid index is given
pub fn get_info(index: HostApiIndex) -> Option<HostApiInfo>
{
    unsafe
    {
        ll::Pa_GetHostApiInfo(index as i32)
            .as_ref()
            .map(|s| HostApiInfo::from_ll(s))
    }
}
