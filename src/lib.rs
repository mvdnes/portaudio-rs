#![crate_type = "lib"]
#![crate_name = "portaudio"]
#![feature(phase, unsafe_destructor)]
#![warn(missing_docs)]

//! PortAudio bindings for Rust
//!
//! # Example
//!
//! ```
//! fn demo() -> portaudio::PaResult
//! {
//!     let stream = try!(portaudio::stream::Stream::open_default(
//!                           0, // input channels
//!                           1, // output channels
//!                           44100.0, // sample rate
//!                           portaudio::stream::FRAMES_PER_BUFFER_UNSPECIFIED,
//!                           None // no callback
//!                      ));
//!
//!     try!(stream.start());
//!
//!     let mut phase = 0.0f32;
//!     let mut buffer = Vec::with_capacity(44100);
//!     for _i in range(0u, 44100)
//!     {
//!         // Small amplitude such that the test does not produce sound
//!         buffer.push(phase * 0.001);
//!
//!         phase += 0.03;
//!         if phase > 1.0 { phase -= 2.0; }
//!     }
//!
//!     try!(stream.write(buffer.as_slice()));
//!
//!     Ok(())
//! }
//!
//! portaudio::initialize().unwrap();
//! println!("{}", demo());
//! portaudio::terminate().unwrap();
//! ```

extern crate libc;
#[phase(plugin, link)] extern crate log;
extern crate "portaudio-sys" as ll;

pub use pa::{PaError, PaResult, initialize, terminate, version, version_text};

pub mod stream;
pub mod pa;
pub mod hostapi;
pub mod device;

mod util;
