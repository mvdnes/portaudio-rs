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
#[deriving(FromPrimitive)]
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

fn to_pa_result(code: i32) -> PaResult
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

// PaStreamCallback

#[repr(u32)]
pub enum PaStreamCallbackResult
{
    Continue = ll::paContinue,
    Complete = ll::paComplete,
    Abort = ll::paAbort,
}

pub type PaStreamCallback = |input: &[f32], output: &mut [f32]|:'static -> PaStreamCallbackResult;

extern "C" fn stream_callback(input: *const ::libc::c_void,
                              output: *mut ::libc::c_void,
                              frame_count: ::libc::c_ulong,
                              _time_info: *const ll::PaStreamCallbackTimeInfo,
                              _status_flags: ll::PaStreamCallbackFlags,
                              user_data: *mut ::libc::c_void) -> ::libc::c_int
{
    let buffer_length = (frame_count * 2) as uint;
    let input_buffer: &[f32] = unsafe
    {
        ::std::mem::transmute(
            ::std::raw::Slice { data: input as *const f32, len: buffer_length}
        )
    };
    let output_buffer: &mut [f32] = unsafe
    {
        ::std::mem::transmute(
            ::std::raw::Slice { data: output as *const f32, len: buffer_length }
        )
    };

    let f: Box<PaStreamCallback> = unsafe { ::std::mem::transmute(user_data) };
    let result = (*f)(input_buffer, output_buffer);

    match result
    {
        Complete | Abort => {},
        Continue => unsafe { ::std::mem::forget(f); },
    }

    result as i32
}

pub struct PaStream
{
    pa_stream: *mut ll::PaStream,
    _callback: Box<PaStreamCallback>,
}

impl PaStream
{
    pub fn open_easy_stream(sample_rate: f64,
                           frames_per_buffer: u64,
                           callback: PaStreamCallback) -> Result<PaStream, PaError>
    {
        unsafe
        {
            let mut pa_stream = ::std::ptr::mut_null();
            let user_data: *mut ::libc::c_void = ::std::mem::transmute(box callback);
            let callback_pointer = user_data.clone();
            let code = ll::Pa_OpenDefaultStream(&mut pa_stream as *mut *mut ll::PaStream,
                                                0,
                                                2,
                                                1,
                                                sample_rate,
                                                frames_per_buffer,
                                                stream_callback,
                                                user_data);
            match to_pa_result(code)
            {
                Ok(()) => Ok(PaStream { pa_stream: pa_stream,
                                        _callback: ::std::mem::transmute(callback_pointer)
                             }),
                Err(v) => Err(v),
            }
        }
    }

    pub fn start(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_StartStream(self.pa_stream) })
    }

    pub fn stop(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_StopStream(self.pa_stream) })
    }

    pub fn abort(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_AbortStream(self.pa_stream) })
    }

    pub fn close(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_CloseStream(self.pa_stream) })
    }

    pub fn is_stopped(&self) -> Result<bool, PaError>
    {
        match unsafe { ll::Pa_IsStreamStopped(self.pa_stream) }
        {
            1 => Ok(true),
            n => to_pa_result(n).map(|_| false),
        }
    }

    pub fn is_active(&self) -> Result<bool, PaError>
    {
        match unsafe { ll::Pa_IsStreamActive(self.pa_stream) }
        {
            1 => Ok(true),
            n => to_pa_result(n).map(|_| false),
        }
    }
}

#[unsafe_destructor]
impl Drop for PaStream
{
    fn drop(&mut self)
    {
        match self.close()
        {
            Err(v) => error!("Error: {}", v),
            Ok(_) => {},
        };
    }
}
