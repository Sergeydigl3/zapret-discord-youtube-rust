import os

with open("locales/ru.yml", "a", encoding="utf-8") as f:
    f.write('msg_closed_editor: "Закрыт редактор для "\n')
    f.write('val_tag: "Тег"\n')
    f.write('val_tag_fmt: "Тег ({})"\n')

with open("locales/en.yml", "a", encoding="utf-8") as f:
    f.write('msg_closed_editor: "Closed editor for "\n')
    f.write('val_tag: "Tag"\n')
    f.write('val_tag_fmt: "Tag ({})"\n')

def replace_in_file(path, replacements):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    for old, new in replacements:
        content = content.replace(old, new)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

ui_replacements = [
    (r'app.status_message = Some(format!("Error: {}", e));', r'app.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e));'),
    (r'app.status_message = Some(format!("Closed editor for {}", std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy()));', r'app.status_message = Some(format!("{}{}", rust_i18n::t!("msg_closed_editor"), std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy()));'),
]
replace_in_file("src/tui/ui.rs", ui_replacements)

download_submenu_replacements = [
    (r'VersionTarget::Tag(t) => format!("Tag ({})", t),', r'VersionTarget::Tag(t) => rust_i18n::t!("val_tag_fmt").replace("{}", &t),'),
    (r'_ => "Tag".to_string(),', r'_ => rust_i18n::t!("val_tag").into_owned(),'),
]
replace_in_file("src/tui/menus/download_submenu.rs", download_submenu_replacements)

