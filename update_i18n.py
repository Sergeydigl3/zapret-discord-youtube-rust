import os

# Update ru.yml
with open("locales/ru.yml", "a") as f:
    f.write('\nerr_dl_arc: "Ошибка скачивания архива: "\n')
    f.write('err_create_file: "Ошибка создания файла: "\n')
    f.write('err_write_arc: "Ошибка записи архива: "\n')
    f.write('err_open_arc: "Ошибка открытия архива: "\n')
    f.write('err_unpack_tar: "Ошибка распаковки tar: "\n')
    f.write('msg_clear_windivert: "Очистка правил windivert... (заглушка)"\n')
    f.write('msg_setup_windivert: "Настройка windivert... (заглушка)"\n')
    f.write('msg_clear_iptables: "Очистка правил iptables..."\n')
    f.write('msg_setup_iptables: "Настройка iptables..."\n')
    f.write('msg_clear_nftables: "Очистка правил nftables..."\n')
    f.write('msg_setup_nftables: "Настройка nftables..."\n')

# Update en.yml
with open("locales/en.yml", "a") as f:
    f.write('\nerr_dl_arc: "Error downloading archive: "\n')
    f.write('err_create_file: "Error creating file: "\n')
    f.write('err_write_arc: "Error writing archive: "\n')
    f.write('err_open_arc: "Error opening archive: "\n')
    f.write('err_unpack_tar: "Error unpacking tar: "\n')
    f.write('msg_clear_windivert: "Clearing windivert rules... (stub)"\n')
    f.write('msg_setup_windivert: "Setting up windivert... (stub)"\n')
    f.write('msg_clear_iptables: "Clearing iptables rules..."\n')
    f.write('msg_setup_iptables: "Setting up iptables..."\n')
    f.write('msg_clear_nftables: "Clearing nftables rules..."\n')
    f.write('msg_setup_nftables: "Setting up nftables..."\n')

def replace_in_file(path, replacements):
    with open(path, "r") as f:
        content = f.read()
    for old, new in replacements:
        content = content.replace(old, new)
    with open(path, "w") as f:
        f.write(content)

runner_replacements = [
    ('println!("Error parsing strategy: {}", e);', 'println!("{}{}", rust_i18n::t!("err_parse_strat"), e);'),
    ('println!("Error configuring firewall: {}", e);', 'println!("{}{}", rust_i18n::t!("msg_err_firewall"), e);'),
    ('println!("Запуск nfqws...");', 'println!("{}", rust_i18n::t!("msg_start_nfqws"));'),
    ('println!("Бинарник не найден: {:?}. Сначала выполните установку зависимостей.", bin_path);', 'println!("{}", rust_i18n::t!("err_bin_miss").replace("{:?}", &format!("{:?}", bin_path)));'),
    ('println!("  (не удалось setcap cap_net_admin+ep, может потребоваться root)"),', 'println!("{}", rust_i18n::t!("err_setcap")),'),
    ('println!("  Команда: {:?} {:?}", bin_path, args);', 'println!("{}{:?} {:?}", rust_i18n::t!("msg_cmd"), bin_path, args);'),
    ('println!("  nfqws запущен");', 'println!("{}", rust_i18n::t!("msg_nfqws_run"));'),
    ('println!("  Ошибка запуска nfqws: {}", e);', 'println!("{}{}", rust_i18n::t!("err_start_nfqws"), e);'),
    ('println!("\\nОстановка nfqws...");', 'println!("{}", rust_i18n::t!("msg_zapret_stop"));'),
    ('println!("Очистка завершена.");', 'println!("{}", rust_i18n::t!("msg_zapret_clear"));'),
]
replace_in_file("src/runner.rs", runner_replacements)

download_replacements = [
    ('println!("\\n[nfqws] Checking binary...");', 'println!("{}", rust_i18n::t!("msg_chk_nfqws"));'),
    ('println!("[nfqws] Detected platform: {}", platform);', 'println!("{}{}", rust_i18n::t!("msg_det_plat"), platform);'),
    ('println!("[nfqws] Fetching latest release tag...");', 'println!("{}", rust_i18n::t!("msg_fetch_rel"));'),
    ('println!("[nfqws] Using tag: {}", tag);', 'println!("{}{}", rust_i18n::t!("msg_using_tag"), tag);'),
    ('println!("[nfqws] Downloading archive: {}", url);', 'println!("{}{}", rust_i18n::t!("msg_dl_arc"), url);'),
    ('format!("Ошибка скачивания архива: {}", e)', 'format!("{}{}", rust_i18n::t!("err_dl_arc"), e)'),
    ('format!("Ошибка создания файла: {}", e)', 'format!("{}{}", rust_i18n::t!("err_create_file"), e)'),
    ('format!("Ошибка записи архива: {}", e)', 'format!("{}{}", rust_i18n::t!("err_write_arc"), e)'),
    ('println!("[nfqws] Extracting archive...");', 'println!("{}", rust_i18n::t!("msg_ext_arc"));'),
    ('format!("Ошибка открытия архива: {}", e)', 'format!("{}{}", rust_i18n::t!("err_open_arc"), e)'),
    ('format!("Ошибка распаковки tar: {}", e)', 'format!("{}{}", rust_i18n::t!("err_unpack_tar"), e)'),
    ('println!("[nfqws] Successfully installed winws and dependencies to bin/");', 'println!("{}", rust_i18n::t!("msg_inst_win_ok"));'),
    ('println!("[nfqws] Successfully installed nfqws to bin/{}", bin_name);', 'println!("{}{}", rust_i18n::t!("msg_inst_nux_ok"), bin_name);'),
    ('println!("[nfqws] cap_net_admin+ep set on nfqws");', 'println!("{}", rust_i18n::t!("msg_setcap_ok"));'),
    ('println!("\\n[strategies] Downloading via HTTP...");', 'println!("{}", rust_i18n::t!("msg_dl_strat"));'),
    ('println!("[strategies] Extracting strategies...");', 'println!("{}", rust_i18n::t!("msg_ext_strat"));'),
    ('println!("[strategies] Successfully downloaded strategies.");', 'println!("{}", rust_i18n::t!("msg_strat_ok"));'),
    ('println!("\\u{F01A} Installing dependencies...");', 'println!("{}", rust_i18n::t!("msg_inst_deps"));'),
]
replace_in_file("src/download.rs", download_replacements)

windivert_replacements = [
    ('println!("Очистка правил windivert... (заглушка)");', 'println!("{}", rust_i18n::t!("msg_clear_windivert"));'),
    ('println!("Настройка windivert... (заглушка)");', 'println!("{}", rust_i18n::t!("msg_setup_windivert"));'),
]
replace_in_file("src/firewalls/windivert.rs", windivert_replacements)

iptables_replacements = [
    ('println!("Очистка правил iptables...");', 'println!("{}", rust_i18n::t!("msg_clear_iptables"));'),
    ('println!("Настройка iptables...");', 'println!("{}", rust_i18n::t!("msg_setup_iptables"));'),
]
replace_in_file("src/firewalls/iptables.rs", iptables_replacements)

nftables_replacements = [
    ('println!("Очистка правил nftables...");', 'println!("{}", rust_i18n::t!("msg_clear_nftables"));'),
    ('println!("Настройка nftables...");', 'println!("{}", rust_i18n::t!("msg_setup_nftables"));'),
]
replace_in_file("src/firewalls/nftables.rs", nftables_replacements)

