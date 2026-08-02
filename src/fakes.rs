use std::fs;
use std::path::PathBuf;

const ACTIVE_DISCORD_NAME: &str = "ACTIVE_DISCORD_UDP.bin";
const ACTIVE_GAME_NAME: &str = "ACTIVE_GAME_UDP.bin";

#[derive(Debug, Clone)]
pub struct FakeFile {
    pub display_name: String,
    pub filename: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FakeTarget {
    DiscordUdp,
    GameUdp,
}

impl FakeTarget {
    pub fn active_filename(&self) -> &str {
        match self {
            FakeTarget::DiscordUdp => ACTIVE_DISCORD_NAME,
            FakeTarget::GameUdp => ACTIVE_GAME_NAME,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FakesState {
    pub discord_active: Option<String>,
    pub game_active: Option<String>,
    pub available: Vec<FakeFile>,
    pub bin_dir: PathBuf,
}

fn bin_dir() -> PathBuf {
    let repo_dir = std::env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    });
    std::path::PathBuf::from(repo_dir).join("bin")
}

pub fn scan_bin_dir() -> Vec<FakeFile> {
    let dir = bin_dir();
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if !name.ends_with(".bin") {
                continue;
            }
            if name.starts_with("ACTIVE_") {
                continue;
            }
            let display_name = name.trim_end_matches(".bin").to_string();
            files.push(FakeFile {
                display_name,
                filename: name,
                path,
            });
        }
    }

    files.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    files
}

pub fn load_fakes_state() -> FakesState {
    let dir = bin_dir();
    let available = scan_bin_dir();
    let (discord_source, game_source) = crate::config::load_active_fakes();

    let discord_active = if !discord_source.is_empty() {
        available
            .iter()
            .find(|f| f.filename == discord_source)
            .map(|f| f.display_name.clone())
    } else {
        None
    };

    let game_active = if !game_source.is_empty() {
        available
            .iter()
            .find(|f| f.filename == game_source)
            .map(|f| f.display_name.clone())
    } else {
        None
    };

    FakesState {
        discord_active,
        game_active,
        available,
        bin_dir: dir,
    }
}

pub fn replace_active_fake(
    state: &FakesState,
    target: &FakeTarget,
    source: &FakeFile,
) -> Result<(), String> {
    let dest = state.bin_dir.join(target.active_filename());
    fs::copy(&source.path, &dest).map_err(|e| format!("{}: {}", source.filename, e))?;

    let (mut discord, mut game) = crate::config::load_active_fakes();
    match target {
        FakeTarget::DiscordUdp => discord = source.filename.clone(),
        FakeTarget::GameUdp => game = source.filename.clone(),
    }
    crate::config::save_active_fakes(&discord, &game).map_err(|e| format!("config: {}", e))
}
