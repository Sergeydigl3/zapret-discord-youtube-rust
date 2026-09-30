//! Command-line surface of zapret-rust.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "run",
    about = "Run zapret in foreground (useful for testing).",
    disable_help_flag = true
)]
pub struct Cli {
    #[arg(short = 'c', long = "config", help = "Load configuration from file")]
    pub config: Option<String>,

    #[arg(short = 's', long = "strategy", help = "Use specific strategy")]
    pub strategy: Option<String>,

    #[arg(
        short = 'i',
        long = "interface",
        default_value = "any",
        help = "Network interface (default: any)"
    )]
    pub interface: String,

    #[arg(long = "gamefiltertcp", short = 't', help = "Enable gamefiltertcp")]
    pub gamefiltertcp: bool,

    #[arg(long = "gamefilterudp", short = 'u', help = "Enable gamefilterudp")]
    pub gamefilterudp: bool,

    #[arg(
        long = "cache-dir",
        short = 'd',
        help = "Cache directory for downloaded dependencies and strategies"
    )]
    pub cache_dir: Option<String>,

    #[arg(short = 'h', long = "help", help = "Show this help")]
    pub help: bool,
}

pub fn show_help() {
    println!("{}", rust_i18n::t!("cli_usage"));
    println!("{}", rust_i18n::t!("cli_desc"));
    println!("{}", rust_i18n::t!("cli_opts"));
    println!("{}", rust_i18n::t!("cli_opt_c"));
    println!("{}", rust_i18n::t!("cli_opt_s"));
    println!("{}", rust_i18n::t!("cli_opt_i"));
    println!("{}", rust_i18n::t!("cli_opt_t"));
    println!("{}", rust_i18n::t!("cli_opt_u"));
    println!("{}", rust_i18n::t!("cli_opt_d"));
    println!("{}", rust_i18n::t!("cli_opt_h"));
    println!("{}", rust_i18n::t!("cli_modes"));
    println!("{}", rust_i18n::t!("cli_mode1"));
    println!("{}", rust_i18n::t!("cli_mode2"));
    println!("{}", rust_i18n::t!("cli_mode3"));
}
