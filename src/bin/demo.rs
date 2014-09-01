extern crate portaudio;

static SECONDS: uint = 1;

fn main()
{
    portaudio::pa::initialize().unwrap();
    println!("{}", demo());
    portaudio::pa::terminate().unwrap();
}

fn demo() -> portaudio::pa::PaResult
{
    let stream = try!(portaudio::stream::Stream::open_default(0, 1, 44100.0, 0, None));

    try!(stream.start());

    let mut phase = 0.0f32;
    let mut buffer = Vec::with_capacity(44100 * SECONDS);
    for _i in range(0u, 44100 * SECONDS)
    {
        buffer.push(phase);

        phase += 0.007;
        if phase > 1.0 { phase -= 2.0; }
    }

    let mut timer = match std::io::timer::Timer::new()
    {
        Err(e) => { fail!("{}", e); },
        Ok(t) => t,
    };
    let waiter = timer.oneshot(std::time::duration::Duration::seconds(SECONDS as i64));

    try!(stream.write(buffer.as_slice()));

    waiter.recv();

    Ok(())
}
