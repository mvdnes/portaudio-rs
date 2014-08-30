use ll;
use pa::PaError;
use std::c_str::CString;
use util::to_pa_result;

pub type HostApiIndex = uint;

#[repr(u32)]
#[deriving(FromPrimitive)]
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
    Unknown,
}

impl HostApiType
{
    pub fn to_api_index(self) -> Result<HostApiIndex, PaError>
    {
        match unsafe { ll::Pa_HostApiTypeIdToHostApiIndex(self as u32) }
        {
            n if n >= 0 => Ok(n as HostApiIndex),
            m => to_pa_result(m).map(|_| 0),
        }
    }
}

pub struct HostApiInfo
{
    pub api_type: HostApiType,
    pub name: String,
    pub device_count: int,
    pub default_input: Option<int>,
    pub default_output: Option<int>,
}

impl HostApiInfo
{
    fn from_ll(input: &ll::PaHostApiInfo) -> HostApiInfo
    {
        HostApiInfo
        {
            api_type: FromPrimitive::from_u32(input._type).unwrap_or(Unknown),
            name: format!("{}", unsafe { CString::new(input.name, false) }),
            device_count: input.deviceCount as int,
            default_input: match input.defaultInputDevice { n if n >= 0 => Some(n as int), _ => None },
            default_output: match input.defaultOutputDevice { n if n >= 0 => Some(n as int), _ => None },
        }
    }
}

pub struct HostErrorInfo
{
    pub code: int,
    pub text: String,
    pub api_type: HostApiType,
}

impl HostErrorInfo
{
    fn from_ll(input: &ll::PaHostErrorInfo) -> HostErrorInfo
    {
        HostErrorInfo
        {
            code: input.errorCode as int,
            text: format!("{}", unsafe { CString::new(input.errorText, false) }),
            api_type: FromPrimitive::from_u32(input.hostApiType).unwrap_or(Unknown),
        }
    }
}

pub fn get_last_error() -> Option<HostErrorInfo>
{
    unsafe
    {
        ll::Pa_GetLastHostErrorInfo()
            .to_option()
            .map(|s| HostErrorInfo::from_ll(s))
    }
}

pub fn get_count() -> Result<HostApiIndex, PaError>
{
    match unsafe { ll::Pa_GetHostApiCount() }
    {
        n if n >= 0 => Ok(n as HostApiIndex),
        m => to_pa_result(m).map(|_| 0),
    }
}

pub fn get_default_index() -> Result<HostApiIndex, PaError>
{
    match unsafe { ll::Pa_GetDefaultHostApi() }
    {
        n if n >= 0 => Ok(n as HostApiIndex),
        m => to_pa_result(m).map(|_| 0),
    }
}

pub fn get_info(index: HostApiIndex) -> Option<HostApiInfo>
{
    unsafe
    {
        ll::Pa_GetHostApiInfo(index as i32)
            .to_option()
            .map(|s| HostApiInfo::from_ll(s))
    }
}
