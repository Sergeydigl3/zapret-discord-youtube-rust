import os

with open("locales/ru.yml", "a", encoding="utf-8") as f:
    f.write('menu_srv_restart: "🔄 Перезапустить службу"\n')

with open("locales/en.yml", "a", encoding="utf-8") as f:
    f.write('menu_srv_restart: "🔄 Restart Service"\n')

