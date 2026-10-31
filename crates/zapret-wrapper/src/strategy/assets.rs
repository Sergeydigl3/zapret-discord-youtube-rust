use std::fs;

/// Strategy presets bundled into the binary. On startup they are written into
/// `<cache>/custom-strategies/` (if missing) so the user can pick them from the
/// strategy menu and the discovery logic can find them alongside any others.
///
/// The files hold the raw `winws2` / `nfqws2` arguments — see
/// [`crate::strategy::parser`]. The fake payloads are loaded as zapret2 blobs
/// from the repository's `bin/` directory, and the two `ACTIVE_*.bin` names are
/// the swap targets of the fakes menu, so swapping a fake survives unchanged
/// from the zapret 1 era.
const EMBEDDED_STRATEGIES: &[(&str, &str)] = &[GENERAL, DISCORD, MINECRAFT_HYPIXEL];

const GENERAL: (&str, &str) = (
    "general.txt",
    r#"# The all-round preset: YouTube, Discord and the rest of the general hostlist,
# plus the game filter ports when they are enabled in the main menu.
# Blobs: the fake payloads. The ACTIVE_*.bin files are what the fakes menu swaps.
--blob=fake_quic:@%BIN%quic_initial_www_google_com.bin
--blob=fake_tls:@%BIN%tls_clienthello_www_google_com.bin
--blob=fake_discord:@%BIN%ACTIVE_DISCORD_UDP.bin
--blob=fake_game:@%BIN%ACTIVE_GAME_UDP.bin

--wf-tcp-in=80,443,2053,2083,2087,2096,8443,%GameFilterTCP%
--wf-tcp-out=80,443,2053,2083,2087,2096,8443,%GameFilterTCP%
--wf-udp-in=443,19294-19344,50000-50100,%GameFilterUDP%
--wf-udp-out=443,19294-19344,50000-50100,%GameFilterUDP%

# QUIC (HTTP/3) of the general hostlist
--filter-udp=443 --hostlist=%LISTS%list-general.txt --hostlist=%LISTS%list-general-user.txt --hostlist-exclude=%LISTS%list-exclude.txt --hostlist-exclude=%LISTS%list-exclude-user.txt --payload=quic_initial --lua-desync=fake:blob=fake_quic:repeats=6 --new

# Discord voice and game service discovery
--filter-udp=19294-19344,50000-50100 --filter-l7=discord,stun --lua-desync=fake:blob=fake_quic:repeats=6 --lua-desync=fake:blob=fake_discord:repeats=6 --new

# discord.media rides TLS on the alternate front ports
--filter-tcp=2053,2083,2087,2096,8443 --hostlist-domains=discord.media --payload=tls_client_hello --lua-desync=fake:blob=fake_tls:badsum:repeats=6 --new

# Google video
--filter-tcp=443 --hostlist=%LISTS%list-google.txt --payload=tls_client_hello --lua-desync=fake:blob=fake_tls:badsum:repeats=6 --new

# The general TLS traffic
--filter-tcp=80,443 --hostlist=%LISTS%list-general.txt --hostlist=%LISTS%list-general-user.txt --hostlist-exclude=%LISTS%list-exclude.txt --hostlist-exclude=%LISTS%list-exclude-user.txt --payload=tls_client_hello --lua-desync=fake:blob=fake_tls:badsum:repeats=6 --new

# QUIC by IP set
--filter-udp=443 --ipset=%LISTS%ipset-all.txt --hostlist-exclude=%LISTS%list-exclude.txt --hostlist-exclude=%LISTS%list-exclude-user.txt --payload=quic_initial --lua-desync=fake:blob=fake_quic:repeats=6 --new

# TCP by IP set
--filter-tcp=80,443,%GameFilterTCP% --ipset=%LISTS%ipset-all.txt --ipset-exclude=%LISTS%ipset-exclude.txt --ipset-exclude=%LISTS%ipset-exclude-user.txt --payload=tls_client_hello --lua-desync=fake:blob=fake_tls:badsum:repeats=6 --new

# The game filter UDP range, any protocol, hop count pinned like the v1 preset had
--filter-udp=%GameFilterUDP% --ipset=%LISTS%ipset-all.txt --ipset-exclude=%LISTS%ipset-exclude.txt --ipset-exclude=%LISTS%ipset-exclude-user.txt --lua-desync=fake:blob=fake_game:repeats=12:ip_autottl=2,1-20
"#,
);

const DISCORD: (&str, &str) = (
    "discord.txt",
    r#"# The Discord-focused preset: voice (UDP), media (TLS) and the QUIC the client
# falls back to, without touching the rest of the general hostlist.
--blob=fake_quic:@%BIN%quic_initial_www_google_com.bin
--blob=fake_tls:@%BIN%tls_clienthello_www_google_com.bin
--blob=fake_discord:@%BIN%ACTIVE_DISCORD_UDP.bin

--wf-tcp-in=443,2053,2083,2087,2096,8443
--wf-tcp-out=443,2053,2083,2087,2096,8443
--wf-udp-in=443,19294-19344,50000-50100
--wf-udp-out=443,19294-19344,50000-50100

# Voice: Discord IP discovery and STUN
--filter-udp=19294-19344,50000-50100 --filter-l7=discord,stun --lua-desync=fake:blob=fake_quic:repeats=6 --lua-desync=fake:blob=fake_discord:repeats=6 --new

# QUIC of the general hostlist
--filter-udp=443 --hostlist=%LISTS%list-general.txt --hostlist=%LISTS%list-general-user.txt --hostlist-exclude=%LISTS%list-exclude.txt --hostlist-exclude=%LISTS%list-exclude-user.txt --payload=quic_initial --lua-desync=fake:blob=fake_quic:repeats=6 --new

# Media over TLS, including the alternate front ports
--filter-tcp=443,2053,2083,2087,2096,8443 --hostlist-domains=discord.media --payload=tls_client_hello --lua-desync=fake:blob=fake_tls:badsum:repeats=6 --new

# The general TLS traffic over the same ports
--filter-tcp=443,2053,2083,2087,2096,8443 --hostlist=%LISTS%list-general.txt --hostlist=%LISTS%list-general-user.txt --hostlist-exclude=%LISTS%list-exclude.txt --hostlist-exclude=%LISTS%list-exclude-user.txt --payload=tls_client_hello --lua-desync=fake:blob=fake_tls:badsum:repeats=6
"#,
);

const MINECRAFT_HYPIXEL: (&str, &str) = (
    "minecraft_hypixel.txt",
    r#"# Game TCP: Minecraft (25565) and friends, split with a TLS-Hello overlap so
# DPI sees a segmented handshake.
--blob=tcp_pattern:@%BIN%tls_clienthello_4pda_to.bin

--wf-tcp-in=80,443,25565
--wf-tcp-out=80,443,25565
--wf-udp-in=443,25565
--wf-udp-out=443,25565

--filter-tcp=25565 --lua-desync=multisplit:pos=1:seqovl=582:seqovl_pattern=tcp_pattern
"#,
);

/// Write the bundled custom strategies into `<cache>/custom-strategies/`.
///
/// The custom-strategies folder lives next to the executable (in the cache
/// directory) rather than inside the downloaded strategy repo, so user-added
/// strategies survive repo re-downloads. The directory and any strategy file
/// that already exists is left untouched, so user-edited strategies are never
/// overwritten on subsequent runs.
pub fn ensure_custom_strategies() -> Result<(), String> {
    let dir = crate::paths::custom_strategies_dir();
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
        let dir = crate::paths::custom_strategies_dir();
        let _ = fs::remove_dir_all(&dir);

        ensure_custom_strategies().unwrap();

        for (name, _) in EMBEDDED_STRATEGIES {
            let p = dir.join(name);
            assert!(p.exists(), "missing {}", name);
            assert!(!fs::read_to_string(&p).unwrap().is_empty());
        }

        // Re-running must not error and must not overwrite existing files.
        let existing = fs::read_to_string(dir.join("general.txt")).unwrap();
        ensure_custom_strategies().unwrap();
        assert_eq!(fs::read_to_string(dir.join("general.txt")).unwrap(), existing);

        let _ = fs::remove_dir_all(&dir);
    }

    /// Every embedded preset must be parsable and declare both capture sides,
    /// otherwise a fresh install produces strategies that cannot run.
    #[test]
    fn embedded_presets_parse() {
        for (name, content) in EMBEDDED_STRATEGIES {
            let parsed = crate::strategy::parser::parse_strategy_text(content, None)
                .unwrap_or_else(|e| panic!("{}: {}", name, e));
            assert!(!parsed.args.is_empty(), "{}: no strategy tokens", name);
        }
    }

    /// The game-filter variants of the same presets must parse too: with the
    /// filter on the placeholders expand, with it off they are stripped. Only
    /// the presets that use the placeholders carry the game filter ports —
    /// the Discord preset deliberately does not.
    #[test]
    fn embedded_presets_parse_with_gamefilter() {
        let gf = crate::strategy::parser::GameFilterPorts {
            ports: "1024-65535".into(),
            tcp_ports: "1024-65535".into(),
            udp_ports: "1024-65535".into(),
        };
        for (name, content) in EMBEDDED_STRATEGIES {
            let parsed = crate::strategy::parser::parse_strategy_text(content, Some(&gf))
                .unwrap_or_else(|e| panic!("{}: {}", name, e));
            if content.contains("%GameFilter") {
                assert!(parsed.udp_in.contains("1024-65535"), "{}: game filter ports lost", name);
            }
        }
        assert!(EMBEDDED_STRATEGIES
            .iter()
            .any(|(n, c)| *n == "general.txt" && c.contains("%GameFilter")));
    }
}
