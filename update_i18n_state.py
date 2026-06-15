import os

with open("locales/ru.yml", "a", encoding="utf-8") as f:
    f.write('msg_def_add_ok: "✅ Добавлено в исключения Windows Defender."\n')
    f.write('msg_def_rm_ok: "✅ Удалено из исключений Windows Defender."\n')
    f.write('msg_strat_sel: "✅ Выбрана стратегия: "\n')
    f.write('msg_op_ok: "✅ Операция успешно завершена."\n')
    f.write('msg_err: "❌ Ошибка: "\n')
    f.write('msg_err_init: "❌ Ошибка: Система инициализации не поддерживается."\n')
    f.write('msg_fetch_zapret_tags: "Получение тегов Zapret с GitHub... Пожалуйста, подождите."\n')
    f.write('msg_fetch_strat_tags: "Получение тегов Стратегий с GitHub... Пожалуйста, подождите."\n')
    f.write('msg_err_fetch_tags: "Ошибка получения тегов: "\n')
    f.write('msg_zapret_tag_sel: "Выбран тег для Zapret."\n')
    f.write('msg_strat_tag_sel: "Выбран тег для Стратегий."\n')

with open("locales/en.yml", "a", encoding="utf-8") as f:
    f.write('msg_def_add_ok: "✅ Added to Windows Defender Exclusions."\n')
    f.write('msg_def_rm_ok: "✅ Removed from Windows Defender Exclusions."\n')
    f.write('msg_strat_sel: "✅ Selected strategy: "\n')
    f.write('msg_op_ok: "✅ Operation completed successfully."\n')
    f.write('msg_err: "❌ Error: "\n')
    f.write('msg_err_init: "❌ Error: Init system not supported."\n')
    f.write('msg_fetch_zapret_tags: "Fetching Zapret tags from GitHub... Please wait."\n')
    f.write('msg_fetch_strat_tags: "Fetching Strategy tags from GitHub... Please wait."\n')
    f.write('msg_err_fetch_tags: "Failed to fetch tags: "\n')
    f.write('msg_zapret_tag_sel: "Tag selected for Zapret."\n')
    f.write('msg_strat_tag_sel: "Tag selected for Strategies."\n')

def replace_in_file(path, replacements):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    for old, new in replacements:
        content = content.replace(old, new)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

state_replacements = [
    (r'self.status_message = Some("\u{F00C} Added to Windows Defender Exclusions.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_def_add_ok").into_owned());'),
    (r'self.status_message = Some(format!("\u{F00D} {}", e)),', 'self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e)),'),
    (r'self.status_message = Some("\u{F00C} Removed from Windows Defender Exclusions.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_def_rm_ok").into_owned());'),
    (r'self.status_message = Some(format!("\u{F00C} Selected strategy: {}", self.strategies[self.selected_strategy]));', 'self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_strat_sel"), self.strategies[self.selected_strategy]));'),
    (r'self.status_message = Some("\u{F00C} Operation completed successfully.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_op_ok").into_owned());'),
    (r'self.status_message = Some(format!("\u{F00D} Error: {}", e));', 'self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e));'),
    (r'self.status_message = Some("\u{F00D} Error: Init system not supported.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_err_init").into_owned());'),
    (r'self.status_message = Some("Fetching Zapret tags from GitHub... Please wait.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_fetch_zapret_tags").into_owned());'),
    (r'self.status_message = Some("Fetching Strategy tags from GitHub... Please wait.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_fetch_strat_tags").into_owned());'),
    (r'self.status_message = Some(format!("Failed to fetch tags: {}", e));', 'self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), e));'),
    (r'self.status_message = Some("Tag selected for Zapret.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_zapret_tag_sel").into_owned());'),
    (r'self.status_message = Some("Tag selected for Strategies.".to_string());', 'self.status_message = Some(rust_i18n::t!("msg_strat_tag_sel").into_owned());'),
]
replace_in_file("src/tui/state.rs", state_replacements)

ui_replacements = [
    (r'("\u{F00D}", Color::Red, rust_i18n::t!("status_srv_not_inst"))', '("❌", Color::Red, rust_i18n::t!("status_srv_not_inst"))'),
    (r'("\u{F00C}", Color::Green, rust_i18n::t!("status_srv_active"))', '("✅", Color::Green, rust_i18n::t!("status_srv_active"))'),
    (r'("\u{F04C}", Color::Yellow, rust_i18n::t!("status_srv_stopped"))', '("⏸️", Color::Yellow, rust_i18n::t!("status_srv_stopped"))'),
]
replace_in_file("src/tui/ui.rs", ui_replacements)

tag_menu_replacements = [
    (r'let prefix = if i == selected_tag_index { "\u{F061} " } else { "   " };', 'let prefix = if i == selected_tag_index { "➡️ " } else { "   " };')
]
replace_in_file("src/tui/menus/tag_menu.rs", tag_menu_replacements)

strategy_menu_replacements = [
    (r'let prefix = if i == app.selected_strategy { "\u{F00C} " } else { "   " };', 'let prefix = if i == app.selected_strategy { "✅ " } else { "   " };')
]
replace_in_file("src/tui/menus/strategy_menu.rs", strategy_menu_replacements)

