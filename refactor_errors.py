import re

with open("src/tui/state.rs", "r", encoding="utf-8") as f:
    state_code = f.read()

# 1. Remove dependency_error field
state_code = re.sub(r'\s*pub dependency_error: Option<\(String, std::time::Instant\)>,', '', state_code)
state_code = re.sub(r'\s*dependency_error: None,', '', state_code)

# 2. Add show_error handler
show_error_fn = """
    pub fn show_error(&mut self, msg: String) {
        self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), msg));
    }
"""

# Insert after refresh_defender_status
state_code = state_code.replace(
    'pub fn refresh_defender_status(&mut self) {\n        self.defender_active = crate::defender::is_defender_exclusion_active();\n    }',
    'pub fn refresh_defender_status(&mut self) {\n        self.defender_active = crate::defender::is_defender_exclusion_active();\n    }' + show_error_fn
)

# 3. Replace dependency_error usages
state_code = re.sub(r'self\.dependency_error = Some\(\(msg, std::time::Instant::now\(\)\)\);', r'self.show_error(msg);', state_code)

# Replace manual status_message assignments with show_error
state_code = re.sub(r'self\.status_message = Some\(format!\("{}\{\}", rust_i18n::t!\("msg_err"\), (.*?)\)\);', r'self.show_error(\1);', state_code)
state_code = re.sub(r'self\.status_message = Some\(format!\("{}\{\}", rust_i18n::t!\("msg_err_fetch_tags"\), (.*?)\)\);', r'self.show_error(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), \1));', state_code)
state_code = re.sub(r'self\.status_message = Some\(format!\("\{\}Стратегии не скачаны", rust_i18n::t!\("msg_err"\)\)\);', r'self.show_error("Стратегии не скачаны".to_string());', state_code)

# 4. Clear status_message on movement
# Let's find the `handle_key_events` and inject `self.status_message = None;` at the beginning of each key match if it's a movement key, or just before the match.
# Wait, if we clear it before the match, we might clear a message set BY the key.
# It's better to clear it when navigating menus: KeyCode::Up, KeyCode::Down, etc.
# But `handle_key_events` has a big match.
# Instead of doing complex parsing in python, I'll write a Rust script or manually replace.
# Actually, I can just inject `self.status_message = None;` in the specific `KeyCode::Up` and `KeyCode::Down` branches inside `match key.code`.

with open("src/tui/state.rs", "w", encoding="utf-8") as f:
    f.write(state_code)


with open("src/tui/ui.rs", "r", encoding="utf-8") as f:
    ui_code = f.read()

# Replace manual error setting in ui.rs
ui_code = re.sub(r'app\.status_message = Some\(format!\("{}\{\}", rust_i18n::t!\("msg_err"\), (.*?)\)\);', r'app.show_error(\1.to_string());', ui_code)

# Fix help_text rendering
old_help_text = """            let help_text = if let Some(ref msg) = app.status_message {
                format!("{} | {}", msg, help_msg)
            } else {
                help_msg
            };"""

new_help_text = """            let help_text = if let Some(ref msg) = app.status_message {
                msg.clone()
            } else {
                help_msg
            };"""
ui_code = ui_code.replace(old_help_text, new_help_text)

# Remove the dependency status error overwrite
old_deps = """                // Dependencies status line
                let mut show_error = false;
                let mut error_msg = String::new();
                
                if let Some((ref msg, instant)) = app.dependency_error {
                    if instant.elapsed() < std::time::Duration::from_secs(3) {
                        show_error = true;
                        error_msg = msg.clone();
                    }
                }

                let status_text = if show_error {
                    Line::from(vec![
                        Span::styled(error_msg, Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD)),
                    ])
                } else {
                    let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
                    let strat_status = if app.strategies_installed { "✅" } else { "❌" };
                    Line::from(vec![
                        Span::styled(rust_i18n::t!("status_deps_title"), Style::default().fg(Color::Gray)),
                        Span::styled("nfqws ", Style::default().fg(Color::White)),
                        Span::raw(nfqws_status),
                        Span::styled(format!(" | {} ", rust_i18n::t!("status_deps_strat")), Style::default().fg(Color::White)),
                        Span::raw(strat_status),
                    ])
                };"""

new_deps = """                // Dependencies status line
                let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
                let strat_status = if app.strategies_installed { "✅" } else { "❌" };
                let status_text = Line::from(vec![
                    Span::styled(rust_i18n::t!("status_deps_title"), Style::default().fg(Color::Gray)),
                    Span::styled("nfqws ", Style::default().fg(Color::White)),
                    Span::raw(nfqws_status),
                    Span::styled(format!(" | {} ", rust_i18n::t!("status_deps_strat")), Style::default().fg(Color::White)),
                    Span::raw(strat_status),
                ]);"""
ui_code = ui_code.replace(old_deps, new_deps)

with open("src/tui/ui.rs", "w", encoding="utf-8") as f:
    f.write(ui_code)

