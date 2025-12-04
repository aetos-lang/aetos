@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo.
echo ===========================================
echo    Aetos Compiler - Windows Installer
echo ===========================================
echo.

REM Сохраняем текущую директорию
set "CURRENT_DIR=%~dp0"

REM Проверка административных прав
echo Checking for administrator privileges...
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This installer requires administrator privileges!
    echo Please run as administrator
    echo.
    pause
    exit /b 1
)
echo [OK] Running as administrator
echo.

REM Проверка Rust
echo [1/7] Checking for Rust installation...
where rustc >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Rust is not installed!
    echo.
    echo Please install Rust:
    echo 1. Download rustup-init.exe from https://rustup.rs/
    echo 2. Run it and follow instructions
    echo 3. Restart terminal and run installer again
    echo.
    pause
    exit /b 1
)
echo [OK] Rust is installed

REM Обновление Rust
echo [2/7] Updating Rust...
call rustup update >nul 2>&1

REM Проверка наличия иконки
echo [3/7] Checking for icon file...
if exist "%CURRENT_DIR%icon.png" (
    echo [OK] Found icon.png in current directory
    set "HAS_ICON=true"
) else (
    echo [WARNING] icon.png not found in current directory
    echo File associations will use executable icons
    set "HAS_ICON=false"
)

REM Сборка проекта
echo [4/7] Building compiler...
echo This may take a few minutes...
cd /d "%CURRENT_DIR%"
call cargo build --release
if %errorLevel% neq 0 (
    echo [ERROR] Build failed!
    pause
    exit /b 1
)
echo [OK] Compiler built

echo [5/7] Building visual editor...
call cargo build --release --bin aetos-visual-editor >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] Visual editor build may have issues
)

echo [5b/7] Building uninstaller...
call cargo build --release --bin aetos-uninstall >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] Uninstaller build may have issues
)

REM Создание директорий в Program Files
echo [6/7] Creating installation directories...
set "INSTALL_DIR=C:\Program Files\Aetos"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "EXAMPLES_DIR=%INSTALL_DIR%\examples"
set "ASSETS_DIR=%INSTALL_DIR%\assets"
set "START_MENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Aetos"
set "DESKTOP_LINKS=%PUBLIC%\Desktop"

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%EXAMPLES_DIR%" mkdir "%EXAMPLES_DIR%"
if not exist "%ASSETS_DIR%" mkdir "%ASSETS_DIR%"
if not exist "%START_MENU%" mkdir "%START_MENU%"

REM Копирование файлов
echo [7/7] Copying files...

REM Компилятор
if exist "target\release\aetosc.exe" (
    copy "target\release\aetosc.exe" "%BIN_DIR%\" >nul
    echo [OK] Compiler copied
) else (
    echo [ERROR] aetosc.exe not found!
    pause
    exit /b 1
)

REM Визуальный редактор
if exist "target\release\aetos-visual-editor.exe" (
    copy "target\release\aetos-visual-editor.exe" "%BIN_DIR%\" >nul
    echo [OK] Visual editor copied
)

REM Утилита удаления
if exist "target\release\aetos-uninstall.exe" (
    copy "target\release\aetos-uninstall.exe" "%BIN_DIR%\" >nul
    echo [OK] Uninstaller copied
)

REM Копирование иконки если она существует
if "%HAS_ICON%"=="true" (
    copy "%CURRENT_DIR%icon.png" "%ASSETS_DIR%\file_icon.png" >nul
    echo [OK] Icon file copied to assets
)

echo Creating shortcuts...

REM Ярлык для командной строки
(
echo @echo off
echo "%~dp0aetosc.exe" %%*
) > "%BIN_DIR%\aetosc.cmd"

REM Ярлык визуального редактора для командной строки
if exist "%BIN_DIR%\aetos-visual-editor.exe" (
    (
    echo @echo off
    echo "%~dp0aetos-visual-editor.exe" %%*
    ) > "%BIN_DIR%\aetos-visual-editor.cmd"
)

REM Создание ярлыков в меню "Пуск"
echo Creating Start Menu shortcuts...

REM Создание ярлыков с использованием PowerShell
where powershell >nul 2>&1
if %errorLevel% equ 0 (
    REM Ярлык для компилятора через командную строку
    powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%START_MENU%\Aetos Compiler.lnk');$s.TargetPath='cmd.exe';$s.Arguments='/K aetosc --help';$s.WorkingDirectory='%USERPROFILE%';$s.IconLocation='C:\Windows\System32\cmd.exe,0';$s.Description='Launch Aetos Compiler';$s.Save()"
    
    REM Ярлык для визуального редактора
    if exist "%BIN_DIR%\aetos-visual-editor.exe" (
        REM Используем иконку из файла если она существует
        if "%HAS_ICON%"=="true" (
            powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%START_MENU%\Aetos Visual Editor.lnk');$s.TargetPath='%BIN_DIR%\aetos-visual-editor.exe';$s.WorkingDirectory='%USERPROFILE%';$s.IconLocation='%ASSETS_DIR%\file_icon.png';$s.Description='Aetos Visual Development Environment';$s.Save()"
        ) else (
            powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%START_MENU%\Aetos Visual Editor.lnk');$s.TargetPath='%BIN_DIR%\aetos-visual-editor.exe';$s.WorkingDirectory='%USERPROFILE%';$s.IconLocation='%BIN_DIR%\aetos-visual-editor.exe,0';$s.Description='Aetos Visual Development Environment';$s.Save()"
        )
        echo [OK] Visual editor shortcut created
    )
    
    REM Ярлык для документации
    powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%START_MENU%\Documentation.lnk');$s.TargetPath='%INSTALL_DIR%\README.txt';$s.WorkingDirectory='%INSTALL_DIR%';$s.IconLocation='C:\Windows\System32\shell32.dll,23';$s.Description='Aetos Documentation';$s.Save()"
    
    REM Ярлык для папки с примерами
    powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%START_MENU%\Examples.lnk');$s.TargetPath='%EXAMPLES_DIR%';$s.WorkingDirectory='%EXAMPLES_DIR%';$s.IconLocation='C:\Windows\System32\shell32.dll,4';$s.Description='Aetos Examples';$s.Save()"
    
    REM Ярлык для удаления
    powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%START_MENU%\Uninstall Aetos.lnk');$s.TargetPath='%BIN_DIR%\aetos-uninstall.exe';$s.WorkingDirectory='%USERPROFILE%';$s.Arguments='--force';$s.IconLocation='C:\Windows\System32\shell32.dll,240';$s.Description='Uninstall Aetos Compiler';$s.Save()"
    
    echo [OK] Start Menu shortcuts created
    
    REM Создание ярлыка на рабочем столе для удаления
    powershell -Command "$s=(New-Object -COM WScript.Shell).CreateShortcut('%DESKTOP_LINKS%\Uninstall Aetos.lnk');$s.TargetPath='%BIN_DIR%\aetos-uninstall.exe';$s.WorkingDirectory='%USERPROFILE%';$s.IconLocation='C:\Windows\System32\shell32.dll,240';$s.Description='Uninstall Aetos Compiler';$s.Save()"
    echo [OK] Desktop shortcut created
    
) else (
    echo [WARNING] PowerShell not found. Creating minimal shortcuts...
    
    REM Альтернативный способ создания ярлыков без PowerShell
    if exist "%BIN_DIR%\aetos-visual-editor.exe" (
        (
        echo @echo off
        echo start "" "%BIN_DIR%\aetos-visual-editor.exe"
        ) > "%START_MENU%\Aetos Visual Editor.cmd"
    )
    
    (
    echo @echo off
    echo start notepad "%INSTALL_DIR%\README.txt"
    ) > "%START_MENU%\Documentation.cmd"
    
    (
    echo @echo off
    echo explorer "%EXAMPLES_DIR%"
    ) > "%START_MENU%\Examples.cmd"
    
    if exist "%BIN_DIR%\aetos-uninstall.exe" (
        (
        echo @echo off
        echo "%BIN_DIR%\aetos-uninstall.exe"
        pause
        ) > "%START_MENU%\Uninstall Aetos.cmd"
    )
    
    echo [OK] Command shortcuts created
)

REM Создание ассоциации файлов .aetos
echo Creating file association for .aetos files...

REM Создаем запись в реестре для расширения .aetos
reg add "HKCR\.aetos" /ve /t REG_SZ /d "Aetos.Source.File" /f >nul 2>&1

REM Создаем тип файла с описанием и иконкой
reg add "HKCR\Aetos.Source.File" /ve /t REG_SZ /d "Aetos Source File" /f >nul 2>&1

REM Устанавливаем иконку для файлов .aetos
if "%HAS_ICON%"=="true" (
    REM Преобразуем PNG в ICO с помощью PowerShell
    where powershell >nul 2>&1
    if %errorLevel% equ 0 (
        echo Converting PNG to ICO for file association...
        powershell -Command "$img = [System.Drawing.Image]::FromFile('%ASSETS_DIR%\file_icon.png');$stream = New-Object System.IO.MemoryStream;$img.Save($stream, [System.Drawing.Imaging.ImageFormat]::Icon);[System.IO.File]::WriteAllBytes('%ASSETS_DIR%\file_icon.ico', $stream.ToArray());$img.Dispose()" >nul 2>&1
        
        if exist "%ASSETS_DIR%\file_icon.ico" (
            reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%ASSETS_DIR%\file_icon.ico" /f >nul 2>&1
            echo [OK] Set custom icon for .aetos files
        ) else (
            echo [WARNING] Could not convert PNG to ICO, using executable icon
            if exist "%BIN_DIR%\aetos-visual-editor.exe" (
                reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%BIN_DIR%\aetos-visual-editor.exe,0" /f >nul 2>&1
            ) else if exist "%BIN_DIR%\aetosc.exe" (
                reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%BIN_DIR%\aetosc.exe,0" /f >nul 2>&1
            )
        )
    ) else (
        echo [WARNING] PowerShell not found for PNG to ICO conversion, using executable icon
        if exist "%BIN_DIR%\aetos-visual-editor.exe" (
            reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%BIN_DIR%\aetos-visual-editor.exe,0" /f >nul 2>&1
        ) else if exist "%BIN_DIR%\aetosc.exe" (
            reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%BIN_DIR%\aetosc.exe,0" /f >nul 2>&1
        )
    )
) else (
    echo [INFO] Using executable icon for .aetos files
    if exist "%BIN_DIR%\aetos-visual-editor.exe" (
        reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%BIN_DIR%\aetos-visual-editor.exe,0" /f >nul 2>&1
    ) else if exist "%BIN_DIR%\aetosc.exe" (
        reg add "HKCR\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%BIN_DIR%\aetosc.exe,0" /f >nul 2>&1
    )
)

REM Основное действие по двойному клику - открыть в визуальном редакторе
if exist "%BIN_DIR%\aetos-visual-editor.exe" (
    reg add "HKCR\Aetos.Source.File\shell\open\command" /ve /t REG_SZ /d "\"%BIN_DIR%\aetos-visual-editor.exe\" \"%%1\"" /f >nul 2>&1
    echo [OK] Associated .aetos files with visual editor
) else (
    REM Если визуального редактора нет, открываем в блокноте
    reg add "HKCR\Aetos.Source.File\shell\open\command" /ve /t REG_SZ /d "notepad.exe \"%%1\"" /f >nul 2>&1
    echo [OK] Associated .aetos files with Notepad
)

REM Добавляем контекстное меню для .aetos файлов

REM 1. Открыть в терминальном IDE
reg add "HKCR\Aetos.Source.File\shell\open_in_terminal" /ve /t REG_SZ /d "Open in Terminal IDE" /f >nul 2>&1
reg add "HKCR\Aetos.Source.File\shell\open_in_terminal\command" /ve /t REG_SZ /d "cmd.exe /k \"aetosc ide \"%%1\"\"" /f >nul 2>&1

REM 2. Компилировать файл
reg add "HKCR\Aetos.Source.File\shell\compile" /ve /t REG_SZ /d "Compile" /f >nul 2>&1
reg add "HKCR\Aetos.Source.File\shell\compile\command" /ve /t REG_SZ /d "cmd.exe /k \"aetosc compile \"%%1\" && pause\"" /f >nul 2>&1

REM 3. Запустить файл
reg add "HKCR\Aetos.Source.File\shell\run" /ve /t REG_SZ /d "Run" /f >nul 2>&1
reg add "HKCR\Aetos.Source.File\shell\run\command" /ve /t REG_SZ /d "cmd.exe /k \"aetosc run \"%%1\" && pause\"" /f >nul 2>&1

REM 4. Проверить синтаксис
reg add "HKCR\Aetos.Source.File\shell\check" /ve /t REG_SZ /d "Check Syntax" /f >nul 2>&1
reg add "HKCR\Aetos.Source.File\shell\check\command" /ve /t REG_SZ /d "cmd.exe /k \"aetosc check \"%%1\" && pause\"" /f >nul 2>&1

REM Создаем запись в CurrentVersion для лучшей интеграции
reg add "HKLM\SOFTWARE\Classes\.aetos" /ve /t REG_SZ /d "Aetos.Source.File" /f >nul 2>&1
reg add "HKLM\SOFTWARE\Classes\Aetos.Source.File" /ve /t REG_SZ /d "Aetos Source File" /f >nul 2>&1

if "%HAS_ICON%"=="true" (
    if exist "%ASSETS_DIR%\file_icon.ico" (
        reg add "HKLM\SOFTWARE\Classes\Aetos.Source.File\DefaultIcon" /ve /t REG_SZ /d "%ASSETS_DIR%\file_icon.ico" /f >nul 2>&1
    )
)

echo [OK] File association created for .aetos files

REM Обновляем проводник Windows
echo Refreshing Windows Explorer...
taskkill /f /im explorer.exe >nul 2>&1
start explorer.exe >nul 2>&1
timeout /t 2 /nobreak >nul

REM Добавление в PATH (системное)
echo Adding to system PATH...
setx /M PATH "%BIN_DIR%;%PATH%" >nul 2>&1
if errorlevel 1 (
    echo [WARNING] Could not add to system PATH automatically
    echo Please add %BIN_DIR% to PATH manually
    echo.
    echo Instructions:
    echo 1. Press Win+R, type "sysdm.cpl"
    echo 2. Go to Advanced tab
    echo 3. Click Environment Variables
    echo 4. Edit PATH system variable
    echo 5. Add: %BIN_DIR%
) else (
    echo [OK] Added to system PATH
)

REM Создание конфигурационного файла
echo Creating configuration...
(
echo # Aetos Configuration File
echo version = "0.3.0"
echo install_dir = "%INSTALL_DIR%"
echo examples_dir = "%EXAMPLES_DIR%"
echo assets_dir = "%ASSETS_DIR%"
echo installed_date = "%date% %time%"
echo file_association_created = true
echo custom_icon_used = %HAS_ICON%
) > "%INSTALL_DIR%\config.toml"

REM Создание README файла
echo Creating README...
(
echo ============================================
echo               AETOS COMPILER
echo ============================================
echo.
echo Installation Directory: %INSTALL_DIR%
echo Installation Date: %date% %time%
echo Custom Icon Used: %HAS_ICON%
echo.
echo QUICK START:
echo -----------
echo 1. Open Command Prompt (as administrator)
echo 2. Run: aetosc --help
echo 3. Try example: aetosc run "%EXAMPLES_DIR%\hello.aetos"
echo.
echo COMPONENTS:
echo ----------
echo - aetosc.exe               : Main compiler
echo - aetos-visual-editor.exe  : Visual development environment
echo - aetos-uninstall.exe      : Advanced uninstaller
echo - Examples folder          : Sample programs
echo - Assets folder            : Icons and resources
echo.
echo FILE ASSOCIATIONS:
echo ------------------
echo • .aetos files are associated with Aetos icon
echo • Double-click: Opens in Aetos Visual Editor
echo • Right-click menu options:
echo   - Open in Terminal IDE
echo   - Compile
echo   - Run
echo   - Check Syntax
echo • Test file created: Desktop\test.aetos
echo.
echo START MENU:
echo ----------
echo Look for "Aetos" in Windows Start Menu
echo.
echo UNINSTALLATION:
echo --------------
echo 1. Run "Uninstall Aetos" from Start Menu
echo 2. Or run: aetos-uninstall.exe
echo    Options:
echo      --force     : Skip confirmation
echo      --path-only : Remove from PATH only
echo.
echo PATH:
echo -----
echo Added to system PATH: %BIN_DIR%
echo.
echo ============================================
) > "%INSTALL_DIR%\README.txt"

REM Копирование примеров, если они существуют
if exist "examples\*" (
    echo Copying examples...
    xcopy "examples" "%EXAMPLES_DIR%\" /E /I /Y >nul
    echo [OK] Examples copied
) else (
    REM Создаем простой пример
    (
    echo // Hello World example
    echo fn main() -> i32 {
    echo     print_string("Hello from Aetos!");
    echo     0
    echo }
    ) > "%EXAMPLES_DIR%\hello.aetos"
    echo [OK] Created example file
)

REM Создание записи в реестре для Add/Remove Programs (опционально)
where reg >nul 2>&1
if %errorLevel% equ 0 (
    echo Creating Windows Programs entry...
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v DisplayName /t REG_SZ /d "Aetos Compiler" /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v DisplayVersion /t REG_SZ /d "0.3.0" /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v Publisher /t REG_SZ /d "Aetos Project" /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v UninstallString /t REG_SZ /d "\"%BIN_DIR%\aetos-uninstall.exe\"" /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v QuietUninstallString /t REG_SZ /d "\"%BIN_DIR%\aetos-uninstall.exe\" --force" /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v InstallLocation /t REG_SZ /d "%INSTALL_DIR%" /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v NoModify /t REG_DWORD /d 1 /f >nul 2>&1
    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v NoRepair /t REG_DWORD /d 1 /f >nul 2>&1
    if "%HAS_ICON%"=="true" (
        reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AetosCompiler" /v DisplayIcon /t REG_SZ /d "%ASSETS_DIR%\file_icon.ico" /f >nul 2>&1
    )
    echo [OK] Windows Programs entry created
)

REM Создаем тестовый файл на рабочем столе
(
echo // Test Aetos file - Double-click to open in Aetos IDE
echo // Right-click for more options: Compile, Run, Check Syntax
echo fn main() -> i32 {
echo     print_string("Hello from Aetos on Windows!");
echo     print_string("File association works!");
echo     print_string("This file uses custom icon: %HAS_ICON%");
echo     0
echo }
) > "%USERPROFILE%\Desktop\test.aetos"

echo [OK] Created test.aetos file on Desktop

echo.
echo ===========================================
echo    Installation Complete!
echo ===========================================
echo.
echo Installed to: %INSTALL_DIR%
echo Start Menu: Aetos folder in Windows Start Menu
echo Desktop: Uninstall shortcut on Public Desktop
echo.
echo Installed components:
echo   - aetosc.exe               - Compiler
echo   - aetos-visual-editor.exe  - Visual editor (if available)
echo   - aetos-uninstall.exe      - Advanced uninstaller
echo   - Examples folder          - Sample programs
echo   - Assets folder            - Icons and resources
echo.
echo FILE ASSOCIATIONS:
if "%HAS_ICON%"=="true" (
    echo   • Custom icon (icon.png) used for .aetos files
    echo   • PNG converted to ICO for Windows compatibility
) else (
    echo   • Executable icon used for .aetos files
)
echo   • Double-click opens in Aetos Visual Editor
echo   • Right-click for compile/run options
echo   • Test file: Desktop\test.aetos
echo.
echo Uninstallation options:
echo   1. Start Menu -> Aetos -> Uninstall Aetos
echo   2. Desktop -> Uninstall Aetos
echo   3. Command line: aetos-uninstall.exe [--force]
echo.
echo Quick start:
echo   1. Open new Command Prompt (as administrator)
echo   2. Type: aetosc --help
echo   3. Try example: aetosc run "%EXAMPLES_DIR%\hello.aetos"
echo.
echo   4. Or use Start Menu: Windows Key -> type "Aetos"
echo.
echo   5. Double-click Desktop\test.aetos to test file association
echo.
echo IMPORTANT: 
echo - Restart Command Prompt for PATH changes to take effect!
echo - Restart Explorer for file associations to take effect!
echo - If icons don't appear immediately, reboot your computer.
echo.
pause