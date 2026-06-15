import re

with open("src/tui/state.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace(
    'pub fn next_menu(&mut self) {\n        match self.active_screen {',
    'pub fn next_menu(&mut self) {\n        self.status_message = None;\n        match self.active_screen {'
)

content = content.replace(
    'pub fn prev_menu(&mut self) {\n        match self.active_screen {',
    'pub fn prev_menu(&mut self) {\n        self.status_message = None;\n        match self.active_screen {'
)

with open("src/tui/state.rs", "w", encoding="utf-8") as f:
    f.write(content)

