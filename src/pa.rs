//! General utilities for PortAudio

use util::to_pa_result;
use ll;
use std::fmt;
use std::ffi::c_str_to_bytes;

/// PortAudio version
pub fn version() -> int
{
    let version = unsafe { ll::Pa_GetVersion() };
    version as int
}

/// Human-readable PortAudio version
pub fn version_text() -> String
{
    let version_c = unsafe { ll::Pa_GetVersionText() };
    let version_s = String::from_utf8_lossy(unsafe { c_str_to_bytes(&version_c) });
    version_s.into_owned()
}

/// Initialize the PortAudio API
///
/// Each successful call must be matched by a call to terminate
pub fn initialize() -> PaResult
{
    to_pa_result(unsafe { ll::Pa_Initialize() })
}

/// Terminate the PortAudio API
///
/// Call this function exactly once for each successful call to initialize
pub fn terminate() -> PaResult
{
    to_pa_result(unsafe { ll::Pa_Terminate() })
}

// PaError and PaResult

/// Enum for all possible errors given by PortAudio
///
/// The NoError value (0) is not present since the Result type can be used then.
#[repr(i32)]
#[derive(FromPrimitive, PartialEq, Copy)]
#[allow(missing_docs)]
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

    /// Added variant for when FromPrimitive returns None
    UnknownError,
}

impl fmt::Show for PaError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self
        {
            PaError::UnknownError => write!(f, "Unknown Error"),
            other =>
            {
                let message_c = unsafe { ll::Pa_GetErrorText(other as i32) };
                let message_s = String::from_utf8_lossy(unsafe { c_str_to_bytes(&message_c) });
                f.write_str(&*message_s)
            }
        }
    }
}

/// A result type wrapping PaError.
///
/// The original NoError is mapped to Ok(()) and other values mapped to Err(x)
pub type PaResult = Result<(), PaError>;
