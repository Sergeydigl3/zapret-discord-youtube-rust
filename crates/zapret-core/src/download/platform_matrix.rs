//! OS and CPU architecture to release-directory mapping.
//!
//! The zapret releases are published as `zapret-<tag>/binaries/<platform>`
//! archives, so the running platform has to be turned into one of the
//! directory names before the download can be unpacked and searched.

use std::env;

pub(crate) fn detect_platform_dir() -> Result<&'static str, String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match os {
        "linux" => match arch {
            "x86_64" => Ok("linux-x86_64"),
            "x86" => Ok("linux-x86"),
            "aarch64" => Ok("linux-arm64"),
            "arm" => Ok("linux-arm"),
            "mips64" => Ok("linux-mips64"),
            "mips" => Ok("linux-mips"),
            _ => Err(format!("Unsupported Linux architecture: {}", arch)),
        },
        "macos" => Ok("mac64"),
        "freebsd" => Ok("freebsd-x86_64"),
        "windows" => match arch {
            "x86_64" => Ok("windows-x86_64"),
            "x86" => Ok("windows-x86"),
            _ => Err(format!("Unsupported Windows architecture: {}", arch)),
        },
        _ => Err(format!("Unsupported OS: {}", os)),
    }
}
