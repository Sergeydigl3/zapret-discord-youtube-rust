//! The `--help` text.
//!
//! Every row is stored in the locale file as `flags<TAB>description`, so the
//! descriptions can line up in one column whatever the language. The value
//! lists in the descriptions are filled from the same tables the code uses:
//! adding a backend, a preset, a protocol or a check adds it to the help too.

use std::io::Write;

/// Print the help, ignoring a closed pipe (`zapret-rust --help | head`).
pub fn show_help() {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(render_help().as_bytes());
    let _ = stdout.flush();
}

/// Build the whole help text.
pub fn render_help() -> String {
    let sections = help_sections();
    let width = flag_column_width(&sections);

    let mut out = String::new();
    out.push_str(&rust_i18n::t!("cli_usage"));
    out.push('\n');
    out.push_str(&rust_i18n::t!("cli_desc"));
    out.push_str("\n\n");

    out.push_str(&rust_i18n::t!("cli_how_title"));
    out.push('\n');
    for key in ["cli_how_1", "cli_how_2", "cli_how_3"] {
        out.push_str("  ");
        out.push_str(&rust_i18n::t!(key));
        out.push('\n');
    }

    for (title, rows) in &sections {
        push_section(&mut out, title, rows, width);
    }

    push_examples(&mut out);
    out
}

/// All option groups, in the order they are printed.
fn help_sections() -> Vec<(String, Vec<String>)> {
    vec![
        (rust_i18n::t!("cli_group_common").to_string(), common_rows()),
        (rust_i18n::t!("cli_group_run").to_string(), run_rows()),
        (rust_i18n::t!("cli_group_settings").to_string(), settings_rows()),
        (rust_i18n::t!("cli_group_downloads").to_string(), download_rows()),
        (rust_i18n::t!("cli_group_actions").to_string(), action_rows()),
        (rust_i18n::t!("cli_group_inspect").to_string(), inspect_rows()),
    ]
}

/// Rows of the "common" group: options that fit every other group.
fn common_rows() -> Vec<String> {
    vec![
        rust_i18n::t!("cli_opt_d").to_string(),
        rust_i18n::t!("cli_opt_h").to_string(),
    ]
}

/// Rows of the "run" group.
fn run_rows() -> Vec<String> {
    let mut rows = vec![
        rust_i18n::t!("cli_opt_c").to_string(),
        rust_i18n::t!("cli_opt_s").to_string(),
    ];
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_i").to_string());
    rows.extend([
        rust_i18n::t!("cli_opt_t").to_string(),
        rust_i18n::t!("cli_opt_u").to_string(),
        rust_i18n::t!("cli_opt_no_tui").to_string(),
        rust_i18n::t!("cli_opt_run").to_string(),
    ]);
    rows
}

/// `--set-backend` choices, taken from the compiled backend list.
#[cfg(target_os = "linux")]
fn backend_names() -> String {
    use zapret_core::firewall::LinuxBackend;

    LinuxBackend::variants()
        .iter()
        .map(|backend| backend.to_config())
        .collect::<Vec<_>>()
        .join(", ")
}

/// `--set-ipset-mode` choices, taken from the mode enum.
fn ipset_mode_names() -> String {
    zapret_wrapper::lists::IpsetMode::all()
        .iter()
        .map(|mode| mode.keyword())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Autotune preset names, taken from the preset table.
fn preset_names() -> String {
    zapret_wrapper::domains::PRESETS
        .iter()
        .map(|preset| preset.name.to_lowercase())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Autotune protocol switches, taken from the protocol table.
fn protocol_names() -> String {
    zapret_wrapper::autotune::types::PROTOCOL_NAMES.join(", ")
}

/// Rows of the "settings" group.
fn settings_rows() -> Vec<String> {
    let mut rows = Vec::new();
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_set_interface").to_string());
    rows.push(rust_i18n::t!("cli_opt_set_strategy").to_string());
    rows.extend([
        rust_i18n::t!("cli_opt_set_gf_tcp").to_string(),
        rust_i18n::t!("cli_opt_set_gf_udp").to_string(),
    ]);
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_set_backend").replace("{backends}", &backend_names()));
    rows.push(rust_i18n::t!("cli_opt_set_ipset").replace("{modes}", &ipset_mode_names()));
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_set_router").to_string());
    rows.push(
        rust_i18n::t!("cli_opt_set_ttl")
            .replace("{min}", &zapret_wrapper::domains::ttl::TTL_MIN.to_string())
            .replace("{max}", &zapret_wrapper::domains::ttl::TTL_MAX.to_string()),
    );
    rows.extend([
        rust_i18n::t!("cli_opt_set_fake_discord").to_string(),
        rust_i18n::t!("cli_opt_set_fake_gamefilter").to_string(),
    ]);
    rows
}

/// Rows of the "downloads" group.
fn download_rows() -> Vec<String> {
    vec![
        rust_i18n::t!("cli_opt_dl_zapret").to_string(),
        rust_i18n::t!("cli_opt_dl_strat").to_string(),
        rust_i18n::t!("cli_opt_dl_defaults").to_string(),
    ]
}

/// Rows of the "actions" group.
fn action_rows() -> Vec<String> {
    vec![
        rust_i18n::t!("cli_opt_autotune").to_string(),
        rust_i18n::t!("cli_opt_autotune_presets").replace("{presets}", &preset_names()),
        rust_i18n::t!("cli_opt_autotune_strategies").to_string(),
        rust_i18n::t!("cli_opt_autotune_requests").to_string(),
        rust_i18n::t!("cli_opt_autotune_protocols").replace("{protocols}", &protocol_names()),
        rust_i18n::t!("cli_opt_ttl_autopick").to_string(),
        rust_i18n::t!("cli_opt_service_install").to_string(),
        rust_i18n::t!("cli_opt_service_start").to_string(),
        rust_i18n::t!("cli_opt_service_stop").to_string(),
        rust_i18n::t!("cli_opt_service_restart").to_string(),
        rust_i18n::t!("cli_opt_service_uninstall").to_string(),
        rust_i18n::t!("cli_opt_service_status").to_string(),
        rust_i18n::t!("cli_opt_defender_add").to_string(),
        rust_i18n::t!("cli_opt_defender_remove").to_string(),
        rust_i18n::t!("cli_opt_defender_status").to_string(),
        rust_i18n::t!("cli_opt_edit_list").to_string(),
        rust_i18n::t!("cli_opt_edit_domains").replace("{presets}", &preset_names()),
    ]
}

/// Rows of the "inspect" group.
fn inspect_rows() -> Vec<String> {
    let mut rows = Vec::new();
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_list_ifaces").to_string());
    rows.extend([
        rust_i18n::t!("cli_opt_list_strategies").to_string(),
        rust_i18n::t!("cli_opt_list_fakes").to_string(),
        rust_i18n::t!("cli_opt_list_lists").to_string(),
        rust_i18n::t!("cli_opt_list_domains").to_string(),
        rust_i18n::t!("cli_opt_list_tags_zapret").to_string(),
        rust_i18n::t!("cli_opt_list_tags_strategies").to_string(),
        rust_i18n::t!("cli_opt_status").to_string(),
        rust_i18n::t!("cli_opt_show_config").to_string(),
        rust_i18n::t!("cli_opt_autotune_results").to_string(),
    ]);
    rows
}

/// The widest flags column across every section, so descriptions line up in
/// one column from the first group to the last.
fn flag_column_width(sections: &[(String, Vec<String>)]) -> usize {
    sections
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| row.split_once('\t'))
        .map(|(flags, _)| flags.chars().count())
        .max()
        .unwrap_or(0)
}

fn push_section(out: &mut String, title: &str, rows: &[String], width: usize) {
    out.push('\n');
    out.push_str(title);
    out.push('\n');
    for row in rows {
        push_row(out, row, width);
    }
}

/// Print one `flags<TAB>description` row, padding the flags to `width`.
fn push_row(out: &mut String, row: &str, width: usize) {
    let (flags, description) = match row.split_once('\t') {
        Some(parts) => parts,
        None => {
            out.push_str("  ");
            out.push_str(row);
            out.push('\n');
            return;
        }
    };

    let column = 2 + width + 2;
    let padding = width.saturating_sub(flags.chars().count()) + 2;
    out.push_str("  ");
    out.push_str(flags);
    out.push_str(&" ".repeat(padding));

    let mut lines = description.split('\n');
    if let Some(first) = lines.next() {
        out.push_str(first);
    }
    out.push('\n');
    for line in lines {
        out.push_str(&" ".repeat(column));
        out.push_str(line);
        out.push('\n');
    }
}

fn push_examples(out: &mut String) {
    out.push('\n');
    out.push_str(&rust_i18n::t!("cli_group_examples"));
    out.push('\n');
    for n in 1..=5 {
        let title = rust_i18n::t!(format!("cli_example_{n}_title"));
        let command = rust_i18n::t!(format!("cli_example_{n}_cmd"));
        out.push_str("  ");
        out.push_str(&title);
        out.push('\n');
        out.push_str("    ");
        out.push_str(&command);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_help_row_has_flags_and_text() {
        let sections = help_sections();
        assert!(!sections.is_empty());
        for (title, rows) in &sections {
            assert!(!title.is_empty());
            assert!(!rows.is_empty(), "section '{title}' is empty");
            for row in rows {
                let (flags, description) = row.split_once('\t').unwrap_or_else(|| panic!("row '{row}' has no tab"));
                assert!(!flags.trim().is_empty(), "row '{row}' has no flags");
                assert!(!description.trim().is_empty(), "row '{row}' has no description");
            }
        }
    }

    #[test]
    fn help_descriptions_share_one_column() {
        let sections = help_sections();
        let width = flag_column_width(&sections);
        let column = 2 + width + 2;

        let mut out = String::new();
        for (title, rows) in &sections {
            push_section(&mut out, title, rows, width);
        }

        for row in sections.iter().flat_map(|(_, rows)| rows) {
            let (flags, description) = row.split_once('\t').unwrap();
            let first = description.split('\n').next().unwrap();
            let line = out
                .lines()
                .find(|line| line.starts_with(&format!("  {flags} ")))
                .unwrap_or_else(|| panic!("row '{flags}' is missing from the help"));
            let desc_at = line.chars().count() - first.chars().count();
            assert_eq!(desc_at, column, "misaligned description in '{row}'");
        }
    }

    #[test]
    fn continuation_lines_align_with_the_description_column() {
        let mut out = String::new();
        push_row(&mut out, "--wide VALUE\tFirst line\nSecond line", 14);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        let column = 2 + 14 + 2;
        assert_eq!(lines[0].chars().count() - "First line".chars().count(), column);
        assert_eq!(lines[1], format!("{}Second line", " ".repeat(column)));
    }

    #[test]
    fn help_mentions_every_group_and_example() {
        let help = render_help();
        for key in [
            "cli_how_title",
            "cli_group_common",
            "cli_group_run",
            "cli_group_settings",
            "cli_group_downloads",
            "cli_group_actions",
            "cli_group_inspect",
            "cli_group_examples",
        ] {
            assert!(help.contains(rust_i18n::t!(key).as_ref()), "help is missing '{key}'");
        }
        for n in 1..=5 {
            assert!(
                help.contains(rust_i18n::t!(format!("cli_example_{n}_title")).as_ref()),
                "help is missing example {n}"
            );
        }
    }
}
