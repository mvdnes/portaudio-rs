extern crate portaudio;

use portaudio::{stream, hostapi, device};

fn main()
{
    println!("version: {} \"{}\"", portaudio::version(), portaudio::version_text());
    println!("init: {:?}", portaudio::initialize());

    print_info();
    doit();

    println!("term: {:?}", portaudio::terminate());
}

fn print_info()
{
    match hostapi::get_count()
    {
        Ok(api_count) => {
            for i in 0 .. api_count
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
            for i in 0 .. device_count
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
    let callback = Box::new(|_input: &[f32], output: &mut [f32], _time: stream::StreamTimeInfo, _flags: stream::StreamCallbackFlags| -> stream::StreamCallbackResult
    {
        static mut lp: f32 = 0.0;
        static mut rp: f32 = 0.0;

        let mut left_phase = unsafe { lp };
        let mut right_phase = unsafe { rp };

        for i in 0 .. output.len() / 2
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
    });

    let finished_callback = Box::new(|| println!("Finshed callback called"));
    let mut stream = match stream::Stream::open_default(0, 2, 44100f64, stream::FRAMES_PER_BUFFER_UNSPECIFIED, Some(callback))
    {
        Err(v) => { println!("Err({:?})", v); return },
        Ok(stream) => stream,
    };
    println!("finished_callback: {:?}", stream.set_finished_callback(finished_callback));
    println!("start: {:?}", stream.start());
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("stop: {:?}", stream.stop());

    println!("finished_callback: {:?}", stream.unset_finished_callback());
    println!("start: {:?}", stream.start());
    std::thread::sleep(std::time::Duration::from_secs(1));
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
    for _ in 0 .. len / 2
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
    let out_idx = match device::get_default_output_index()
    {
        Some(o) => o,
        None => return,
    };
    let out_lat = match device::get_info(out_idx)
    {
        None => return,
        Some(d) => d.default_low_output_latency,
    };
    let output = stream::StreamParameters { device: out_idx, channel_count: 2, suggested_latency: out_lat, data: 0i8 };

    let supported = stream::is_format_supported::<i8, _>(None, Some(output), 44100f64);
    println!("support? {:?}", supported);
    if supported.is_err() { return }

    let stream = match stream::Stream::<i8, _>::open(None, Some(output), 44100f64, stream::FRAMES_PER_BUFFER_UNSPECIFIED, stream::StreamFlags::empty(), None)
    {
        Ok(s) => s,
        Err(o) => { println!("stream: Err({:?})", o); return },
    };

    let buffer = get_buffer(2*44100).into_iter().map(|v| (v * 127.0) as i8).collect::<Vec<i8>>();
    println!("start: {:?}", stream.start());
    println!("write: {:?}", stream.write(&buffer));
    println!("stop: {:?}", stream.stop());
}
