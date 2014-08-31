use util::to_pa_result;
use ll;
use std::fmt;
use std::c_str::CString;

pub fn version() -> int
{
    let version = unsafe { ll::Pa_GetVersion() };
    version as int
}

pub fn version_text() -> String
{
    let version = unsafe { CString::new(ll::Pa_GetVersionText(), false) };
    format!("{}", version)
}

pub fn initialize() -> PaResult
{
    to_pa_result(unsafe { ll::Pa_Initialize() })
}

pub fn terminate() -> PaResult
{
    to_pa_result(unsafe { ll::Pa_Terminate() })
}

// PaError and PaResult

#[repr(i32)]
#[deriving(FromPrimitive, PartialEq)]
pub enum PaError
{
    // paNoError is not present in this enum
    NotInitialized = ll::paNotInitialized,
    UnanticipatedHostError = ll::paUnanticipatedHostError,
    InvalidChannelCount = ll::paInvalidChannelCount,
    InvalidSampleRate = ll::paInvalidSampleRate,
    InvalidDevice = ll::paInvalidDevice,
    InvalidFlag = ll::paInvalidFlag,
    SampleFormatNotSupported = ll::paSampleFormatNotSupported,
    BadIODeviceCombination = ll::paBadIODeviceCombination,
    InsufficientMemory = ll::paInsufficientMemory,
    BufferTooBig = ll::paBufferTooBig,
    BufferTooSmall = ll::paBufferTooSmall,
    NullCallback = ll::paNullCallback,
    BadStreamPtr = ll::paBadStreamPtr,
    TimedOut = ll::paTimedOut,
    InternalError = ll::paInternalError,
    DeviceUnavailable = ll::paDeviceUnavailable,
    IncompatibleHostApiSpecificStreamInfo = ll::paIncompatibleHostApiSpecificStreamInfo,
    StreamIsStopped = ll::paStreamIsStopped,
    StreamIsNotStopped = ll::paStreamIsNotStopped,
    InputOverflowed = ll::paInputOverflowed,
    OutputUnderflowed = ll::paOutputUnderflowed,
    HostApiNotFound = ll::paHostApiNotFound,
    InvalidHostApi = ll::paInvalidHostApi,
    CanNotReadFromACallbackStream = ll::paCanNotReadFromACallbackStream,
    CanNotWriteToACallbackStream = ll::paCanNotWriteToACallbackStream,
    CanNotReadFromAnOutputOnlyStream = ll::paCanNotReadFromAnOutputOnlyStream,
    CanNotWriteToAnInputOnlyStream = ll::paCanNotWriteToAnInputOnlyStream,
    IncompatibleStreamHostApi = ll::paIncompatibleStreamHostApi,
    BadBufferPtr = ll::paBadBufferPtr,
    UnknownError,
}

impl fmt::Show for PaError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self
        {
            UnknownError => write!(f, "Unknown Error"),
            other =>
            {
                let message = unsafe { CString::new(ll::Pa_GetErrorText(other as i32), false) };
                write!(f, "{}", message)
            }
        }
    }
}

pub type PaResult = Result<(), PaError>;
