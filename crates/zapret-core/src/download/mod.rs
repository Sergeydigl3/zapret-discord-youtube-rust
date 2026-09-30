//! Downloading and installing the zapret runtime binary and the strategy
//! repository.
//!
//! The zone is split by concern: `platform_matrix` maps the running OS and
//! architecture onto a release directory, `github` talks to the GitHub API,
//! `archive` unpacks what was downloaded, and `state` answers whether the
//! payloads are already on disk. This module keeps the two top-level
//! operations and the version constants the rest of the crate refers to.

mod archive;
mod github;
mod platform_matrix;
mod state;

use std::env;
use std::fs;

use platform_matrix::detect_platform_dir;

pub use github::fetch_repo_tags;
pub use state::{check_nfqws_installed, check_strategies_installed};

pub const ZAPRET_REPO: &str = "bol-van/zapret";
pub const ZAPRET_REC_VER: &str = "v72.13";
pub const STRAT_REC_VER: &str = "9503dc045133000af8075e066f09bb469008e530";

const STRAT_REPO_ZIP: &str = "https://github.com/Flowseal/zapret-discord-youtube/archive/refs/heads/main.zip";

pub fn download_nfqws(version: &str) -> Result<(), String> {
    if version == "skip" {
        return Ok(());
    }

    println!("{}", rust_i18n::t!("msg_chk_nfqws"));

    let bin_dir = crate::paths::bin_runtime_dir();
    let _ = fs::create_dir_all(&bin_dir);

    let platform = detect_platform_dir()?;
    println!("{}{}", rust_i18n::t!("msg_det_plat"), platform);

    let tag = github::resolve_tag(version)?;
    println!("{}{}", rust_i18n::t!("msg_using_tag"), tag);
    let archive = format!("zapret-{}.tar.gz", tag);
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        ZAPRET_REPO, tag, archive
    );

    // Use local temp directory inside cache_dir to avoid Windows Defender blocks
    let tmp_dir = crate::paths::cache_dir().join(".tmp_zapret_download");
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::create_dir_all(&tmp_dir);
    let tmp_archive = tmp_dir.join(&archive);

    println!("{}{}", rust_i18n::t!("msg_dl_arc"), url);
    let mut response = ureq::get(&url)
        .call()
        .map_err(|e| format!("{}{}", rust_i18n::t!("err_dl_arc"), e))?
        .into_reader();
    let mut file = fs::File::create(&tmp_archive).map_err(|e| format!("{}{}", rust_i18n::t!("err_create_file"), e))?;
    std::io::copy(&mut response, &mut file).map_err(|e| format!("{}{}", rust_i18n::t!("err_write_arc"), e))?;

    println!("{}", rust_i18n::t!("msg_ext_arc"));
    archive::unpack_tar_gz(&tmp_archive, &tmp_dir)?;

    let expected_bin_path = tmp_dir.join(format!("zapret-{}", tag)).join("binaries").join(platform);

    if expected_bin_path.exists() {
        if env::consts::OS == "windows" {
            // For Windows, we need winws.exe, WinDivert.dll, WinDivert64.sys, cygwin1.dll, etc.
            // Copy everything in the platform folder.
            for entry in fs::read_dir(&expected_bin_path)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_read_bin"), e))?
                .flatten()
            {
                let file_name = entry.file_name();
                fs::copy(entry.path(), bin_dir.join(&file_name))
                    .map_err(|e| format!("{}{:?}: {}", rust_i18n::t!("err_copy_file"), file_name, e))?;
            }
            println!("{}", rust_i18n::t!("msg_inst_win_ok"));
        } else {
            let bin_name = "nfqws";
            let bin_file = expected_bin_path.join(bin_name);
            if bin_file.exists() {
                let dest = bin_dir.join(bin_name);
                let _ = fs::remove_file(&dest);
                fs::copy(&bin_file, &dest).map_err(|e| format!("{}{}", rust_i18n::t!("err_copy_bin"), e))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(mut perms) = fs::metadata(bin_dir.join(bin_name)).map(|m| m.permissions()) {
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(bin_dir.join(bin_name), perms);
                    }
                }
                println!("{}{}", rust_i18n::t!("msg_inst_nux_ok"), bin_name);

                // Set CAP_NET_ADMIN so nfqws can use nfqueue without full root
                if crate::process::try_set_cap(&bin_dir.join(bin_name)) == Some(true) {
                    println!("{}", rust_i18n::t!("msg_setcap_ok"));
                }
            } else {
                return Err(format!("Could not find {} in {:?}", bin_name, expected_bin_path));
            }
        }
    } else {
        return Err(format!("Could not find binaries path {:?}", expected_bin_path));
    }

    let _ = fs::remove_dir_all(tmp_dir);
    Ok(())
}

pub fn download_strategies(version: &str) -> Result<(), String> {
    if version == "skip" {
        return Ok(());
    }

    let target_dir = crate::paths::cache_dir().join(crate::paths::REPO_DIR_NAME);

    let url = if version == "latest" {
        STRAT_REPO_ZIP.to_string()
    } else if version == "recommended" {
        format!(
            "https://github.com/Flowseal/zapret-discord-youtube/archive/{}.zip",
            STRAT_REC_VER
        )
    } else {
        format!(
            "https://github.com/Flowseal/zapret-discord-youtube/archive/{}.zip",
            version
        )
    };

    println!("{}", rust_i18n::t!("msg_dl_strat"));
    let req = ureq::get(&url)
        .call()
        .map_err(|e| format!("{}{}", rust_i18n::t!("err_dl_strat_zip"), e))?;
    let mut body = req.into_reader();

    let tmp_zip = crate::paths::cache_dir().join(".tmp_strategies.zip");
    let mut file = fs::File::create(&tmp_zip).map_err(|e| format!("{}{}", rust_i18n::t!("err_create_tmp_zip"), e))?;
    std::io::copy(&mut body, &mut file).map_err(|e| format!("{}{}", rust_i18n::t!("err_write_zip"), e))?;

    println!("{}", rust_i18n::t!("msg_ext_strat"));
    archive::unpack_zip(&tmp_zip, &target_dir)?;

    let _ = fs::remove_file(tmp_zip);
    println!("{}", rust_i18n::t!("msg_strat_ok"));
    Ok(())
}

pub fn install_dependencies(nfqws_ver: &str, strat_ver: &str) -> Result<(), String> {
    println!("=======================================================");
    println!("{}", rust_i18n::t!("msg_inst_deps"));
    println!("=======================================================");

    let mut errors = Vec::new();
    if nfqws_ver != "skip" {
        if let Err(e) = download_nfqws(nfqws_ver) {
            errors.push(format!("nfqws error: {}", e));
        }
    }
    if strat_ver != "skip" {
        if let Err(e) = download_strategies(strat_ver) {
            errors.push(format!("strategies error: {}", e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
