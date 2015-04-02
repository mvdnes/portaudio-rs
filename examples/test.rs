extern crate portaudio;

use portaudio::{pa, stream, hostapi, device};

fn main()
{
    println!("version: {} \"{}\"", pa::version(), pa::version_text());
    println!("init: {:?}", pa::initialize());

    print_info();
    doit();

    println!("term: {:?}", pa::terminate());
}

fn print_info()
{
    match hostapi::get_count()
    {
        Ok(api_count) => {
            for i in (0 .. api_count)
            {
                let name = match hostapi::get_info(i)
                {
                    None => "???".to_string(),
                    Some(ha) => ha.name,
                };
                println!("api {}: {}", i, name);
            }
        },
        _ => {},
    }

    match device::get_count()
    {
        Ok(device_count) => {
            for i in (0 .. device_count)
            {
                let name = match device::get_info(i)
                {
                    None => "???".to_string(),
                    Some(d) => d.name,
                };
                println!("dev {}: {}", i, name);
            }
        },
        _ => {},
    }
}

fn doit()
{
    callback_demo();
    write_demo();
    mixed_demo();
}

fn callback_demo()
{
    let mut callback = |_input: &[f32], output: &mut [f32], _time: stream::StreamTimeInfo, _flags: stream::StreamCallbackFlags| -> stream::StreamCallbackResult
    {
        static mut lp: f32 = 0.0;
        static mut rp: f32 = 0.0;

        let mut left_phase = unsafe { lp };
        let mut right_phase = unsafe { rp };

        for i in (0 .. output.len() / 2)
        {
            output[i*2] = left_phase;
            output[i*2+1] = right_phase;

            left_phase += 0.01;
            if left_phase >= 1.0 { left_phase -= 2.0; }

            right_phase += 0.03;
            if right_phase >= 1.0 { right_phase -= 2.0; }
        }

        unsafe { lp = left_phase; }
        unsafe { rp = right_phase; }

        stream::StreamCallbackResult::Continue
    };

    let mut finished_callback = || println!("Finshed callback called");
    let mut stream = match stream::Stream::open_default(0, 2, 44100f64, stream::FRAMES_PER_BUFFER_UNSPECIFIED, Some(&mut callback))
    {
        Err(v) => { println!("Err({:?})", v); return },
        Ok(stream) => stream,
    };
    println!("finished_callback: {:?}", stream.set_finished_callback(&mut finished_callback));
    println!("start: {:?}", stream.start());
    std::thread::sleep_ms(1000);
    println!("stop: {:?}", stream.stop());

    println!("finished_callback: {:?}", stream.unset_finished_callback());
    println!("start: {:?}", stream.start());
    std::thread::sleep_ms(1000);
    println!("stop: {:?}", stream.stop());
}

fn write_demo()
{
    let stream = match stream::Stream::open_default(0, 2, 44100f64, stream::FRAMES_PER_BUFFER_UNSPECIFIED, None)
    {
        Err(v) => { println!("Err({:?})", v); return },
        Ok(stream) => stream,
    };

    println!("start: {:?}", stream.start());
    println!("write: {:?}", stream.write(&get_buffer(44100*3)));
    println!("stop: {:?}", stream.stop());
}

fn get_buffer(len: usize) -> Vec<f32>
{
    let mut left = 0.0f32;
    let mut right = 0.0f32;
    let mut result = Vec::with_capacity(len);
    for _ in (0 .. len / 2)
    {
        result.push(left);
        result.push(right);
        left += 0.03;
        right += 0.01;
        if left >= 1.0 { left -= 2.0; }
        if right >= 1.0 { right -= 2.0; }
    }
    result
}

fn mixed_demo()
{
    let in_idx = match device::get_default_input_index()
    {
        Ok(i) => i,
        Err(_) => return,
    };
    let out_idx = match device::get_default_output_index()
    {
        Ok(o) => o,
        Err(_) => return,
    };
    let in_lat = match device::get_info(in_idx)
    {
        None => return,
        Some(d) => d.default_low_input_latency,
    };
    let out_lat = match device::get_info(out_idx)
    {
        None => return,
        Some(d) => d.default_low_output_latency,
    };
    let input = stream::StreamParameters { device: in_idx, channel_count: 2, suggested_latency: in_lat, data: 0f32 };
    let output = stream::StreamParameters { device: out_idx, channel_count: 2, suggested_latency: out_lat, data: 0i8 };

    let supported = stream::is_format_supported(input, output, 44100f64);
    println!("support? {:?}", supported);
    if supported.is_err() { return }

    let stream = match stream::Stream::open(input, output, 44100f64, stream::FRAMES_PER_BUFFER_UNSPECIFIED, stream::StreamFlags::empty(), None)
    {
        Ok(s) => s,
        Err(o) => { println!("stream: Err({:?})", o); return },
    };

    let buffer = get_buffer(2*44100).into_iter().map(|v| (v * 127.0) as i8).collect::<Vec<i8>>();
    println!("start: {:?}", stream.start());
    println!("write: {:?}", stream.write(&buffer));
    println!("stop: {:?}", stream.stop());
}
