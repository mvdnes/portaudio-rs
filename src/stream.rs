//! Contains the Stream class and associated values

use ll;
use pa::{PaError, PaResult};
use device::DeviceIndex;
use util::{to_pa_result, pa_time_to_duration, duration_to_pa_time};
use std::raw::Slice;
use std::mem;
use std::time::duration::Duration;
use libc::c_void;

type StreamCallbackType = extern "C" fn(*const c_void, *mut c_void, ::libc::c_ulong, *const ll::PaStreamCallbackTimeInfo, ll::PaStreamCallbackFlags, *mut c_void) -> ::libc::c_int;
type StreamFinishedCallbackType = extern "C" fn(*mut c_void);

/// Allowable return values for a StreamCallback
#[repr(u32)]
#[derive(Copy)]
pub enum StreamCallbackResult
{
    /// Continue invoking the callback
    Continue = ll::paContinue,

    /// Stop invoking the callback and finish once everything has played
    Complete = ll::paComplete,

    /// Stop invoking the callback and finish as soon as possible
    Abort = ll::paAbort,
}

/// Callback to consume, process or generate audio
pub type StreamCallback<'a, I, O> = FnMut(&[I], &mut [O], StreamTimeInfo, StreamCallbackFlags) -> StreamCallbackResult + 'a;

/// Callback to be fired when a StreamCallback is stopped
pub type StreamFinishedCallback<'a> = FnMut() + 'a;

struct StreamUserData<'a, I, O>
{
    num_input: uint,
    num_output: uint,
    callback: Option<&'a mut StreamCallback<'a, I, O>>,
    finished_callback: Option<&'a mut StreamFinishedCallback<'a>>,
}

/// Time information for various stream related values
#[derive(Copy)]
pub struct StreamTimeInfo
{
    /// Timestamp for the ADC capture time of the first frame
    pub input_adc_time: Duration,

    /// Timestamp that the callback was invoked
    pub current_time: Duration,

    /// Timestamp for the DAC output time of the first frame
    pub output_dac_time: Duration,
}

impl StreamTimeInfo
{
    fn from_ll(data: &ll::PaStreamCallbackTimeInfo) -> StreamTimeInfo
    {
        StreamTimeInfo
        {
            input_adc_time: pa_time_to_duration(data.inputBufferAdcTime),
            current_time: pa_time_to_duration(data.currentTime),
            output_dac_time: pa_time_to_duration(data.outputBufferDacTime),
        }
    }
}

bitflags!(
    #[doc="Flags indicating the status of the callback"]
    flags StreamCallbackFlags: u64 {
        #[doc="Indicates that the callback has inserted one or more zeroes since not enough data was available"]
        const INPUT_UNDERFLOW = 0x01,

        #[doc="Indicates that the callback has discarded some data"]
        const INPUT_OVERFLOW = 0x02,

        #[doc="Indicates that extra data was inserted in the output since there was not engough available"]
        const OUTPUT_UNDERFLOW = 0x04,

        #[doc="Indicates that certain data was discarded since there was no room"]
        const OUTPUT_OVERFLOW = 0x08,

        #[doc="Some or all of the output data will be used to prime the stream, input data may be zero"]
        const PRIMING_OUTPUT = 0x10
    }
);

bitflags!(
    #[doc="Flags used to control the behavior of a stream"]
    flags StreamFlags: u64 {
        #[doc="Disable clipping of out of range samples"]
        const CLIP_OFF                                   = 0x00000001,

        #[doc="Disable dithering"]
        const DITHER_OFF                                 = 0x00000002,

        #[doc="Request that a full duplex stream will not discard overflowed input samples. The frames_per_buffer must be set to unspecified (0)"]
        const NEVER_DROP_INPUT                           = 0x00000004,

        #[doc="Call the stream callback to fill initial output buffers, rather than priming the buffers with silence"]
        const PRIME_OUTPUT_BUFFERS_USING_STREAM_CALLBACK = 0x00000008,

        #[doc="Range for platform specific flags. Not all of the upper 16 bits need to be set at the same time."]
        const PLATFORM_SPECIFIC                          = 0xFFFF0000
    }
);

extern "C" fn stream_callback<I, O>(input: *const c_void,
                                    output: *mut c_void,
                                    frame_count: ::libc::c_ulong,
                                    time_info: *const ll::PaStreamCallbackTimeInfo,
                                    status_flags: ll::PaStreamCallbackFlags,
                                    user_data: *mut c_void) -> ::libc::c_int
{
    let mut stream_data: Box<StreamUserData<I, O>> = unsafe { mem::transmute(user_data) };

    let input_buffer: &[I] = unsafe
    {
        mem::transmute(
            Slice { data: input as *const I, len: frame_count as uint * stream_data.num_input }
        )
    };
    let output_buffer: &mut [O] = unsafe
    {
        mem::transmute(
            Slice { data: output as *const O, len: frame_count as uint * stream_data.num_output }
        )
    };

    let flags = StreamCallbackFlags::from_bits_truncate(status_flags);

    // PortAudio will probably never set time_info to NULL
    let time_info_ll = unsafe { time_info.as_ref() }.unwrap();
    let timeinfo = StreamTimeInfo::from_ll(time_info_ll);

    let result = match stream_data.callback
    {
        Some(ref mut f) => (*f)(input_buffer, output_buffer, timeinfo, flags),
        None => StreamCallbackResult::Abort,
    };

    unsafe { mem::forget(stream_data); }

    result as i32
}

extern "C" fn stream_finished_callback<I, O>(user_data: *mut c_void)
{
    let mut stream_data: Box<StreamUserData<I, O>> = unsafe { mem::transmute(user_data) };
    match stream_data.finished_callback
    {
        Some(ref mut f) => (*f)(),
        None => {},
    };

    unsafe { mem::forget(stream_data); }
}

/// Types that are allowed to be used as samples in a Stream
///
/// *WARNING*: It is not advised to implement this trait for any other types as the size and flag
/// may not be the correct one.
pub trait SampleType
{
    /// Should return the PortAudio flag which corresponds to the type
    fn sample_format(_: Option<Self>) -> u64;
}
impl SampleType for f32 { fn sample_format(_: Option<f32>) -> u64 { 0x00000001 } }
impl SampleType for i32 { fn sample_format(_: Option<i32>) -> u64 { 0x00000002 } }
impl SampleType for i16 { fn sample_format(_: Option<i16>) -> u64 { 0x00000008 } }
impl SampleType for i8 { fn sample_format(_: Option<i8>) -> u64 { 0x00000010 } }
impl SampleType for u8 { fn sample_format(_: Option<u8>) -> u64 { 0x00000020 } }

fn get_sample_format<T: SampleType>() -> u64
{
    SampleType::sample_format(None::<T>)
}

#[doc(hidden)]
pub fn get_sample_size<T: SampleType>() -> Result<uint, PaError>
{
    match unsafe { ll::Pa_GetSampleSize(get_sample_format::<T>()) }
    {
        n if n >= 0 => Ok(n as uint),
        m => to_pa_result(m).map(|_| 0),
    }
}

/// Argument to Stream::open() or Stream::open_default() to allow PortAudio itself determine the
/// optimal number of frames per buffer. This number may differ each time the callback is called.
pub const FRAMES_PER_BUFFER_UNSPECIFIED: u64 = 0;

/// An object for an PortAudio stream
///
/// Streams can have an input type I and output type O.
pub struct Stream<'a, I, O>
{
    pa_stream: *mut ll::PaStream,
    inputs: uint,
    outputs: uint,
    user_data: Box<StreamUserData<'a, I, O>>,
}

impl<'a, T: SampleType + Send> Stream<'a, T, T>
{
    /// Constructs a stream using the default input and output devices
    ///
    /// ## Arguments
    /// * num_input_channels: Desired number of input channels
    /// * num_output_channels: Desired number of output channels
    /// * sample_rate: Sample rate of the stream
    /// * frames_per_buffer: Number of frames per buffer. Use FRAMES_PER_BUFFER_UNSPECIFIED to let
    /// portaudio determine the optimal number.
    /// * callback: Some(callback) which PortAudio will call to read/write the buffers, or None
    /// when using the read and write methods
    pub fn open_default(num_input_channels: uint,
                        num_output_channels: uint,
                        sample_rate: f64,
                        frames_per_buffer: u64,
                        callback: Option<&'a mut StreamCallback<'a, T, T>>)
                       -> Result<Stream<'a, T, T>, PaError>
    {
        let callback_pointer = match callback
        {
            Some(_) => Some(stream_callback::<T, T> as StreamCallbackType),
            None => None,
        };
        let userdata = Box::new(StreamUserData
        {
            num_input: num_input_channels,
            num_output: num_output_channels,
            callback: callback,
            finished_callback: None,
        });
        let mut pa_stream = ::std::ptr::null_mut();

        let pointer_for_callback: *mut c_void = unsafe { mem::transmute(userdata) };
        let pointer_for_struct = pointer_for_callback.clone();

        let code = unsafe
        {
            ll::Pa_OpenDefaultStream(&mut pa_stream as *mut *mut ll::PaStream,
                                     num_input_channels as i32,
                                     num_output_channels as i32,
                                     get_sample_format::<T>(),
                                     sample_rate,
                                     frames_per_buffer,
                                     callback_pointer,
                                     pointer_for_callback)
        };

        match to_pa_result(code)
        {
            Ok(()) => Ok(Stream { pa_stream: pa_stream,
                                  user_data: unsafe { mem::transmute(pointer_for_struct) },
                                  inputs: num_input_channels,
                                  outputs: num_output_channels,
                         }),
            Err(v) => Err(v),
        }
    }
}

impl<'a, I: SampleType + Send, O: SampleType + Send> Stream<'a, I, O>
{
    /// Constructs a stream with the desired input and output specifications
    ///
    /// ## Arguments
    /// * input: Specification for the input channel
    /// * output: Specification for the output channel
    /// * sample_rate: Sample rate of the stream
    /// * frames_per_buffer: Number of frames per buffer. Use FRAMES_PER_BUFFER_UNSPECIFIED to let
    /// portaudio determine the optimal number.
    /// * flags: Additional flags for the behaviour of the stream
    /// * callback: Some(callback) which PortAudio will call to read/write the buffers, or None
    /// when using the read and write methods
    pub fn open(input: StreamParameters<I>,
                output: StreamParameters<O>,
                sample_rate: f64,
                frames_per_buffer: u64,
                flags: StreamFlags,
                callback: Option<&'a mut StreamCallback<'a, I, O>>)
               -> Result<Stream<'a, I, O>, PaError>
    {
        let callback_pointer = match callback
        {
            Some(_) => Some(stream_callback::<I, O> as StreamCallbackType),
            None => None,
        };

        let user_data = Box::new(StreamUserData
        {
            num_input: input.channel_count,
            num_output: output.channel_count,
            callback: callback,
            finished_callback: None,
        });

        let mut pa_stream = ::std::ptr::null_mut();
        let pointer_for_callback: *mut c_void = unsafe { mem::transmute(user_data) };
        let pointer_for_struct = pointer_for_callback.clone();

        let result = unsafe
        {
            ll::Pa_OpenStream(&mut pa_stream,
                              &input.to_ll(),
                              &output.to_ll(),
                              sample_rate,
                              frames_per_buffer,
                              flags.bits,
                              callback_pointer,
                              pointer_for_callback)
        };

        match to_pa_result(result)
        {
            Ok(()) => Ok(Stream { pa_stream: pa_stream,
                                  user_data: unsafe { mem::transmute(pointer_for_struct) },
                                  inputs: input.channel_count,
                                  outputs: output.channel_count,
                      }),
            Err(v) => Err(v),
        }
    }

    /// Starts the stream
    pub fn start(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_StartStream(self.pa_stream) })
    }

    /// Stops the stream. It will block untill all audio has finished playing
    pub fn stop(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_StopStream(self.pa_stream) })
    }

    /// Stop stream immediately without waiting for the buffers to complete
    pub fn abort(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_AbortStream(self.pa_stream) })
    }

    fn close(&self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_CloseStream(self.pa_stream) })
    }

    /// Returns wether the stream is stopped
    pub fn is_stopped(&self) -> Result<bool, PaError>
    {
        match unsafe { ll::Pa_IsStreamStopped(self.pa_stream) }
        {
            1 => Ok(true),
            n => to_pa_result(n).map(|_| false),
        }
    }

    /// Returns wether the stream is active
    pub fn is_active(&self) -> Result<bool, PaError>
    {
        match unsafe { ll::Pa_IsStreamActive(self.pa_stream) }
        {
            1 => Ok(true),
            n => to_pa_result(n).map(|_| false),
        }
    }

    /// Get the number of frames that can be read from the stream without waiting
    pub fn num_read_available(&self) -> Result<uint, PaError>
    {
        match unsafe { ll::Pa_GetStreamReadAvailable(self.pa_stream) }
        {
            n if n >= 0 => { Ok(n as uint) },
            n => to_pa_result(n as i32).map(|_| 0),
        }
    }

    /// Get the number of frames that can be written to the stream without waiting
    pub fn num_write_available(&self) -> Result<uint, PaError>
    {
        match unsafe { ll::Pa_GetStreamWriteAvailable(self.pa_stream) }
        {
            n if n >= 0 => { Ok(n as uint) },
            n => to_pa_result(n as i32).map(|_| 0),
        }
    }

    /// Write the given buffer to the stream. This function blocks
    ///
    /// Possible Error codes:
    ///
    /// * `CanNotWriteToAnInputOnlyStream`: when num_output_channels = 0
    /// * `BadBufferPtr`: when buffer.len() is not a multiple of num_output_channels
    /// * Some other error given by PortAudio
    pub fn write(&self, buffer: &[O]) -> PaResult
    {
        if self.outputs == 0
        {
            return Err(PaError::CanNotWriteToAnInputOnlyStream)
        }

        // Ensure the buffer is the correct size.
        if buffer.len() % self.outputs != 0
        {
            return Err(PaError::BadBufferPtr)
        }

        let pointer = buffer.as_ptr() as *const c_void;
        let frames = (buffer.len() / self.outputs) as u64;

        to_pa_result(unsafe { ll::Pa_WriteStream(self.pa_stream, pointer, frames) })
    }

    /// Reads the requested number of frames from the input devices. This function blocks until
    /// the whole buffer has been filled.
    ///
    /// Will return `CanNotReadFromAnOutputOnlyStream` if num_input_channels = 0.
    pub fn read(&self, frames: uint) -> Result<Vec<I>, PaError>
    {
        if self.inputs == 0 { return Err(PaError::CanNotReadFromAnOutputOnlyStream) }

        // We create a buffer with the needed capacity. Then we feed that to the library, which
        // will fill the buffer accordingly. Afterwards, we set the length of the vector as all its
        // elements are now initialized.
        let vec_len = frames * self.inputs;
        let mut buffer = Vec::with_capacity(vec_len);

        let buffer_ptr = buffer.as_mut_ptr() as *mut c_void;
        match to_pa_result(unsafe { ll::Pa_ReadStream(self.pa_stream, buffer_ptr, frames as u64) })
        {
            Ok(()) =>
            {
                unsafe { buffer.set_len(vec_len); }
                Ok(buffer)
            },
            Err(e) => Err(e),
        }
    }

    /// Returns the cpu load the stream callback consumes. This will return 0.0 if the stream uses
    /// blocking read/write, or if an error occured.
    pub fn cpu_load(&self) -> f64
    {
        unsafe { ll::Pa_GetStreamCpuLoad(self.pa_stream) }
    }

    /// Get the current timestamp of the stream
    pub fn time(&self) -> Duration
    {
        let time = unsafe { ll::Pa_GetStreamTime(self.pa_stream) };
        pa_time_to_duration(time)
    }

    /// Get the actual latencies and sample rate
    ///
    /// Returns None when the stream is invalid or an error occured
    pub fn info(&self) -> Option<StreamInfo>
    {
        unsafe
        {
            ll::Pa_GetStreamInfo(self.pa_stream)
                .as_ref()
                .map(|s| StreamInfo::from_ll(s))
        }
    }

    /// Set a callback which is to be called when the StreamCallback finishes
    pub fn set_finished_callback(&mut self, finished_callback: &'a mut StreamFinishedCallback<'a>) -> PaResult
    {
        self.user_data.finished_callback = Some(finished_callback);
        let callback_pointer = Some(stream_finished_callback::<I, O> as StreamFinishedCallbackType);
        to_pa_result(unsafe { ll::Pa_SetStreamFinishedCallback(self.pa_stream, callback_pointer) })
    }

    /// Remove any previously attached finish callback
    pub fn unset_finished_callback(&mut self) -> PaResult
    {
        self.user_data.finished_callback = None;
        to_pa_result(unsafe { ll::Pa_SetStreamFinishedCallback(self.pa_stream, None) })
    }
}

#[unsafe_destructor]
impl<'a, I: SampleType + Send, O: SampleType + Send> Drop for Stream<'a, I, O>
{
    fn drop(&mut self)
    {
        match self.close()
        {
            Err(v) => error!("Stream drop error: {:?}", v),
            Ok(_) => {},
        };
    }
}

/// Stream parameters to be used with Stream::open()
#[derive(Copy)]
pub struct StreamParameters<T>
{
    /// Index of the device to use
    pub device: DeviceIndex,

    /// Requested number of channels
    pub channel_count: uint,

    /// Desired latency of the stream
    pub suggested_latency: Duration,
}

impl<T: SampleType> StreamParameters<T>
{
    fn to_ll(&self) -> ll::Struct_PaStreamParameters
    {
        ll::Struct_PaStreamParameters
        {
            device: self.device as i32,
            channelCount: self.channel_count as i32,
            sampleFormat: get_sample_format::<T>(),
            suggestedLatency: duration_to_pa_time(self.suggested_latency),
            hostApiSpecificStreamInfo: ::std::ptr::null_mut(),
        }
    }
}

/// Returns Ok when the StreamParameters are supported. This ignores the latency field.
pub fn is_format_supported<I: SampleType, O: SampleType>(input: StreamParameters<I>, output: StreamParameters<O>, sample_rate: f64) -> PaResult
{
    to_pa_result(unsafe { ll::Pa_IsFormatSupported(&input.to_ll(), &output.to_ll(), sample_rate) })
}

/// Information about the actual latency and sample rate values the stream uses
#[derive(Copy)]
pub struct StreamInfo
{
    /// Input latency
    pub input_latency: Duration,

    /// Output latency
    pub output_latency: Duration,

    /// Sample rate
    pub sample_rate: f64,
}

impl StreamInfo
{
    fn from_ll(data: &ll::PaStreamInfo) -> StreamInfo
    {
        StreamInfo
        {
            input_latency: pa_time_to_duration(data.inputLatency),
            output_latency: pa_time_to_duration(data.outputLatency),
            sample_rate: data.sampleRate,
        }
    }
}

#[cfg(test)]
mod test
{
    use super::SampleType;

    // This test asserts that the sizes used by PortAudio are the same as
    // those used by Rust
    #[test]
    fn sample_sizes()
    {
        test_sample_size::<f32>();
        test_sample_size::<i32>();
        test_sample_size::<i16>();
        test_sample_size::<i8>();
        test_sample_size::<u8>();
    }

    fn test_sample_size<T: SampleType>()
    {
        use std::mem;

        let pa_size = super::get_sample_size::<T>().unwrap();
        let rs_size = mem::size_of::<T>();
        assert_eq!(rs_size, pa_size);
    }

    // In the FFI some assumptions are made as to how Some(p) and None are
    // represented when used as function pointers. This test asserts these
    // assumptions.
    #[test]
    fn option_pointer()
    {
        use std::{mem, ptr};
        use libc::c_void;

        unsafe
        {
            assert_eq!(mem::transmute::<Option<extern "C" fn()>, *const c_void>(Some(external_function as extern "C" fn())), external_function as *const c_void);
            assert_eq!(mem::transmute::<Option<extern "C" fn()>, *const c_void>(None), ptr::null());
        }
    }

    extern "C" fn external_function() {}
}
