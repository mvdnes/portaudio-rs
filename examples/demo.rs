extern crate portaudio;

static SECONDS: usize = 1;

fn main()
{
    portaudio::initialize().unwrap();
    print_devs();
    println!("{:?}", demo());
    portaudio::terminate().unwrap();
}

fn print_devs()
{
    for i in 0 .. portaudio::device::get_count().unwrap()
    {
        match portaudio::device::get_info(i)
        {
            None => {},
            Some(info) => println!("{}: {}", i, info.name),
        }
    }
}

fn demo() -> portaudio::PaResult
{
    let stream = try!(portaudio::stream::Stream::open_default(1, 1, 44100.0, portaudio::stream::FRAMES_PER_BUFFER_UNSPECIFIED, None));

    try!(stream.start());

    let input = try!(stream.read(44100));

    let mut phase = 0.0f32;
    let mut buffer = Vec::with_capacity(44100 * SECONDS);
    for _i in 0 .. 44100 * SECONDS
    {
        buffer.push(phase);

        phase += 0.007;
        if phase > 1.0 { phase -= 2.0; }
    }

    let waiter = std::thread::spawn(move|| {
        std::thread::sleep(std::time::Duration::from_secs(SECONDS as u64));
    });

    match stream.write(&*buffer)
    {
        Err(e) => { println!("write 1: Err({:?})", e); },
        Ok(()) => {},
    }

    match stream.write(&*input)
    {
        Err(e) => { println!("write 2: Err({:?})", e); },
        Ok(()) => {},
    }

    let _ = waiter.join();

    Ok(())
}
