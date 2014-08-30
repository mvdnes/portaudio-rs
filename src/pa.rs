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

pub type PaStreamCallback<T> = |input: &[T], output: &mut [T]|:'static -> PaStreamCallbackResult;

extern "C" fn stream_callback<T>(input: *const ::libc::c_void,
                              output: *mut ::libc::c_void,
                              frame_count: ::libc::c_ulong,
                              _time_info: *const ll::PaStreamCallbackTimeInfo,
                              _status_flags: ll::PaStreamCallbackFlags,
                              user_data: *mut ::libc::c_void) -> ::libc::c_int
{
    let stream_data: Box<PaStreamUserData<T>> = unsafe { ::std::mem::transmute(user_data) };
    let input_buffer: &[T] = unsafe
    {
        ::std::mem::transmute(
            ::std::raw::Slice { data: input as *const T, len: frame_count as uint * stream_data.num_input }
        )
    };
    let output_buffer: &mut [T] = unsafe
    {
        ::std::mem::transmute(
            ::std::raw::Slice { data: output as *const T, len: frame_count as uint * stream_data.num_output }
        )
    };

    let result = (stream_data.callback)(input_buffer, output_buffer);

    unsafe { ::std::mem::forget(stream_data); }

    result as i32
}

struct PaStreamUserData<T>
{
    num_input: uint,
    num_output: uint,
    callback: PaStreamCallback<T>,
}

trait PaType { fn as_sample_format(_: Option<Self>) -> u64; }
impl PaType for f32 { fn as_sample_format(_: Option<f32>) -> u64 { 0x00000001 } }
impl PaType for i32 { fn as_sample_format(_: Option<i32>) -> u64 { 0x00000002 } }
impl PaType for i16 { fn as_sample_format(_: Option<i16>) -> u64 { 0x00000008 } }
impl PaType for i8 { fn as_sample_format(_: Option<i8>) -> u64 { 0x00000010 } }
impl PaType for u8 { fn as_sample_format(_: Option<u8>) -> u64 { 0x00000020 } }

pub struct PaStream<T>
{
    pa_stream: *mut ll::PaStream,
    _callback: Box<PaStreamUserData<T>>,
}

impl<T: PaType> PaStream<T>
{
    pub fn open_default_stream(num_input_channels: uint,
                               num_output_channels: uint,
                               sample_rate: f64,
                               frames_per_buffer: u64,
                               callback: PaStreamCallback<T>)
                              -> Result<PaStream<T>, PaError>
    {
        unsafe
        {
            let userdata = box PaStreamUserData
            {
                num_input: num_input_channels,
                num_output: num_output_channels,
                callback: callback,
            };
            let mut pa_stream = ::std::ptr::mut_null();

            let ud_pointer: *mut ::libc::c_void = ::std::mem::transmute(userdata);
            let ud_pointer_2 = ud_pointer.clone();
            let code = ll::Pa_OpenDefaultStream(&mut pa_stream as *mut *mut ll::PaStream,
                                                num_input_channels as i32,
                                                num_output_channels as i32,
                                                PaType::as_sample_format(None::<T>),
                                                sample_rate,
                                                frames_per_buffer,
                                                stream_callback::<T>,
                                                ud_pointer);
            match to_pa_result(code)
            {
                Ok(()) => Ok(PaStream { pa_stream: pa_stream,
                                        _callback: ::std::mem::transmute(ud_pointer_2)
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

    fn close(&self) -> PaResult
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
impl<T: PaType> Drop for PaStream<T>
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
