//! Unpacking of the two downloaded payloads.
//!
//! zapret ships as a `tar.gz` release archive, the strategy repository ships as
//! a `zip`. Both are downloaded into the cache first and unpacked from there.

use std::fs;
use std::path::Path;

pub(crate) fn unpack_tar_gz(tmp_archive: &Path, tmp_dir: &Path) -> Result<(), String> {
    let tar_gz = fs::File::open(tmp_archive).map_err(|e| format!("{}{}", rust_i18n::t!("err_open_arc"), e))?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive
        .unpack(tmp_dir)
        .map_err(|e| format!("{}{}", rust_i18n::t!("err_unpack_tar"), e))?;
    Ok(())
}

pub(crate) fn unpack_zip(tmp_zip: &Path, target_dir: &Path) -> Result<(), String> {
    let zip_file = fs::File::open(tmp_zip).map_err(|e| format!("{}{}", rust_i18n::t!("err_open_zip"), e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("{}{}", rust_i18n::t!("err_read_zip"), e))?;

    let _ = fs::create_dir_all(target_dir);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => {
                let mut components = path.components();
                components.next(); // Skip root folder
                components.as_path().to_owned()
            }
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
            let mut outfile =
                fs::File::create(&full_path).map_err(|e| format!("{}{}", rust_i18n::t!("err_extract"), e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_copy_content"), e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&full_path, fs::Permissions::from_mode(mode)).ok();
                }
            }
        }
    }

    Ok(())
}
