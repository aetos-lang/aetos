@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo.
echo ===========================================
echo    Aetos Compiler - Windows Installer
echo ===========================================
echo.

REM Проверка Rust
echo [1/6] Checking for Rust installation...
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
echo [2/6] Updating Rust...
call rustup update

REM Сборка проекта
echo [3/6] Building compiler...
echo This may take a few minutes...
call cargo build --release
if %errorLevel% neq 0 (
    echo [ERROR] Build failed!
    pause
    exit /b 1
)
echo [OK] Compiler built

echo [4/6] Building visual editor...
call cargo build --release --bin aetos-visual-editor
if %errorLevel% neq 0 (
    echo [WARNING] Visual editor build may have issues
)

REM Создание директорий
echo [5/6] Creating installation directories...
set "INSTALL_DIR=%USERPROFILE%\AppData\Local\Aetos"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "EXAMPLES_DIR=%INSTALL_DIR%\examples"
set "START_MENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Aetos"

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%EXAMPLES_DIR%" mkdir "%EXAMPLES_DIR%"
if not exist "%START_MENU%" mkdir "%START_MENU%"

REM Копирование файлов
echo [6/6] Copying files...

REM Компилятор
if exist "target\release\aetosc.exe" (
    copy /Y "target\release\aetosc.exe" "%BIN_DIR%\"
    echo [OK] Compiler copied
) else (
    echo [ERROR] aetosc.exe not found!
    pause
    exit /b 1
)

REM Визуальный редактор
if exist "target\release\aetos-visual-editor.exe" (
    copy /Y "target\release\aetos-visual-editor.exe" "%BIN_DIR%\"
    echo [OK] Visual editor copied
)

REM Примеры
if exist "examples" (
    xcopy /E /I /Y "examples" "%EXAMPLES_DIR%\" >nul
    echo [OK] Examples copied
) else (
    echo Creating default examples...
    
    echo // Hello World example> "%EXAMPLES_DIR%\hello.aetos"
    echo fn main() -^> i32 {>> "%EXAMPLES_DIR%\hello.aetos"
    echo     print_string("Hello, Aetos!");>> "%EXAMPLES_DIR%\hello.aetos"
    echo     0>> "%EXAMPLES_DIR%\hello.aetos"
    echo }>> "%EXAMPLES_DIR%\hello.aetos"
    
    echo [OK] Examples created
)

REM Создание скриптов запуска
echo Creating shortcuts...

REM Компилятор - командная строка
echo @echo off> "%BIN_DIR%\aetosc.cmd"
echo setlocal>> "%BIN_DIR%\aetosc.cmd"
echo "%BIN_DIR%\aetosc.exe" %%*>> "%BIN_DIR%\aetosc.cmd"

REM Визуальный редактор - командная строка
if exist "%BIN_DIR%\aetos-visual-editor.exe" (
    echo @echo off> "%BIN_DIR%\aetos-visual-editor.cmd"
    echo setlocal>> "%BIN_DIR%\aetos-visual-editor.cmd"
    echo "%BIN_DIR%\aetos-visual-editor.exe" %%*>> "%BIN_DIR%\aetos-visual-editor.cmd"
)

REM Создание ярлыков в меню Пуск
echo Creating Start Menu shortcuts...

REM PowerShell для создания ярлыков
set "PS_SCRIPT=%TEMP%\create_shortcuts.ps1"

REM Создание PowerShell скрипта
(
echo $WshShell = New-Object -ComObject WScript.Shell
echo $Shortcut = $WshShell.CreateShortcut("%START_MENU%\Aetos Compiler.lnk")
echo $Shortcut.TargetPath = "%BIN_DIR%\aetosc.cmd"
echo $Shortcut.WorkingDirectory = "%USERPROFILE%"
echo $Shortcut.Description = "Aetos Language Compiler"
echo $Shortcut.Save()
) > "%PS_SCRIPT%"

if exist "%BIN_DIR%\aetos-visual-editor.exe" (
    (
    echo $WshShell = New-Object -ComObject WScript.Shell
    echo $Shortcut = $WshShell.CreateShortcut("%START_MENU%\Aetos Visual Editor.lnk")
    echo $Shortcut.TargetPath = "%BIN_DIR%\aetos-visual-editor.exe"
    echo $Shortcut.WorkingDirectory = "%USERPROFILE%"
    echo $Shortcut.Description = "Aetos Visual Node Editor"
    echo $Shortcut.Save()
    ) >> "%PS_SCRIPT%"
)

REM Запуск PowerShell скрипта
powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%" >nul 2>&1
del "%PS_SCRIPT%"

REM Создание файла обновления
echo Creating update script...
echo @echo off> "%BIN_DIR%\aetos-update.bat"
echo echo Updating Aetos...>> "%BIN_DIR%\aetos-update.bat"
echo echo.>> "%BIN_DIR%\aetos-update.bat"
echo if exist "Cargo.toml" (>> "%BIN_DIR%\aetos-update.bat"
echo     git pull origin main ^|^| echo Could not pull changes>> "%BIN_DIR%\aetos-update.bat"
echo     cargo build --release>> "%BIN_DIR%\aetos-update.bat"
echo     cargo build --release --bin aetos-visual-editor>> "%BIN_DIR%\aetos-update.bat"
echo     copy /Y target\release\aetosc.exe "%BIN_DIR%\" >> "%BIN_DIR%\aetos-update.bat"
echo     copy /Y target\release\aetos-visual-editor.exe "%BIN_DIR%\" 2^>nul ^|^| echo Visual editor not built>> "%BIN_DIR%\aetos-update.bat"
echo     echo Update complete!>> "%BIN_DIR%\aetos-update.bat"
echo ) else (>> "%BIN_DIR%\aetos-update.bat"
echo     echo Please run from Aetos source directory>> "%BIN_DIR%\aetos-update.bat"
echo )>> "%BIN_DIR%\aetos-update.bat"

REM Создание файла удаления
echo Creating uninstall script...
echo @echo off> "%BIN_DIR%\aetos-uninstall.bat"
echo echo Uninstalling Aetos...>> "%BIN_DIR%\aetos-uninstall.bat"
echo echo.>> "%BIN_DIR%\aetos-uninstall.bat"
echo echo Are you sure? (y/N)>> "%BIN_DIR%\aetos-uninstall.bat"
echo set /p choice=>> "%BIN_DIR%\aetos-uninstall.bat"
echo if /i "!choice!"=="y" (>> "%BIN_DIR%\aetos-uninstall.bat"
echo     echo Removing files...>> "%BIN_DIR%\aetos-uninstall.bat"
echo     rmdir /S /Q "%INSTALL_DIR%" >> "%BIN_DIR%\aetos-uninstall.bat"
echo     rmdir /S /Q "%START_MENU%" >> "%BIN_DIR%\aetos-uninstall.bat"
echo     echo Aetos has been uninstalled.>> "%BIN_DIR%\aetos-uninstall.bat"
echo )>> "%BIN_DIR%\aetos-uninstall.bat"

REM Добавление в PATH (требует прав администратора)
echo Adding to PATH...
set "NEW_PATH=%BIN_DIR%;%PATH%"
setx PATH "!NEW_PATH!" >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] Could not add to PATH automatically
    echo Please add %BIN_DIR% to PATH manually:
    echo 1. Open System Properties
    echo 2. Advanced -> Environment Variables
    echo 3. Edit PATH variable
    echo 4. Add: %BIN_DIR%
)

REM Создание конфигурационного файла
echo Creating configuration...
echo # Aetos Configuration File> "%INSTALL_DIR%\config.toml"
echo version = "0.3.0">> "%INSTALL_DIR%\config.toml"
echo install_dir = "%INSTALL_DIR%">> "%INSTALL_DIR%\config.toml"
echo examples_dir = "%EXAMPLES_DIR%">> "%INSTALL_DIR%\config.toml"

echo.
echo ===========================================
echo    Installation Complete!
echo ===========================================
echo.
echo Installed components:
echo   - aetosc.exe               - Compiler
echo   - aetos-visual-editor.exe  - Visual editor (if available)
echo   - aetos-update.bat         - Update utility
echo   - aetos-uninstall.bat      - Uninstaller
echo   - Examples folder          - Sample programs
echo.
echo Start Menu shortcuts created in:
echo   "%START_MENU%"
echo.
echo Quick start:
echo   1. Open Command Prompt
echo   2. Type: aetosc run "%EXAMPLES_DIR%\hello.aetos"
echo   3. For GUI editor: Start Menu -> Aetos Visual Editor
echo.
echo Notes:
echo   - You may need to restart Command Prompt for PATH changes
echo   - If GUI editor doesn't work, install Visual C++ Redistributable
echo.
pause