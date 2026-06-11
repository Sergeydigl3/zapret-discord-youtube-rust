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

    println!("\n[nfqws] Checking binary...");
    
    let bin_dir = crate::config::get_cache_dir().join("bin");
    let _ = fs::create_dir_all(&bin_dir);

    let platform = detect_platform_dir()?;
    println!("[nfqws] Detected platform: {}", platform);

    let tag = if version == "latest" {
        println!("[nfqws] Fetching latest release tag...");
        let latest_url = format!("https://api.github.com/repos/{}/releases/latest", ZAPRET_REPO);
        let req = ureq::get(&latest_url)
            .set("User-Agent", "zapret-rust")
            .call()
            .map_err(|e| format!("Failed to fetch release info: {}", e))?;
        
        let json_str = req.into_string().unwrap_or_else(|_| "{}".to_string());
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        parsed.get("tag_name")
            .and_then(|t| t.as_str())
            .unwrap_or(ZAPRET_REC_VER)
            .to_string()
    } else {
        version.to_string()
    };

    println!("[nfqws] Using tag: {}", tag);
    let archive = format!("zapret-{}.tar.gz", tag);
    let url = format!("https://github.com/{}/releases/download/{}/{}", ZAPRET_REPO, tag, archive);
    
    // Use local temp directory inside cache_dir to avoid Windows Defender blocks
    let tmp_dir = crate::config::get_cache_dir().join(".tmp_zapret_download");
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::create_dir_all(&tmp_dir);
    let tmp_archive = tmp_dir.join(&archive);

    println!("[nfqws] Downloading archive: {}", url);
    let mut response = ureq::get(&url).call().map_err(|e| format!("Ошибка скачивания архива: {}", e))?.into_reader();
    let mut file = fs::File::create(&tmp_archive).map_err(|e| format!("Ошибка создания файла: {}", e))?;
    std::io::copy(&mut response, &mut file).map_err(|e| format!("Ошибка записи архива: {}", e))?;

    println!("[nfqws] Extracting archive...");
    let tar_gz = fs::File::open(&tmp_archive).map_err(|e| format!("Ошибка открытия архива: {}", e))?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(&tmp_dir).map_err(|e| format!("Ошибка распаковки tar: {}", e))?;

    let expected_bin_path = tmp_dir.join(format!("zapret-{}", tag)).join("binaries").join(platform);

    if expected_bin_path.exists() {
        if env::consts::OS == "windows" {
            // For Windows, we need winws.exe, WinDivert.dll, WinDivert64.sys, cygwin1.dll, etc.
            // Copy everything in the platform folder.
            for entry in fs::read_dir(&expected_bin_path).map_err(|e| format!("Failed to read bin dir: {}", e))? {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    fs::copy(entry.path(), bin_dir.join(&file_name))
                        .map_err(|e| format!("Failed to copy {:?}: {}", file_name, e))?;
                }
            }
            println!("[nfqws] Successfully installed winws and dependencies to bin/");
        } else {
            let bin_name = "nfqws";
            let bin_file = expected_bin_path.join(bin_name);
            if bin_file.exists() {
                let dest = bin_dir.join(bin_name);
                let _ = fs::remove_file(&dest);
                fs::copy(&bin_file, &dest).map_err(|e| format!("Failed to copy binary: {}", e))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(mut perms) = fs::metadata(bin_dir.join(bin_name)).map(|m| m.permissions()) {
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(bin_dir.join(bin_name), perms);
                    }
                }
                println!("[nfqws] Successfully installed nfqws to bin/{}", bin_name);

                // Set CAP_NET_ADMIN so nfqws can use nfqueue without full root
                if let Ok(output) = std::process::Command::new("setcap")
                    .args(["cap_net_admin+ep", bin_dir.join(bin_name).to_str().unwrap()])
                    .output()
                {
                    if output.status.success() {
                        println!("[nfqws] cap_net_admin+ep set on nfqws");
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
    
    println!("\n[strategies] Downloading via HTTP...");
    let req = ureq::get(&url).call().map_err(|e| format!("Failed to download strategies zip: {}", e))?;
    let mut body = req.into_reader();
    
    let tmp_zip = crate::config::get_cache_dir().join(".tmp_strategies.zip");
    let mut file = fs::File::create(&tmp_zip).map_err(|e| format!("Failed to create temp zip: {}", e))?;
    std::io::copy(&mut body, &mut file).map_err(|e| format!("Failed to write zip: {}", e))?;

    println!("[strategies] Extracting strategies...");
    let zip_file = fs::File::open(&tmp_zip).map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("Failed to read zip: {}", e))?;

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
            fs::create_dir_all(&full_path).map_err(|e| format!("Failed to create dir: {}", e))?;
        } else {
            if let Some(p) = full_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| format!("Failed to create dir: {}", e))?;
                }
            }
            let mut outfile = fs::File::create(&full_path).map_err(|e| format!("Failed to extract file: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy file content: {}", e))?;
            
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
    println!("[strategies] Successfully downloaded strategies.");
    Ok(())
}

pub fn install_dependencies(nfqws_ver: &str, strat_ver: &str) -> Result<(), String> {
    println!("=======================================================");
    println!("📥 Installing dependencies...");
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


pub fn fetch_repo_tags(repo: &str) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/repos/{}/tags", repo);
    let req = ureq::get(&url)
        .set("User-Agent", "zapret-rust-tui")
        .call()
        .map_err(|e| format!("Failed to fetch tags for {}: {}", repo, e))?;
    
    let json_str = req.into_string().map_err(|e| format!("Failed to read tags: {}", e))?;
    let tags_json: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse tags JSON: {}", e))?;
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
