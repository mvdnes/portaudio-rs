use ll;
use pa;
use pa::{PaError, PaResult};
use device::DeviceIndex;
use util::{to_pa_result, pa_time_to_duration};
use std::raw::Slice;
use std::mem;
use std::time::duration::Duration;

#[repr(u32)]
pub enum StreamCallbackResult
{
    Continue = ll::paContinue,
    Complete = ll::paComplete,
    Abort = ll::paAbort,
}

pub type StreamCallback<'a, T> = |input: &[T], output: &mut [T], timeinfo: StreamTimeInfo, StreamCallbackFlags|:'a -> StreamCallbackResult;
pub type StreamFinishedCallback<'a> = ||:'a;

struct StreamUserData<'a, T>
{
    num_input: uint,
    num_output: uint,
    callback: Option<StreamCallback<'a, T>>,
    finished_callback: Option<StreamFinishedCallback<'a>>,
}

pub struct StreamTimeInfo
{
    pub input_adc_time: Duration,
    pub current_time: Duration,
    pub output_dac_time: Duration,
}

bitflags!(
    flags StreamCallbackFlags: u64 {
        static inputUnderflow = 0x01,
        static inputOverflow = 0x02,
        static outputUnderflow = 0x04,
        static outputOverflow = 0x08,
        static primingOutput = 0x10
    }
)

bitflags!(
    flags StreamFlags: u64 {
        static ClipOff                               = 0x00000001,
        static DitherOff                             = 0x00000002,
        static NeverDropInput                        = 0x00000004,
        static PrimeOutputBuffersUsingStreamCallback = 0x00000008,
        static PlatformSpecific                      = 0xFFFF0000
    }
)

extern "C" fn stream_callback<T>(input: *const ::libc::c_void,
                                 output: *mut ::libc::c_void,
                                 frame_count: ::libc::c_ulong,
                                 time_info: *const ll::PaStreamCallbackTimeInfo,
                                 status_flags: ll::PaStreamCallbackFlags,
                                 user_data: *mut ::libc::c_void) -> ::libc::c_int
{
    let mut stream_data: Box<StreamUserData<T>> = unsafe { mem::transmute(user_data) };

    let input_buffer: &[T] = unsafe
    {
        mem::transmute(
            Slice { data: input as *const T, len: frame_count as uint * stream_data.num_input }
        )
    };
    let output_buffer: &mut [T] = unsafe
    {
        mem::transmute(
            Slice { data: output as *const T, len: frame_count as uint * stream_data.num_output }
        )
    };

    let flags = StreamCallbackFlags::from_bits_truncate(status_flags);

    let timeinfo = match unsafe { time_info.to_option() }
    {
        Some(ref info) => StreamTimeInfo { input_adc_time: pa_time_to_duration(info.inputBufferAdcTime),
                                           current_time: pa_time_to_duration(info.currentTime),
                                           output_dac_time: pa_time_to_duration(info.outputBufferDacTime) },
        None => StreamTimeInfo { input_adc_time: Duration::seconds(0),
                                 current_time: Duration::seconds(0),
                                 output_dac_time: Duration::seconds(0), },
    };

    let result = match stream_data.callback
    {
        Some(ref mut f) => (*f)(input_buffer, output_buffer, timeinfo, flags),
        None => Abort,
    };

    unsafe { mem::forget(stream_data); }

    result as i32
}

extern "C" fn stream_finished_callback<T>(user_data: *mut ::libc::c_void)
{
    let mut stream_data: Box<StreamUserData<T>> = unsafe { mem::transmute(user_data) };
    match stream_data.finished_callback
    {
        Some(ref mut f) => (*f)(),
        None => {},
    };

    unsafe { mem::forget(stream_data); }
}

trait SampleType
{
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

pub fn get_sample_size<T: SampleType>() -> Result<uint, PaError>
{
    match unsafe { ll::Pa_GetSampleSize(get_sample_format::<T>()) }
    {
        n if n >= 0 => Ok(n as uint),
        m => to_pa_result(m).map(|_| 0),
    }
}

pub struct Stream<'a, T>
{
    pa_stream: *mut ll::PaStream,
    inputs: uint,
    outputs: uint,
    user_data: Box<StreamUserData<'a, T>>,
}

impl<'a, T: SampleType> Stream<'a, T>
{
    pub fn open_default(num_input_channels: uint,
                        num_output_channels: uint,
                        sample_rate: f64,
                        frames_per_buffer: u64,
                        callback: Option<StreamCallback<'a, T>>)
                       -> Result<Stream<'a, T>, PaError>
    {
        unsafe
        {
            let callback_pointer = match callback
            {
                Some(_) => Some(stream_callback::<T>),
                None => None,
            };
            let userdata = box StreamUserData
            {
                num_input: num_input_channels,
                num_output: num_output_channels,
                callback: callback,
                finished_callback: None,
            };
            let mut pa_stream = ::std::ptr::mut_null();

            let ud_pointer: *mut ::libc::c_void = ::std::mem::transmute(userdata);
            let ud_pointer_2 = ud_pointer.clone();
            let code = ll::Pa_OpenDefaultStream(&mut pa_stream as *mut *mut ll::PaStream,
                                                num_input_channels as i32,
                                                num_output_channels as i32,
                                                get_sample_format::<T>(),
                                                sample_rate,
                                                frames_per_buffer,
                                                callback_pointer,
                                                ud_pointer);
            match to_pa_result(code)
            {
                Ok(()) => Ok(Stream { pa_stream: pa_stream,
                                      user_data: ::std::mem::transmute(ud_pointer_2),
                                      inputs: num_input_channels,
                                      outputs: num_output_channels,
                             }),
                Err(v) => Err(v),
            }
        }
    }

    pub fn open(input: StreamParameters<T>,
                output: StreamParameters<T>,
                sample_rate: f64,
                frames_per_buffer: u64,
                flags: StreamFlags,
                callback: Option<StreamCallback<'a, T>>)
               -> Result<Stream<'a, T>, PaError>
    {
        unsafe
        {
            let callback_pointer = match callback
            {
                Some(_) => Some(stream_callback::<T>),
                None => None,
            };

            let user_data = box StreamUserData
            {
                num_input: input.channel_count,
                num_output: output.channel_count,
                callback: callback,
                finished_callback: None,
            };

            let mut pa_stream = ::std::ptr::mut_null();
            let ud_pointer: *mut ::libc::c_void = mem::transmute(user_data);
            let ud_pointer_2 = ud_pointer.clone();

            let result = ll::Pa_OpenStream(&mut pa_stream,
                                                  &input.to_ll(),
                                                  &output.to_ll(),
                                                  sample_rate,
                                                  frames_per_buffer,
                                                  flags.bits,
                                                  callback_pointer,
                                                  ud_pointer);
            match to_pa_result(result)
            {
                Ok(()) => Ok(Stream { pa_stream: pa_stream,
                                      user_data: mem::transmute(ud_pointer_2),
                                      inputs: input.channel_count,
                                      outputs: output.channel_count,
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

    pub fn read_available(&self) -> Result<uint, PaError>
    {
        match unsafe { ll::Pa_GetStreamReadAvailable(self.pa_stream) }
        {
            n if n >= 0 => { Ok(n as uint) },
            n => to_pa_result(n as i32).map(|_| 0),
        }
    }

    pub fn write_available(&self) -> Result<uint, PaError>
    {
        match unsafe { ll::Pa_GetStreamWriteAvailable(self.pa_stream) }
        {
            n if n >= 0 => { Ok(n as uint) },
            n => to_pa_result(n as i32).map(|_| 0),
        }
    }

    pub fn write(&self, buffer: &[T]) -> PaResult
    {
        if self.outputs == 0 { return Err(pa::CanNotWriteToAnInputOnlyStream) }

        let pointer = buffer.as_ptr() as *const ::libc::c_void;
        let frames = (buffer.len() / self.outputs) as u64;

        to_pa_result(unsafe { ll::Pa_WriteStream(self.pa_stream, pointer, frames) })
    }

    pub fn read(&self, buffer: &mut [T]) -> PaResult
    {
        if self.inputs == 0 { return Err(pa::CanNotReadFromAnOutputOnlyStream) }

        let pointer = buffer.as_mut_ptr() as *mut ::libc::c_void;
        let frames = (buffer.len() / self.inputs) as u64;

        to_pa_result(unsafe { ll::Pa_ReadStream(self.pa_stream, pointer, frames) })
    }

    pub fn cpu_load(&self) -> f64
    {
        unsafe { ll::Pa_GetStreamCpuLoad(self.pa_stream) }
    }

    pub fn time(&self) -> Duration
    {
        let time = unsafe { ll::Pa_GetStreamTime(self.pa_stream) };
        pa_time_to_duration(time)
    }

    pub fn info(&self) -> Option<StreamInfo>
    {
        unsafe
        {
            ll::Pa_GetStreamInfo(self.pa_stream)
                .to_option()
                .map(|s| StreamInfo::from_ll(s))
        }
    }

    pub fn set_finished_callback(&mut self, finished_callback: StreamFinishedCallback<'a>) -> PaResult
    {
        self.user_data.finished_callback = Some(finished_callback);
        let callback_pointer = Some(stream_finished_callback::<T>);
        to_pa_result(unsafe { ll::Pa_SetStreamFinishedCallback(self.pa_stream, callback_pointer) })
    }

    pub fn unset_finished_callback(&mut self) -> PaResult
    {
        to_pa_result(unsafe { ll::Pa_SetStreamFinishedCallback(self.pa_stream, None) })
    }
}

#[unsafe_destructor]
impl<'a, T: SampleType> Drop for Stream<'a, T>
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

pub struct StreamParameters<T>
{
    pub device: DeviceIndex,
    pub channel_count: uint,
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
            suggestedLatency: self.suggested_latency.num_milliseconds() as f64 / 1000.0,
            hostApiSpecificStreamInfo: ::std::ptr::mut_null(),
        }
    }
}

pub fn is_format_supported<I: SampleType, O: SampleType>(input: StreamParameters<I>, output: StreamParameters<O>, sample_rate: f64) -> PaResult
{
    to_pa_result(unsafe { ll::Pa_IsFormatSupported(&input.to_ll(), &output.to_ll(), sample_rate) })
}

pub struct StreamInfo
{
    pub input_latency: Duration,
    pub output_latency: Duration,
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

        let pa_size = super::get_sample_size::<T>();
        let rs_size = Ok(mem::size_of::<T>());
        assert_eq!(rs_size, pa_size);
    }

    // In the FFI some assumptions are made as to how Some(p) and None are
    // represented when used as function pointers. This test asserts these
    // assumptions.
    #[test]
    fn option_pointer()
    {
        use std::{mem, ptr};
        use libc;

        unsafe
        {
            assert!   (mem::transmute::<Option<extern "C" fn()>, *const libc::c_void>(Some(external_function)) != ptr::null());
            assert_eq!(mem::transmute::<Option<extern "C" fn()>, *const libc::c_void>(None)                    ,  ptr::null());
        }
    }

    extern "C" fn external_function() {}
}
