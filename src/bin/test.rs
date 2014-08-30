extern crate portaudio;

use portaudio::pa;
use portaudio::stream;

fn main()
{
    println!("version: {} \"{}\"", pa::version(), pa::version_text());
    println!("init: {}", pa::initialize());
 
    doit();

    println!("term: {}", pa::terminate());
}

fn doit()
{
    let callback = |_input: &[f32], output: &mut [f32], _time: stream::StreamTimeInfo, _flags: stream::StreamFlags| -> stream::StreamCallbackResult
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

            right_phase += 0.3;
            if right_phase >= 1.0 { right_phase -= 2.0; }
        }

        unsafe { lp = left_phase; }
        unsafe { rp = right_phase; }

        stream::Continue
    };

    let stream = match stream::Stream::open_default_stream(0, 2, 44100f64, 0, callback)
    {
        Err(v) => { println!("Err({})", v); return },
        Ok(stream) => stream,
    };

    println!("start: {}", stream.start());
    std::io::timer::sleep(std::time::duration::Duration::seconds(1));
    println!("stopped? {}", stream.is_stopped());
    println!("active? {}", stream.is_active());
    std::io::timer::sleep(std::time::duration::Duration::seconds(1));
    println!("stop: {}", stream.stop());
}
