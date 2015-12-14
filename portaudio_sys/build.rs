extern crate pkg_config;

#[cfg(not(windows))]
fn main() {
    match pkg_config::find_library("portaudio-2.0")
    {
        Ok(..) => {},
        Err(e) => panic!("{}", e),
    }
}

#[cfg(windows)]
fn main() {
    // Assume the library is in the correct path
}
