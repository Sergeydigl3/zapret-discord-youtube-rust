import re

with open("src/tui/menus/main_menu.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Define regex patterns to capture blocks
# Blocks look like:
#     // Name
#     {
#         let is_sel = app.main_menu == MainMenuState::Name;
#         ...
#         index += 1;
#     }
# We can just split the content or find the specific blocks and reorder them.

def extract_block(name_comment):
    pattern = re.compile(rf'(\s*// {name_comment}\s+{{.*?index \+= 1;\s*}})', re.DOTALL)
    match = pattern.search(content)
    if match:
        return match.group(1)
    # ListsEditor, ServiceSettings, GamefilterSettings, Strategy, Ipset Mode
    return None

interface_block = extract_block("Interface")
ipset_block = extract_block("Ipset Mode")
strategy_block = extract_block("Strategy")
gamefilter_block = extract_block("GamefilterSettings")
service_block = extract_block("ServiceSettings")
lists_editor_block = extract_block("ListsEditor")

# The run block is after ListsEditor.
# We will just replace the whole section from "// Interface" down to "// Run"
start_idx = content.find("    // Interface")
end_idx = content.find("    // Run")

if start_idx != -1 and end_idx != -1:
    before = content[:start_idx]
    after = content[end_idx:]
    
    new_middle = (
        interface_block + "\n" +
        strategy_block + "\n" +
        gamefilter_block + "\n" +
        ipset_block + "\n" +
        lists_editor_block + "\n" +
        service_block + "\n"
    )
    
    new_content = before + new_middle + after
    
    with open("src/tui/menus/main_menu.rs", "w", encoding="utf-8") as f:
        f.write(new_content)
    print("Reordered main_menu.rs")
else:
    print("Failed to find start or end index")

