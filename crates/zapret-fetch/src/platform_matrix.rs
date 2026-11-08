//! OS and CPU architecture to release-directory mapping.
//!
//! The zapret2 releases are published as `zapret2-<tag>/binaries/<platform>`
//! archives, so the running platform has to be turned into one of the
//! directory names before the download can be unpacked and searched.

use std::env;

pub(crate) fn detect_platform_dir() -> Result<&'static str, String> {
    detect_platform_dir_for(env::consts::OS, env::consts::ARCH)
}

/// The one definition of the mapping; `detect_platform_dir` feeds it the
/// running platform so tests can feed it any other.
fn detect_platform_dir_for(os: &str, arch: &str) -> Result<&'static str, String> {
    match os {
        "linux" => match arch {
            "x86_64" => Ok("linux-x86_64"),
            "x86" => Ok("linux-x86"),
            "aarch64" => Ok("linux-arm64"),
            "arm" => Ok("linux-arm"),
            "mips64" => Ok("linux-mips64"),
            "mips" => Ok("linux-mips"),
            "riscv64gc" => Ok("linux-riscv64"),
            _ => Err(format!("Unsupported Linux architecture: {}", arch)),
        },
        // zapret2 dropped macOS builds; zapret 1 was the last to ship mac64.
        "macos" => Err("zapret2 does not support macOS".to_string()),
        "freebsd" => Ok("freebsd-x86_64"),
        "windows" => match arch {
            "x86_64" => Ok("windows-x86_64"),
            "x86" => Ok("windows-x86"),
            _ => Err(format!("Unsupported Windows architecture: {}", arch)),
        },
        _ => Err(format!("Unsupported OS: {}", os)),
    }
}

#[cfg(test)]
mod tests {
    use super::detect_platform_dir_for;

    #[test]
    fn macos_is_rejected() {
        // macOS has no zapret2 release directory; the error must say so rather
        // than pointing at a directory that will never exist in the archive.
        assert!(detect_platform_dir_for("macos", "x86_64").is_err());
    }

    #[test]
    fn names_match_the_release_directories() {
        assert_eq!(detect_platform_dir_for("linux", "x86_64").unwrap(), "linux-x86_64");
        assert_eq!(detect_platform_dir_for("linux", "riscv64gc").unwrap(), "linux-riscv64");
        assert_eq!(detect_platform_dir_for("linux", "aarch64").unwrap(), "linux-arm64");
        assert_eq!(detect_platform_dir_for("windows", "x86_64").unwrap(), "windows-x86_64");
        assert_eq!(detect_platform_dir_for("freebsd", "x86_64").unwrap(), "freebsd-x86_64");
        assert!(detect_platform_dir_for("linux", "sparc64").is_err());
    }
}
