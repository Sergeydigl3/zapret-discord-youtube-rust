import os
import re

ru_yml = open("locales/ru.yml", "a", encoding="utf-8")
en_yml = open("locales/en.yml", "a", encoding="utf-8")

def add_translation(key, en_text, ru_text):
    ru_yml.write(f'{key}: "{ru_text}"\n')
    en_yml.write(f'{key}: "{en_text}"\n')

# We'll manually specify the replacements to be safe
translations = {
    "Failed to fetch release info: ": ("err_fetch_rel", "Ошибка получения информации о релизе: "),
    "Failed to read bin dir: ": ("err_read_bin", "Ошибка чтения папки bin: "),
    "Failed to copy {:?}: ": ("err_copy_file", "Ошибка копирования {:?}: "),
    "Failed to copy binary: ": ("err_copy_bin", "Ошибка копирования бинарника: "),
    "Failed to download strategies zip: ": ("err_dl_strat_zip", "Ошибка скачивания zip архива стратегий: "),
    "Failed to create temp zip: ": ("err_create_tmp_zip", "Ошибка создания временного zip архива: "),
    "Failed to write zip: ": ("err_write_zip", "Ошибка записи zip архива: "),
    "Failed to open zip: ": ("err_open_zip", "Ошибка открытия zip архива: "),
    "Failed to read zip: ": ("err_read_zip", "Ошибка чтения zip архива: "),
    "Failed to create dir: ": ("err_mkdir", "Ошибка создания папки: "),
    "Failed to extract file: ": ("err_extract", "Ошибка извлечения файла: "),
    "Failed to copy file content: ": ("err_copy_content", "Ошибка копирования содержимого файла: "),
    "Failed to fetch tags for {}: ": ("err_fetch_tags", "Ошибка получения тегов для {}: "),
    "Failed to read tags: ": ("err_read_tags", "Ошибка чтения тегов: "),
    "Failed to parse tags JSON: ": ("err_parse_tags", "Ошибка парсинга JSON тегов: "),
    "Invalid executable path": ("err_invalid_exe", "Неверный путь к исполняемому файлу"),
    "Invalid config path": ("err_invalid_cfg", "Неверный путь к файлу конфигурации"),
    "Invalid cache directory path": ("err_invalid_cache", "Неверный путь к папке кэша"),
    "Failed to execute sc: ": ("err_exec_sc", "Ошибка выполнения sc: "),
    "Failed to start service dispatcher: ": ("err_start_dispatcher", "Ошибка запуска диспетчера службы: "),
    "Failed to execute systemctl: ": ("err_exec_systemctl", "Ошибка выполнения systemctl: "),
    "Failed to write service file: ": ("err_write_svc", "Ошибка записи файла службы: "),
    "Failed to remove service file: ": ("err_rm_svc", "Ошибка удаления файла службы: "),
    "Failed to execute sv: ": ("err_exec_sv", "Ошибка выполнения sv: "),
    "Failed to create runit service directory: ": ("err_mkdir_runit", "Ошибка создания папки службы runit: "),
    "Failed to write run script: ": ("err_write_run", "Ошибка записи скрипта запуска: "),
    "Failed to set run script as executable: ": ("err_chmod_run", "Ошибка установки прав на выполнение: "),
    "Failed to create symlink at {}: ": ("err_symlink", "Ошибка создания символической ссылки {}: "),
    "Failed to remove symlink {}: ": ("err_rm_symlink", "Ошибка удаления символической ссылки {}: "),
    "Failed to remove runit service directory: ": ("err_rm_runit", "Ошибка удаления папки службы runit: "),
    "Failed to execute s6-svc: ": ("err_exec_s6", "Ошибка выполнения s6-svc: "),
    "Failed to create s6 service directory: ": ("err_mkdir_s6", "Ошибка создания папки службы s6: "),
    "Failed to remove s6 service directory: ": ("err_rm_s6", "Ошибка удаления папки службы s6: "),
    "Failed to execute rc-service: ": ("err_exec_rc_svc", "Ошибка выполнения rc-service: "),
    "Failed to execute rc-update: ": ("err_exec_rc_update", "Ошибка выполнения rc-update: "),
    "Failed to write OpenRC init script: ": ("err_write_openrc", "Ошибка записи init скрипта OpenRC: "),
    "Failed to set executable permissions on OpenRC script: ": ("err_chmod_openrc", "Ошибка установки прав OpenRC скрипта: "),
    "Failed to remove OpenRC init script: ": ("err_rm_openrc", "Ошибка удаления init скрипта OpenRC: "),
    "Failed to execute init script: ": ("err_exec_init", "Ошибка выполнения init скрипта: "),
    "Failed to execute update-rc.d: ": ("err_exec_update_rc", "Ошибка выполнения update-rc.d: "),
    "Failed to execute chkconfig: ": ("err_exec_chkconfig", "Ошибка выполнения chkconfig: "),
    "Failed to write SysV init script: ": ("err_write_sysv", "Ошибка записи SysV init скрипта: "),
    "Failed to set executable permissions on SysV init script: ": ("err_chmod_sysv", "Ошибка установки прав SysV скрипта: "),
    "Failed to remove SysV init script: ": ("err_rm_sysv", "Ошибка удаления SysV init скрипта: "),
    "Failed to execute dinitctl: ": ("err_exec_dinit", "Ошибка выполнения dinitctl: "),
    "Failed to write dinit service file: ": ("err_write_dinit", "Ошибка записи файла службы dinit: "),
    "Failed to remove dinit service file: ": ("err_rm_dinit", "Ошибка удаления файла службы dinit: "),
    "Failed to create iptables chain: ": ("err_iptables_chain", "Ошибка создания цепочки iptables: "),
    "Failed to link iptables chain: ": ("err_iptables_link", "Ошибка привязки цепочки iptables: "),
}

for en_text, (key, ru_text) in translations.items():
    add_translation(key, en_text, ru_text)

ru_yml.close()
en_yml.close()

def replace_in_file(path):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    for en_text, (key, ru_text) in translations.items():
        if "{" in en_text and "}" in en_text:
            # It has formatting args
            # Old: format!("Failed to copy {:?}: {}", file_name, e)
            # New: format!("{}{:?}: {}", rust_i18n::t!("err_copy_file"), file_name, e)
            
            # Since rust macro string can have {}; we just replace the string literal 
            # and adjust formatting. But doing this with regex is tricky.
            # Easiest way: format!("Failed to copy {{:?}}: {{}}", ...)
            # We replace "Failed to copy {:?}: " with "{}{:?}: " and prepend rust_i18n::t!("key")
            pass

    # Actually, the simplest string replacement for format!("Failed to ...: {}", e)
    # is to replace the exact string "Failed to ...: {}" with "{}{}" and prepend the translation argument.
    
    # We will do explicit replacements.
    old_content = content
    
    for en_text, (key, ru_text) in translations.items():
        # Example 1: format!("Failed to read bin dir: {}", e)
        # We replace `"Failed to read bin dir: {}"` with `"{}{}", rust_i18n::t!("err_read_bin")`
        if "{}" not in en_text and "{:?}" not in en_text:
            content = content.replace(f'"{en_text}{{}}"', f'"{{}}{{}}", rust_i18n::t!("{key}")')
            content = content.replace(f'"{en_text}"', f'rust_i18n::t!("{key}").into_owned()')
        elif en_text == "Failed to copy {:?}: ":
            content = content.replace(f'"{en_text}{{}}"', f'"{{}}{{:?}}: {{}}", rust_i18n::t!("{key}")')
        elif en_text == "Failed to fetch tags for {}: ":
            content = content.replace(f'"{en_text}{{}}"', f'"{{}}{{}}: {{}}", rust_i18n::t!("{key}")')
        elif en_text == "Failed to create symlink at {}: ":
            content = content.replace(f'"{en_text}{{}}"', f'"{{}}{{}}: {{}}", rust_i18n::t!("{key}")')
        elif en_text == "Failed to remove symlink {}: ":
            content = content.replace(f'"{en_text}{{}}"', f'"{{}}{{}}: {{}}", rust_i18n::t!("{key}")')

    if old_content != content:
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)

# Process files
for root, dirs, files in os.walk("src"):
    for file in files:
        if file.endswith(".rs"):
            replace_in_file(os.path.join(root, file))

