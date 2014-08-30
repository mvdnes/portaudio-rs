use ll;
use pa;
use pa::{PaError, PaResult};
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

pub type StreamCallback<'a, T> = |input: &[T], output: &mut [T], timeinfo: StreamTimeInfo, StreamFlags|:'a -> StreamCallbackResult;

struct StreamUserData<'a, T>
{
    num_input: uint,
    num_output: uint,
    callback: StreamCallback<'a, T>,
}

pub struct StreamTimeInfo
{
    pub input_adc_time: Duration,
    pub current_time: Duration,
    pub output_dac_time: Duration,
}

bitflags!(
    flags StreamFlags: u64 {
        static inputUnderflow = 0x01,
        static inputOverflow = 0x02,
        static outputUnderflow = 0x04,
        static outputOverflow = 0x08,
        static primingOutput = 0x10
    }
)

extern "C" fn stream_callback<T>(input: *const ::libc::c_void,
                              output: *mut ::libc::c_void,
                              frame_count: ::libc::c_ulong,
                              time_info: *const ll::PaStreamCallbackTimeInfo,
                              status_flags: ll::PaStreamCallbackFlags,
                              user_data: *mut ::libc::c_void) -> ::libc::c_int
{
    let stream_data: Box<StreamUserData<T>> = unsafe { mem::transmute(user_data) };
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

    let flags = StreamFlags::from_bits_truncate(status_flags);

    let timeinfo = match unsafe { time_info.to_option() }
    {
        Some(ref info) => StreamTimeInfo { input_adc_time: pa_time_to_duration(info.inputBufferAdcTime),
                                           current_time: pa_time_to_duration(info.currentTime),
                                           output_dac_time: pa_time_to_duration(info.outputBufferDacTime) },
        None => StreamTimeInfo { input_adc_time: Duration::seconds(0),
                                 current_time: Duration::seconds(0),
                                 output_dac_time: Duration::seconds(0), },
    };

    let result = (stream_data.callback)(input_buffer, output_buffer, timeinfo, flags);

    unsafe { mem::forget(stream_data); }

    result as i32
}

trait PaType { fn as_sample_format(_: Option<Self>) -> u64; }
impl PaType for f32 { fn as_sample_format(_: Option<f32>) -> u64 { 0x00000001 } }
impl PaType for i32 { fn as_sample_format(_: Option<i32>) -> u64 { 0x00000002 } }
impl PaType for i16 { fn as_sample_format(_: Option<i16>) -> u64 { 0x00000008 } }
impl PaType for i8 { fn as_sample_format(_: Option<i8>) -> u64 { 0x00000010 } }
impl PaType for u8 { fn as_sample_format(_: Option<u8>) -> u64 { 0x00000020 } }

pub struct Stream<'a, T>
{
    pa_stream: *mut ll::PaStream,
    inputs: uint,
    outputs: uint,
    _callback: Box<StreamUserData<'a, T>>,
}

impl<'a, T: PaType> Stream<'a, T>
{
    pub fn open_default_stream(num_input_channels: uint,
                               num_output_channels: uint,
                               sample_rate: f64,
                               frames_per_buffer: u64,
                               callback: StreamCallback<'a, T>)
                              -> Result<Stream<'a, T>, PaError>
    {
        unsafe
        {
            let userdata = box StreamUserData
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
                Ok(()) => Ok(Stream { pa_stream: pa_stream,
                                      _callback: ::std::mem::transmute(ud_pointer_2),
                                      inputs: num_input_channels,
                                      outputs: num_output_channels,
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

        let Slice { data, len } = unsafe { mem::transmute::<&[T], Slice<T>>(buffer) };
        let buffer = data as *const ::libc::c_void;
        let frames = (len / self.outputs) as u64;

        to_pa_result(unsafe { ll::Pa_WriteStream(self.pa_stream, buffer, frames) })
    }

    pub fn read(&self, buffer: &mut [T]) -> PaResult
    {
        if self.inputs == 0 { return Err(pa::CanNotReadFromAnOutputOnlyStream) }

        let Slice { data, len } = unsafe { mem::transmute::<&mut [T], Slice<T>>(buffer) };
        let buffer = data as *mut ::libc::c_void;
        let frames = (len / self.inputs) as u64;

        to_pa_result(unsafe { ll::Pa_ReadStream(self.pa_stream, buffer, frames) })
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
}

#[unsafe_destructor]
impl<'a, T: PaType> Drop for Stream<'a, T>
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
