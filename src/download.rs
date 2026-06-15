use std::env;
use std::fs;


pub const ZAPRET_REPO: &str = "bol-van/zapret";
pub const ZAPRET_REC_VER: &str = "v72.9";
pub const STRAT_REC_VER: &str = "ef19845a801e4e743f7bdfdbd58f9745c6adbd60";

const STRAT_REPO_ZIP: &str = "https://github.com/Flowseal/zapret-discord-youtube/archive/refs/heads/main.zip";

fn detect_platform_dir() -> Result<&'static str, String> {
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

pub fn download_nfqws(version: &str) -> Result<(), String> {
    if version == "skip" {
        return Ok(());
    }

    println!("{}", rust_i18n::t!("msg_chk_nfqws"));
    
    let bin_dir = crate::config::get_cache_dir().join("bin");
    let _ = fs::create_dir_all(&bin_dir);

    let platform = detect_platform_dir()?;
    println!("{}{}", rust_i18n::t!("msg_det_plat"), platform);

    let tag = if version == "latest" {
        println!("{}", rust_i18n::t!("msg_fetch_rel"));
        let latest_url = format!("https://api.github.com/repos/{}/releases/latest", ZAPRET_REPO);
        let req = ureq::get(&latest_url)
            .set("User-Agent", "zapret-rust")
            .call()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_fetch_rel"), e))?;
        
        let json_str = req.into_string().unwrap_or_else(|_| "{}".to_string());
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        parsed.get("tag_name")
            .and_then(|t| t.as_str())
            .unwrap_or(ZAPRET_REC_VER)
            .to_string()
    } else {
        version.to_string()
    };

    println!("{}{}", rust_i18n::t!("msg_using_tag"), tag);
    let archive = format!("zapret-{}.tar.gz", tag);
    let url = format!("https://github.com/{}/releases/download/{}/{}", ZAPRET_REPO, tag, archive);
    
    // Use local temp directory inside cache_dir to avoid Windows Defender blocks
    let tmp_dir = crate::config::get_cache_dir().join(".tmp_zapret_download");
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::create_dir_all(&tmp_dir);
    let tmp_archive = tmp_dir.join(&archive);

    println!("{}{}", rust_i18n::t!("msg_dl_arc"), url);
    let mut response = ureq::get(&url).call().map_err(|e| format!("{}{}", rust_i18n::t!("err_dl_arc"), e))?.into_reader();
    let mut file = fs::File::create(&tmp_archive).map_err(|e| format!("{}{}", rust_i18n::t!("err_create_file"), e))?;
    std::io::copy(&mut response, &mut file).map_err(|e| format!("{}{}", rust_i18n::t!("err_write_arc"), e))?;

    println!("{}", rust_i18n::t!("msg_ext_arc"));
    let tar_gz = fs::File::open(&tmp_archive).map_err(|e| format!("{}{}", rust_i18n::t!("err_open_arc"), e))?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(&tmp_dir).map_err(|e| format!("{}{}", rust_i18n::t!("err_unpack_tar"), e))?;

    let expected_bin_path = tmp_dir.join(format!("zapret-{}", tag)).join("binaries").join(platform);

    if expected_bin_path.exists() {
        if env::consts::OS == "windows" {
            // For Windows, we need winws.exe, WinDivert.dll, WinDivert64.sys, cygwin1.dll, etc.
            // Copy everything in the platform folder.
            for entry in fs::read_dir(&expected_bin_path).map_err(|e| format!("{}{}", rust_i18n::t!("err_read_bin"), e))? {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    fs::copy(entry.path(), bin_dir.join(&file_name))
                        .map_err(|e| format!("{}{:?}: {}", rust_i18n::t!("err_copy_file"), file_name, e))?;
                }
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
                if let Ok(output) = std::process::Command::new("setcap")
                    .args(["cap_net_admin+ep", bin_dir.join(bin_name).to_str().unwrap()])
                    .output()
                {
                    if output.status.success() {
                        println!("{}", rust_i18n::t!("msg_setcap_ok"));
                    }
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

    let target_dir = crate::config::get_cache_dir().join("zapret-discord-youtube-linux");
    
    let url = if version == "latest" {
        STRAT_REPO_ZIP.to_string()
    } else if version == "recommended" {
        format!("https://github.com/Flowseal/zapret-discord-youtube/archive/{}.zip", STRAT_REC_VER)
    } else {
        format!("https://github.com/Flowseal/zapret-discord-youtube/archive/{}.zip", version)
    };
    
    println!("{}", rust_i18n::t!("msg_dl_strat"));
    let req = ureq::get(&url).call().map_err(|e| format!("{}{}", rust_i18n::t!("err_dl_strat_zip"), e))?;
    let mut body = req.into_reader();
    
    let tmp_zip = crate::config::get_cache_dir().join(".tmp_strategies.zip");
    let mut file = fs::File::create(&tmp_zip).map_err(|e| format!("{}{}", rust_i18n::t!("err_create_tmp_zip"), e))?;
    std::io::copy(&mut body, &mut file).map_err(|e| format!("{}{}", rust_i18n::t!("err_write_zip"), e))?;

    println!("{}", rust_i18n::t!("msg_ext_strat"));
    let zip_file = fs::File::open(&tmp_zip).map_err(|e| format!("{}{}", rust_i18n::t!("err_open_zip"), e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("{}{}", rust_i18n::t!("err_read_zip"), e))?;

    let _ = fs::create_dir_all(&target_dir);
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => {
                let mut components = path.components();
                components.next(); // Skip root folder
                components.as_path().to_owned()
            },
            None => continue,
        };

        if outpath.as_os_str().is_empty() {
            continue;
        }

        let full_path = target_dir.join(outpath);
        
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&full_path).map_err(|e| format!("{}{}", rust_i18n::t!("err_mkdir"), e))?;
        } else {
            if let Some(p) = full_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("{}{}", rust_i18n::t!("err_mkdir"), e))?;
                }
            }
            let mut outfile = fs::File::create(&full_path).map_err(|e| format!("{}{}", rust_i18n::t!("err_extract"), e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("{}{}", rust_i18n::t!("err_copy_content"), e))?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&full_path, fs::Permissions::from_mode(mode)).ok();
                }
            }
        }
    }

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


pub fn check_nfqws_installed() -> bool {
    let bin_dir = crate::config::get_cache_dir().join("bin");
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    bin_dir.join(bin_name).exists()
}

pub fn check_strategies_installed() -> bool {
    let repo_dir = crate::config::get_cache_dir().join("zapret-discord-youtube-linux");
    if !repo_dir.exists() {
        return false;
    }
    let mut has_bat = false;
    if let Ok(entries) = std::fs::read_dir(&repo_dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bat") {
                    has_bat = true;
                    break;
                }
            }
        }
    }
    if !has_bat {
        if let Ok(entries) = std::fs::read_dir(repo_dir.join("custom-strategies")) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".bat") {
                        has_bat = true;
                        break;
                    }
                }
            }
        }
    }
    has_bat
}

pub fn fetch_repo_tags(repo: &str) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/repos/{}/tags", repo);
    let req = ureq::get(&url)
        .set("User-Agent", "zapret-rust-tui")
        .call()
        .map_err(|e| format!("{}{}: {}", rust_i18n::t!("err_fetch_tags"), repo, e))?;
    
    let json_str = req.into_string().map_err(|e| format!("{}{}", rust_i18n::t!("err_read_tags"), e))?;
    let tags_json: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| format!("{}{}", rust_i18n::t!("err_parse_tags"), e))?;
    let mut tags = Vec::new();
    
    if let Some(arr) = tags_json.as_array() {
        for item in arr {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                tags.push(name.to_string());
            }
        }
    }
    
    Ok(tags)
}
