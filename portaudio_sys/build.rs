extern crate pkg_config;

fn main() {
    match pkg_config::find_library("portaudio-2.0")
    {
        Ok(..) => {},
        Err(e) => panic!("{}", e),
    }
}

