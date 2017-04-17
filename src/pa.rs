//! General utilities for PortAudio

use util::to_pa_result;
use ll;
use std::fmt;
use std::ffi::CStr;

/// PortAudio version
pub fn version() -> i32
{
    let version = unsafe { ll::Pa_GetVersion() };
    version as i32
}

/// Human-readable PortAudio version
pub fn version_text() -> String
{
    let version_c = unsafe { ll::Pa_GetVersionText() };
    let version_s = String::from_utf8_lossy(unsafe { CStr::from_ptr(version_c as *const _).to_bytes() });
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
#[derive(PartialEq, Copy, Clone)]
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

impl PaError {
    /// Get the enum value corresponding to the given i32
    pub fn from_i32(num: i32) -> PaError {
        match num {
            ll::paNotInitialized => PaError::NotInitialized,
            ll::paUnanticipatedHostError => PaError::UnanticipatedHostError,
            ll::paInvalidChannelCount => PaError::InvalidChannelCount,
            ll::paInvalidSampleRate => PaError::InvalidSampleRate,
            ll::paInvalidDevice => PaError::InvalidDevice,
            ll::paInvalidFlag => PaError::InvalidFlag,
            ll::paSampleFormatNotSupported => PaError::SampleFormatNotSupported,
            ll::paBadIODeviceCombination => PaError::BadIODeviceCombination,
            ll::paInsufficientMemory => PaError::InsufficientMemory,
            ll::paBufferTooBig => PaError::BufferTooBig,
            ll::paBufferTooSmall => PaError::BufferTooSmall,
            ll::paNullCallback => PaError::NullCallback,
            ll::paBadStreamPtr => PaError::BadStreamPtr,
            ll::paTimedOut => PaError::TimedOut,
            ll::paInternalError => PaError::InternalError,
            ll::paDeviceUnavailable => PaError::DeviceUnavailable,
            ll::paIncompatibleHostApiSpecificStreamInfo => PaError::IncompatibleHostApiSpecificStreamInfo,
            ll::paStreamIsStopped => PaError::StreamIsStopped,
            ll::paStreamIsNotStopped => PaError::StreamIsNotStopped,
            ll::paInputOverflowed => PaError::InputOverflowed,
            ll::paOutputUnderflowed => PaError::OutputUnderflowed,
            ll::paHostApiNotFound => PaError::HostApiNotFound,
            ll::paInvalidHostApi => PaError::InvalidHostApi,
            ll::paCanNotReadFromACallbackStream => PaError::CanNotReadFromACallbackStream,
            ll::paCanNotWriteToACallbackStream => PaError::CanNotWriteToACallbackStream,
            ll::paCanNotReadFromAnOutputOnlyStream => PaError::CanNotReadFromAnOutputOnlyStream,
            ll::paCanNotWriteToAnInputOnlyStream => PaError::CanNotWriteToAnInputOnlyStream,
            ll::paIncompatibleStreamHostApi => PaError::IncompatibleStreamHostApi,
            ll::paBadBufferPtr => PaError::BadBufferPtr,
            _ => PaError::UnknownError,
        }
    }
}

impl fmt::Display for PaError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self
        {
            PaError::UnknownError => write!(f, "Unknown Error"),
            other =>
            {
                let message_c = unsafe { ll::Pa_GetErrorText(other as i32) };
                let message_s = String::from_utf8_lossy(unsafe { CStr::from_ptr(message_c as *const _).to_bytes() });
                f.write_str(&*message_s)
            }
        }
    }
}

impl fmt::Debug for PaError
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        ::std::fmt::Display::fmt(self, fmt)
    }
}

/// A result type wrapping PaError.
///
/// The original NoError is mapped to Ok(()) and other values mapped to Err(x)
pub type PaResult = Result<(), PaError>;
