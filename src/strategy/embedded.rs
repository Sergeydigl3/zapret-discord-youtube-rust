use std::env;
use std::fs;
use std::path::PathBuf;

/// Strategy scripts bundled into the binary. On startup they are written into
/// `<repo>/custom-strategies/` (if missing) so the user can pick them from the
/// strategy menu and the discovery logic can find them alongside any others.
const EMBEDDED_STRATEGIES: &[(&str, &str)] = &[GENERAL_SIMPLE_FAKE, GENERAL_NIX1, MINECRAFT_HYPIXEL_ETC];

const GENERAL_SIMPLE_FAKE: (&str, &str) = (
    "general_SIMPLE_FAKE.bat",
    r#"@echo off
chcp 65001 > nul
:: 65001 - UTF-8

cd /d "%~dp0"
call service.bat status_zapret
call service.bat check_updates
call service.bat load_game_filter
echo:

set "BIN=%~dp0bin\"
set "LISTS=%~dp0lists\"
cd /d %BIN%

start "zapret: %~n0" /min "%BIN%winws.exe" --wf-tcp=80,443,2053,2083,2087,2096,8443,%GameFilterTCP% --wf-udp=443,19294-19344,50000-50100,%GameFilterUDP% ^
--filter-udp=443 --hostlist="%LISTS%list-general.txt" --hostlist="%LISTS%list-general-user.txt" --hostlist-exclude="%LISTS%list-exclude.txt" --hostlist-exclude="%LISTS%list-exclude-user.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic="%BIN%quic_initial_www_google_com.bin" --new ^
--filter-udp=19294-19344,50000-50100 --filter-l7=discord,stun --dpi-desync=fake --dpi-desync-fake-discord="%BIN%quic_initial_www_google_com.bin" --dpi-desync-fake-discord="%BIN%ACTIVE_DISCORD_UDP.bin" --dpi-desync-fake-stun="%BIN%ACTIVE_DISCORD_UDP.bin" --dpi-desync-fake-unknown-udp="%BIN%quic_initial_www_google_com.bin" --dpi-desync-fake-unknown-udp="%BIN%ACTIVE_DISCORD_UDP.bin" --dpi-desync-repeats=6 --new ^
--filter-tcp=2053,2083,2087,2096,8443 --hostlist-domains=discord.media --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-tcp=443 --hostlist="%LISTS%list-google.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-tcp=80,443 --hostlist="%LISTS%list-general.txt" --hostlist="%LISTS%list-general-user.txt" --hostlist-exclude="%LISTS%list-exclude.txt" --hostlist-exclude="%LISTS%list-exclude-user.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-udp=443 --ipset="%LISTS%ipset-all.txt" --hostlist-exclude="%LISTS%list-exclude.txt" --hostlist-exclude="%LISTS%list-exclude-user.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic="%BIN%quic_initial_www_google_com.bin" --new ^
--filter-tcp=80,443,%GameFilterTCP% - --ipset="%LISTS%ipset-all.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-udp=%GameFilterUDP% --ipset="%LISTS%ipset-all.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-autottl=2 --dpi-desync-repeats=12 --dpi-desync-any-protocol=1 --dpi-desync-fake-unknown-udp="%BIN%quic_initial_www_google_com.bin" --dpi-desync-fake-unknown-udp="%BIN%ACTIVE_GAME_UDP.bin" --dpi-desync-cutoff=n3
"#,
);

const GENERAL_NIX1: (&str, &str) = (
    "general_nix1.bat",
    r#"@echo off
chcp 65001 > nul
:: 65001 - UTF-8

cd /d "%~dp0"
call service.bat status_zapret
call service.bat check_updates
call service.bat load_game_filter
echo:

set "BIN=%~dp0bin\"
set "LISTS=%~dp0lists\"
cd /d %BIN%

start "zapret: %~n0" /min "%BIN%winws.exe" --wf-tcp=80,443,2053,2083,2087,2096,8443,%GameFilterTCP% --wf-udp=443,19294-19344,50000-50100,%GameFilterUDP% ^
--filter-udp=443 --hostlist="%LISTS%list-general.txt" --hostlist="%LISTS%list-general-user.txt" --hostlist-exclude="%LISTS%list-exclude.txt" --hostlist-exclude="%LISTS%list-exclude-user.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic="%BIN%quic_initial_www_google_com.bin" --new ^
--filter-udp=19294-19344,50000-50100 --filter-l7=discord,stun --dpi-desync=fake --dpi-desync-fake-discord="%BIN%quic_initial_www_google_com.bin" --dpi-desync-fake-discord="%BIN%ACTIVE_DISCORD_UDP.bin" --dpi-desync-fake-stun="%BIN%ACTIVE_DISCORD_UDP.bin" --dpi-desync-fake-unknown-udp="%BIN%quic_initial_www_google_com.bin" --dpi-desync-fake-unknown-udp="%BIN%ACTIVE_DISCORD_UDP.bin" --dpi-desync-repeats=6 --new ^
--filter-tcp=2053,2083,2087,2096,8443 --hostlist-domains=discord.media --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-tcp=443 --hostlist="%LISTS%list-google.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-tcp=80,443 --hostlist="%LISTS%list-general.txt" --hostlist="%LISTS%list-general-user.txt" --hostlist-exclude="%LISTS%list-exclude.txt" --hostlist-exclude="%LISTS%list-exclude-user.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-udp=443 --ipset="%LISTS%ipset-all.txt"  --hostlist-exclude="%LISTS%list-exclude.txt" --hostlist-exclude="%LISTS%list-exclude-user.txt" --ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic="%BIN%quic_initial_www_google_com.bin" --new ^
--filter-tcp=80,443,%GameFilterTCP% --ipset="%LISTS%ipset-all.txt" -ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fooling=ts --dpi-desync-fake-tls="%BIN%tls_clienthello_www_google_com.bin" --new ^
--filter-udp=%GameFilterUDP% --ipset="%LISTS%ipset-all.txt" -ipset-exclude="%LISTS%ipset-exclude.txt" --ipset-exclude="%LISTS%ipset-exclude-user.txt" --dpi-desync=fake --dpi-desync-autottl=2 --dpi-desync-repeats=12 --dpi-desync-any-protocol=1 --dpi-desync-fake-unknown-udp="%BIN%quic_initial_www_google_com.bin" --dpi-desync-fake-unknown-udp="%BIN%ACTIVE_GAME_UDP.bin" --dpi-desync-cutoff=n3
"#,
);

const MINECRAFT_HYPIXEL_ETC: (&str, &str) = (
    "minecraft_hypixel_etc.bat",
    r#"@echo off
chcp 65001 > nul
:: 65001 - UTF-8

cd /d "%~dp0"
call service.bat status_zapret
call service.bat check_updates
call service.bat load_game_filter
echo:

set "BIN=%~dp0bin\"
set "LISTS=%~dp0lists\"
cd /d %BIN%


start "zapret: %~n0" /min "%BIN%winws.exe" ^
--wf-tcp=80,443,2053,2083,2087,2096,8443,25565 ^
--wf-udp=443,19294-19344,50000-50100,25565 ^

--filter-tcp=25565 --ipset-exclude="%LISTS%ipset-exclude.txt" --dpi-desync-any-protocol=1 --dpi-desync-cutoff=n5 --dpi-desync=multisplit --dpi-desync-split-seqovl=582 --dpi-desync-split-pos=1 --dpi-desync-split-seqovl-pattern="%BIN%tls_clienthello_4pda_to.bin" --new ^
"#,
);

/// Repository directory that holds the strategy `.bat` files and lists.
pub fn repo_dir() -> PathBuf {
    PathBuf::from(env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    }))
}

/// Write the bundled custom strategies into `<cache>/custom-strategies/`.
///
/// The custom-strategies folder lives next to the executable (in the cache
/// directory) rather than inside the downloaded strategy repo, so user-added
/// strategies survive repo re-downloads. The directory and any strategy file
/// that already exists is left untouched, so user-edited strategies are never
/// overwritten on subsequent runs.
pub fn ensure_custom_strategies() -> Result<(), String> {
    let dir = crate::config::get_cache_dir().join("custom-strategies");
    if dir.exists() && !dir.is_dir() {
        return Err(format!(
            "custom-strategies exists but is not a directory: {}",
            dir.display()
        ));
    }
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Cannot create custom-strategies dir: {}", e))?;
    }

    for (name, content) in EMBEDDED_STRATEGIES {
        let path = dir.join(name);
        if path.exists() {
            continue;
        }
        fs::write(&path, content).map_err(|e| format!("Cannot write custom strategy '{}': {}", name, e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_embedded_strategies() {
        std::env::set_var("ZAPRET_CACHE_DIR", "/tmp/cs_test");
        let dir = crate::config::get_cache_dir().join("custom-strategies");
        let _ = fs::remove_dir_all(&dir);

        ensure_custom_strategies().unwrap();

        for (name, _) in EMBEDDED_STRATEGIES {
            let p = dir.join(name);
            assert!(p.exists(), "missing {}", name);
            assert!(!fs::read_to_string(&p).unwrap().is_empty());
        }

        // Re-running must not error and must not overwrite existing files.
        let existing = fs::read_to_string(dir.join("general_nix1.bat")).unwrap();
        ensure_custom_strategies().unwrap();
        assert_eq!(fs::read_to_string(dir.join("general_nix1.bat")).unwrap(), existing);

        let _ = fs::remove_dir_all(&dir);
    }
}
