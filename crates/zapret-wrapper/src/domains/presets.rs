pub struct DomainPreset {
    pub name: &'static str,
    pub domains: &'static [&'static str],
}

/// One editable text file per preset (index-aligned with `PRESETS`).
/// For built-in presets the file holds EXTRA domains added to the defaults;
/// for the last (Custom) preset the file is the full domain list.
pub const PRESET_FILES: &[&str] = &[
    "autotune_preset_discord.txt",
    "autotune_preset_youtube.txt",
    "autotune_preset_social.txt",
    "autotune_custom.txt",
];

/// The presets a sweep tests against before the user touches anything.
///
/// Discord and YouTube, because those are the two that a user of this program
/// is here to unblock, and sweeping every preset costs several times as long for
/// no extra answer. Named rather than written as indices: `PRESETS` is a list
/// someone will edit, and `[0, 1]` would quietly start meaning something else
/// the day a preset is added above them.
pub const DEFAULT_PRESETS: &[&str] = &["Discord", "YouTube"];

/// [`DEFAULT_PRESETS`] resolved to indices into [`PRESETS`].
///
/// A name that is not in the list is skipped rather than guessed at, so a renamed
/// preset means "one fewer default", never "the wrong domains".
pub fn default_preset_indices() -> Vec<usize> {
    DEFAULT_PRESETS
        .iter()
        .filter_map(|name| PRESETS.iter().position(|preset| preset.name == *name))
        .collect()
}

pub const PRESETS: &[DomainPreset] = &[
    DomainPreset {
        name: "Discord",
        domains: &[
            "dis.gd",
            "discord-attachments-uploads-prd.storage.googleapis.com",
            "discord.app",
            "discord.co",
            "discord.com",
            "discord.design",
            "discord.dev",
            "discord.gift",
            "discord.gifts",
            "discord.gg",
            "discord.media",
            "discord.new",
            "discord.store",
            "discord.status",
            "discord-activities.com",
            "discordactivities.com",
            "discordapp.com",
            "discordapp.net",
            "discordcdn.com",
            "discordmerch.com",
            "discordpartygames.com",
            "discordsays.com",
            "discordsez.com",
            "discordstatus.com",
            "cdn.discordapp.com",
            "discord-attachments-uploads.s3.amazonaws.com",
        ],
    },
    DomainPreset {
        name: "YouTube",
        domains: &[
            "googlevideo.com",
            "google.ru",
            "jnn-pa.googleapis.com",
            "play.google.com",
            "stable.dl2.discordapp.net",
            "wide-youtube.l.google.com",
            "yt-video-upload.l.google.com",
            "yt3.ggpht.com",
            "yt3.googleusercontent.com",
            "yt4.ggpht.com",
            "ytimg.com",
            "ytimg.l.google.com",
            "youtu.be",
            "youtube-nocookie.com",
            "youtube-ui.l.google.com",
            "youtube.com",
            "youtube.googleapis.com",
            "youtubeembeddedplayer.googleapis.com",
            "youtubei.googleapis.com",
            "youtubekids.com",
        ],
    },
    DomainPreset {
        name: "Social",
        domains: &[
            "twitter.com",
            "twimg.com",
            "reddit.com",
            "redditmedia.com",
            "t.me",
            "telegram.org",
            "instagram.com",
            "facebook.com",
            "whatsapp.com",
        ],
    },
    DomainPreset {
        name: "Custom",
        domains: &[],
    },
];
