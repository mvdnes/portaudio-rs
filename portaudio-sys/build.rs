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
mod windows_build {
    use std;
    use std::path::Path;
    use std::process::Command;

    extern crate cmake;

    pub fn download_sources() {
        let mut command = Command::new("cmake");

        command.arg("-P");
        command.arg("download.cmake");

        match command.status() {
            Ok(status) =>
                if !status.success() {
                    panic!("Failed to execute command: {:?}", command)
                },
            Err(error) =>
                panic!("Failed to execute command: {:?}\n{}", command, error)
        }
    }

    pub fn build_sources() {
        let out_dir_env = std::env::var("OUT_DIR").unwrap();
        let out_dir = Path::new(&out_dir_env);

        let source_path = out_dir.join("portaudio");

        cmake::Config::new(source_path)
            .define("CMAKE_ARCHIVE_OUTPUT_DIRECTORY_DEBUG", out_dir)
            .define("CMAKE_ARCHIVE_OUTPUT_DIRECTORY_RELEASE", out_dir)
            .out_dir(out_dir)
            .build_target("portaudio_static")
            .build();

        std::fs::rename(
            out_dir.join(platform_specific_library_name()),
            out_dir.join("portaudio.lib")).unwrap();

        println!(
            "cargo:rustc-link-search=native={}", out_dir.to_str().unwrap());
    }

    #[cfg(target_arch = "x86")]
    fn platform_specific_library_name() -> &'static str {
        "portaudio_static_x86.lib"
    }

    #[cfg(target_arch = "x86_64")]
    fn platform_specific_library_name() -> &'static str {
        "portaudio_static_x64.lib"
    }
}

#[cfg(windows)]
fn main() {
    windows_build::download_sources();
    windows_build::build_sources();
}
