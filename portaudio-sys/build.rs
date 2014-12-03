extern crate "pkg-config" as pkg_config;

fn main() {
    let opts = pkg_config::default_options("portaudio-2.0");
    match pkg_config::find_library_opts("portaudio-2.0", &opts)
    {
        Ok(..) => {},
        Err(e) => panic!("{}", e),
    }
}

