extern crate portaudio;

use portaudio::{pa, stream, hostapi, device};

fn main()
{
    println!("version: {} \"{}\"", pa::version(), pa::version_text());
    println!("init: {}", pa::initialize());

    print_info();
    doit();

    println!("term: {}", pa::terminate());
}

fn print_info()
{
    match hostapi::get_count()
    {
        Ok(api_count) => {
            for i in range(0, api_count)
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
            for i in range(0, device_count)
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
}

fn callback_demo()
{
    let callback = |_input: &[f32], output: &mut [f32], _time: stream::StreamTimeInfo, _flags: stream::StreamCallbackFlags| -> stream::StreamCallbackResult
    {
        static mut lp: f32 = 0.0;
        static mut rp: f32 = 0.0;

        let mut left_phase = unsafe { lp };
        let mut right_phase = unsafe { rp };

        for i in range(0, output.len() / 2)
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

        stream::Continue
    };

    let mut stream = match stream::Stream::open_default(0, 2, 44100f64, 0, Some(callback))
    {
        Err(v) => { println!("Err({})", v); return },
        Ok(stream) => stream,
    };
    println!("finished_callback: {}", stream.set_finished_callback(|| println!("Finished callback called")));
    println!("start: {}", stream.start());
    std::io::timer::sleep(std::time::duration::Duration::seconds(1));
    println!("stop: {}", stream.stop());

    println!("finished_callback: {}", stream.unset_finished_callback());
    println!("start: {}", stream.start());
    std::io::timer::sleep(std::time::duration::Duration::seconds(1));
    println!("stop: {}", stream.stop());
}

fn write_demo()
{
    let stream = match stream::Stream::open_default(0, 2, 44100f64, 0, None::<stream::StreamCallback<f32>>)
    {
        Err(v) => { println!("Err({})", v); return },
        Ok(stream) => stream,
    };

    println!("start: {}", stream.start());
    println!("write: {}", stream.write(get_buffer(44100*3).as_slice()));
    println!("stop: {}", stream.stop());
}

fn get_buffer(len: uint) -> Vec<f32>
{
    let mut left = 0.0f32;
    let mut right = 0.0f32;
    let mut result = Vec::with_capacity(len);
    for i in range(0, len / 2)
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
