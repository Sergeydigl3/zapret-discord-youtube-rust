import re

with open("src/tui/ui.rs", "r", encoding="utf-8") as f:
    ui_code = f.read()

# Replace the block that still refers to dependency_error
pattern = re.compile(
    r'// Dependencies status line\s*'
    r'let mut show_error = false;\s*'
    r'let mut error_msg = String::new\(\);\s*'
    r'if let Some\(\(ref msg, instant\)\) = app\.dependency_error \{.*?'
    r'let status_paragraph = Paragraph::new\(status_text\)',
    re.DOTALL
)

new_deps = """// Dependencies status line
                let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
                let strat_status = if app.strategies_installed { "✅" } else { "❌" };
                let status_text = Line::from(vec![
                    Span::styled(rust_i18n::t!("status_deps_title"), Style::default().fg(Color::Gray)),
                    Span::styled("nfqws ", Style::default().fg(Color::White)),
                    Span::raw(nfqws_status),
                    Span::styled(format!(" | {} ", rust_i18n::t!("status_deps_strat")), Style::default().fg(Color::White)),
                    Span::raw(strat_status),
                ]);
                let status_paragraph = Paragraph::new(status_text)"""

ui_code = pattern.sub(new_deps, ui_code)

with open("src/tui/ui.rs", "w", encoding="utf-8") as f:
    f.write(ui_code)

