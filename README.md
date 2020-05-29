portaudio-rs
============

[![Build Status](https://travis-ci.org/mvdnes/portaudio-rs.svg?branch=master)](https://travis-ci.org/mvdnes/portaudio-rs)

[Documentation](https://mvdnes.github.io/rust-docs/portaudio-rs/portaudio_rs/index.html)

PortAudio bindings for Rust

See http://portaudio.com/

Example
-------

```rust
extern crate portaudio_rs as portaudio;

fn demo() -> portaudio::PaResult
{
    let stream = portaudio::stream::Stream::open_default(
                          0, // input channels
                          1, // output channels
                          44100.0, // sample rate
                          portaudio::stream::FRAMES_PER_BUFFER_UNSPECIFIED,
                          None // no callback
                 )?;

    stream.start()?;

    let mut phase = 0.0f32;
    let mut buffer = Vec::with_capacity(44100);
    for _i in 0..44100
    {
        // Small amplitude such that the test does not produce sound
        buffer.push(phase * 0.001);

        phase += 0.03;
        if phase > 1.0 { phase -= 2.0; }
    }

    stream.write(&buffer)?;

    Ok(())
}

fn main()
{
    portaudio::initialize().unwrap();
    println!("{:?}", demo());
    portaudio::terminate().unwrap();
}
```
