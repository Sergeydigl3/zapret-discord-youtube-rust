import re

with open("src/tui/menus/main_menu.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Replace unwrap_or(&rust_i18n::t!("val_none").into_owned()) with unwrap_or(&String::new())
content = content.replace('.unwrap_or(&rust_i18n::t!("val_none").into_owned())', '.unwrap_or(&String::new())')

with open("src/tui/menus/main_menu.rs", "w", encoding="utf-8") as f:
    f.write(content)


with open("src/ipset.rs", "r", encoding="utf-8") as f:
    content = f.read()

new_modes = """pub fn get_available_modes() -> Vec<IpsetMode> {
    if !crate::download::check_strategies_installed() {
        return vec![IpsetMode::None];
    }

    let mut modes = vec![IpsetMode::None, IpsetMode::Any, IpsetMode::Loaded];"""

content = content.replace('pub fn get_available_modes() -> Vec<IpsetMode> {\n    let mut modes = vec![IpsetMode::None, IpsetMode::Any, IpsetMode::Loaded];', new_modes)

with open("src/ipset.rs", "w", encoding="utf-8") as f:
    f.write(content)


with open("src/tui/state.rs", "r", encoding="utf-8") as f:
    content = f.read()

new_lists_editor = """                    MainMenuState::ListsEditor => {
                        if !crate::download::check_strategies_installed() {
                            self.status_message = Some(format!("{}Стратегии не скачаны", rust_i18n::t!("msg_err")));
                        } else {
                            self.lists_files = crate::utils::get_lists_files();
                            self.lists_menu_index = 0;
                            self.active_screen = ActiveScreen::ListsEditorSubmenu;
                            self.status_message = None;
                        }
                    }"""

# Need to accurately replace the ListsEditor block
pattern = re.compile(r'                    MainMenuState::ListsEditor => \{\s+self\.lists_files = crate::utils::get_lists_files\(\);\s+self\.lists_menu_index = 0;\s+self\.active_screen = ActiveScreen::ListsEditorSubmenu;\s+self\.status_message = None;\s+\}')
content = pattern.sub(new_lists_editor, content)

with open("src/tui/state.rs", "w", encoding="utf-8") as f:
    f.write(content)

