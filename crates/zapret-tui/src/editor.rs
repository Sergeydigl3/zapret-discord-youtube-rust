/// Launch the user's editor on a list file.
///
/// Only the TUI opens files, so the editor handling lives here rather than in
/// `zapret-core::process`: picking a program from `$EDITOR` is a UI concern.
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

        let status = std::process::Command::new(editor).arg(file_path).status();

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
