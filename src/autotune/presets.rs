pub struct DomainPreset {
    pub name: &'static str,
    pub domains: &'static [&'static str],
}

pub const PRESETS: &[DomainPreset] = &[
    DomainPreset {
        name: "Discord",
        domains: &[
            "discord.com", "discordapp.com", "discordapp.net",
            "discord.gg", "discordstatus.com", "discord.media",
            "discord.gift", "discord.gifts", "discord.new",
            "discord.co", "discord.store", "discord.status",
            "discord.design", "discord.dev", "discord.app",
            "discordcdn.com", "discordmerch.com",
            "discord-activities.com", "discordactivities.com",
            "discordpartygames.com", "discordsays.com", "discordsez.com",
            "cdn.discordapp.com",
            "discord-attachments-uploads.s3.amazonaws.com",
            "discord-attachments-uploads-prd.storage.googleapis.com",
        ],
    },
    DomainPreset {
        name: "YouTube",
        domains: &[
            "youtube.com", "ytimg.com", "googlevideo.com", "youtu.be",
            "ggpht.com", "googleusercontent.com", "withyoutube.com",
            "yt3.ggpht.com", "yt4.ggpht.com",
            "yt3.googleusercontent.com",
            "jnn-pa.googleapis.com",
            "stable.dl2.discordapp.net",
            "wide-youtube.l.google.com",
            "youtube-nocookie.com",
            "youtube-ui.l.google.com",
            "youtubeembeddedplayer.googleapis.com",
            "youtubekids.com",
            "youtubei.googleapis.com",
            "yt-video-upload.l.google.com",
            "ytimg.l.google.com",
            "play.google.com",
        ],
    },
    DomainPreset {
        name: "Social",
        domains: &[
            "twitter.com", "twimg.com", "reddit.com", "redditmedia.com",
            "t.me", "telegram.org", "instagram.com", "facebook.com",
            "whatsapp.com",
        ],
    },
    DomainPreset {
        name: "Custom",
        domains: &[],
    },
];
