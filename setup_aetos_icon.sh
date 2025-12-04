#!/bin/bash

# Установка иконки для файлов .aetos (упрощенная версия)

echo "Настраиваем иконку для .aetos файлов..."

# Создаем необходимые директории
mkdir -p ~/.local/share/mime/packages
mkdir -p ~/.local/share/icons/hicolor/scalable/mimetypes

# 1. Создаем MIME-тип для .aetos файлов
cat > ~/.local/share/mime/packages/aetos.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-aetos">
    <comment>Aetos language source code</comment>
    <glob pattern="*.aetos"/>
    <sub-class-of type="text/plain"/>
  </mime-type>
</mime-info>
EOF

# 2. Обновляем базу MIME-типов
update-mime-database ~/.local/share/mime

echo "MIME-тип для .aetos файлов создан."

# 3. Создаем простую ассоциацию с текстовым редактором
echo "Создаем ассоциацию с текстовым редактором..."

cat > ~/.local/share/applications/aetos.desktop << 'EOF'
[Desktop Entry]
Name=Aetos File
Comment=Edit Aetos source code
Exec=xdg-open %f
Terminal=false
Type=Application
NoDisplay=true
MimeType=text/x-aetos;
EOF

# 4. Создаем скрипт для запуска .aetos файлов
mkdir -p ~/.local/bin

cat > ~/.local/bin/aetos-run-script << 'EOF'
#!/bin/bash
# Скрипт для запуска .aetos файлов

if [ -z "$1" ]; then
    echo "Использование: aetos-run-script <файл.aetos>"
    exit 1
fi

if [ -f "$1" ]; then
    echo "Запуск Aetos файла: $1"
    aetosc run "$1"
else
    echo "Файл не найден: $1"
    exit 1
fi
EOF

chmod +x ~/.local/bin/aetos-run-script

# 5. Создаем десктоп-файл для запуска
cat > ~/.local/share/applications/aetos-launcher.desktop << 'EOF'
[Desktop Entry]
Name=Aetos Launcher
Comment=Run Aetos programs
Exec=aetos-run-script %f
Icon=aetosc
Terminal=true
Type=Application
Categories=Development;
MimeType=text/x-aetos;
EOF

echo "Ассоциация создана. Файлы .aetos теперь можно запускать двойным кликом."

# 6. Обновляем базу данных .desktop файлов
update-desktop-database ~/.local/share/applications

echo ""
echo "Настройка завершена!"
echo "Файлы .aetos теперь будут распознаваться системой."
echo ""
echo "Чтобы файлы .aetos открывались в IDE, запустите:"
echo "  xdg-mime default aetos-launcher.desktop text/x-aetos"