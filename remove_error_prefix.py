import re

def process_yml(file_path, prefixes):
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    new_lines = []
    for line in lines:
        match = re.match(r'^([^:]+):\s*"(.*?)"$', line)
        if match:
            key = match.group(1)
            val = match.group(2)
            # Remove "Error: " or "Ошибка: " or "Ошибка " from the value
            for prefix in prefixes:
                if val.startswith(prefix):
                    val = val[len(prefix):]
                    # optionally capitalize the first letter
                    if val:
                        val = val[0].upper() + val[1:]
                    break
            new_lines.append(f'{key}: "{val}"\n')
        else:
            new_lines.append(line)
            
    with open(file_path, 'w', encoding='utf-8') as f:
        f.writelines(new_lines)

process_yml('locales/en.yml', ["Error: ", "Error ", "Failed to "])
process_yml('locales/ru.yml', ["Ошибка: ", "Ошибка ", "Неверный путь ", "Неверный "])

