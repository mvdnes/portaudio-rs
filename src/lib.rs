#![crate_type = "lib"]
#![crate_name = "portaudio_rs"]
#![warn(missing_docs)]

//! PortAudio bindings for Rust
//!
//! # Example
//!
//! ```
//! fn demo() -> portaudio_rs::PaResult
//! {
//!     let stream = try!(portaudio_rs::stream::Stream::open_default(
//!                           0, // input channels
//!                           1, // output channels
//!                           44100.0, // sample rate
//!                           portaudio_rs::stream::FRAMES_PER_BUFFER_UNSPECIFIED,
//!                           None // no callback
//!                      ));
//!
//!     try!(stream.start());
//!
//!     let mut phase = 0.0f32;
//!     let mut buffer = Vec::with_capacity(44100);
//!     for _i in (0..44100)
//!     {
//!         // Small amplitude such that the test does not produce sound
//!         buffer.push(phase * 0.001);
//!
//!         phase += 0.03;
//!         if phase > 1.0 { phase -= 2.0; }
//!     }
//!
//!     try!(stream.write(&buffer));
//!
//!     Ok(())
//! }
//!
//! portaudio_rs::initialize().unwrap();
//! println!("{:?}", demo());
//! portaudio_rs::terminate().unwrap();
//! ```

extern crate libc;
#[macro_use] extern crate bitflags;
extern crate portaudio_sys as ll;

pub use pa::{PaError, PaResult, initialize, terminate, version, version_text};

pub mod stream;
mod pa;
pub mod hostapi;
pub mod device;

mod util;
