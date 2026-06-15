use std::path::Path;
use std::fs;

pub fn get_lists_files() -> Vec<String> {
    let mut files = Vec::new();
    
    // Check next to executable first
    let exe_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        
    let base_dir = exe_dir.join("zapret-discord-youtube-linux");
    let lists_dir = base_dir.join("lists");

    if lists_dir.exists() && lists_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(lists_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path.to_string_lossy().into_owned());
                }
            }
        }
    } else if base_dir.exists() && base_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "txt") {
                    files.push(path.to_string_lossy().into_owned());
                }
            }
        }
    } else {
        // Fallback to current directory for dev mode
        let local_base = Path::new("zapret-discord-youtube-linux");
        let local_lists = local_base.join("lists");
        
        if local_lists.exists() && local_lists.is_dir() {
            if let Ok(entries) = fs::read_dir(local_lists) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path.to_string_lossy().into_owned());
                    }
                }
            }
        } else if local_base.exists() && local_base.is_dir() {
            if let Ok(entries) = fs::read_dir(local_base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |e| e == "txt") {
                        files.push(path.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    
    files.sort();
    files
}

pub fn open_editor(file_path: &str) -> std::io::Result<std::process::ExitStatus> {
    let editors = [
        std::env::var("EDITOR").unwrap_or_default(),
        "nano".to_string(),
        "micro".to_string(),
        "nvim".to_string(),
        "vim".to_string(),
        "vi".to_string(),
        "notepad".to_string(), // Windows fallback
    ];

    for editor in editors.iter() {
        if editor.is_empty() {
            continue;
        }

        let status = std::process::Command::new(editor)
            .arg(file_path)
            .status();

        if let Ok(st) = status {
            if st.success() || st.code().is_some() {
                return Ok(st);
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No suitable editor found",
    ))
}
