import os

with open("locales/ru.yml", "a") as f:
    f.write('err_srv: "Ошибка службы: "\n')
    f.write('err_load_cfg: "Ошибка загрузки конфигурации: "\n')
    f.write('err_ctrl_c: "Ошибка установки обработчика Ctrl-C: "\n')
    f.write('err_tui: "Ошибка TUI: "\n')

with open("locales/en.yml", "a") as f:
    f.write('err_srv: "Service error: "\n')
    f.write('err_load_cfg: "Error loading config: "\n')
    f.write('err_ctrl_c: "Error setting Ctrl-C handler: "\n')
    f.write('err_tui: "TUI Error: "\n')

def replace_in_file(path, replacements):
    with open(path, "r") as f:
        content = f.read()
    for old, new in replacements:
        content = content.replace(old, new)
    with open(path, "w") as f:
        f.write(content)

main_replacements = [
    ('eprintln!("Service error: {}", e);', 'eprintln!("{}{}", rust_i18n::t!("err_srv"), e);'),
    ('println!("Error loading config: {}", e);', 'println!("{}{}", rust_i18n::t!("err_load_cfg"), e);'),
    ('eprintln!("Error setting Ctrl-C handler: {}", e)', 'eprintln!("{}{}", rust_i18n::t!("err_ctrl_c"), e)'),
    ('println!("TUI Error: {}", e);', 'println!("{}{}", rust_i18n::t!("err_tui"), e);')
]
replace_in_file("src/main.rs", main_replacements)

