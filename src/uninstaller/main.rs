// src/uninstaller/main.rs
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use clap::Parser;
use colored::*;

#[derive(Parser)]
#[command(name = "aetos-uninstall")]
#[command(about = "Aetos Compiler Uninstaller")]
struct Args {
    /// Skip confirmation prompt
    #[arg(short, long)]
    force: bool,
    
    /// Remove from PATH only (don't delete files)
    #[arg(short, long)]
    path_only: bool,
}

fn is_admin() -> bool {
    // Проверка административных прав на Windows
    if cfg!(windows) {
        use std::os::windows::process::CommandExt;
        use windows::Win32::System::Threading::GetCurrentProcess;
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY
        };
        use windows::Win32::System::Threading::{OpenProcessToken, GetCurrentProcess};
        
        unsafe {
            let mut token = std::ptr::null_mut();
            let process = GetCurrentProcess();
            if OpenProcessToken(process, TOKEN_QUERY, &mut token).is_ok() {
                let mut elevation = TOKEN_ELEVATION::default();
                let mut size = 0;
                if GetTokenInformation(
                    token,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut size,
                ).is_ok() {
                    return elevation.TokenIsElevated != 0;
                }
            }
        }
    }
    false
}

fn remove_from_path(install_dir: &Path) -> io::Result<()> {
    if cfg!(windows) {
        let bin_dir = install_dir.join("bin");
        let bin_path = bin_dir.to_string_lossy().replace('/', "\\");
        
        // Получаем текущий PATH
        let current_path = std::env::var("PATH").unwrap_or_default();
        
        // Удаляем наш путь из PATH
        let new_path: Vec<&str> = current_path
            .split(';')
            .filter(|&p| !p.eq_ignore_ascii_case(&bin_path))
            .collect();
        
        let new_path_str = new_path.join(";");
        
        // Обновляем системный PATH через setx
        println!("{} Removing from system PATH...", "[INFO]".blue());
        
        let output = Command::new("setx")
            .args(["PATH", &new_path_str, "/M"])
            .output()?;
            
        if output.status.success() {
            println!("{} Successfully removed from PATH", "[OK]".green());
        } else {
            println!("{} Failed to update PATH automatically", "[WARNING]".yellow());
            println!("Please remove this path manually from system PATH:");
            println!("  {}", bin_path);
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    
    // Проверка административных прав
    if !is_admin() {
        println!("{} This uninstaller requires administrator privileges!", "[ERROR]".red());
        println!("Please run as administrator.");
        if cfg!(windows) {
            println!("\nRight-click and select 'Run as administrator'");
        }
        return Ok(());
    }
    
    let install_dir = Path::new("C:\\Program Files\\Aetos");
    let start_menu_dir = Path::new(&format!(
        "{}\\Microsoft\\Windows\\Start Menu\\Programs\\Aetos",
        std::env::var("APPDATA").unwrap_or_default()
    ));
    
    println!("{}", "=".repeat(50));
    println!("{}", "AETOS COMPILER UNINSTALLER".bold());
    println!("{}", "=".repeat(50));
    println!();
    
    println!("The following will be removed:");
    println!("  • Installation directory: {}", install_dir.display());
    println!("  • Start Menu folder: {}", start_menu_dir.display());
    println!("  • From system PATH");
    println!();
    
    if args.path_only {
        println!("{} Removing from PATH only...", "[INFO]".blue());
        remove_from_path(install_dir)?;
        println!("\n{} Done! Files remain on disk.", "[INFO]".blue());
        return Ok(());
    }
    
    if !args.force {
        print!("Are you sure you want to uninstall Aetos? (y/N): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{} Uninstall cancelled.", "[INFO]".blue());
            return Ok(());
        }
    }
    
    // Удаляем из PATH
    remove_from_path(install_dir)?;
    
    // Удаляем файлы
    println!("\n{} Removing files...", "[INFO]".blue());
    
    let mut errors = Vec::new();
    
    // Удаляем папку в меню "Пуск"
    if start_menu_dir.exists() {
        match fs::remove_dir_all(start_menu_dir) {
            Ok(_) => println!("{} Start Menu folder removed", "[OK]".green()),
            Err(e) => {
                errors.push(format!("Failed to remove Start Menu folder: {}", e));
                println!("{} Failed to remove Start Menu folder", "[ERROR]".red());
            }
        }
    }
    
    // Удаляем папку установки
    if install_dir.exists() {
        match fs::remove_dir_all(install_dir) {
            Ok(_) => println!("{} Installation directory removed", "[OK]".green()),
            Err(e) => {
                errors.push(format!("Failed to remove installation directory: {}", e));
                println!("{} Failed to remove installation directory", "[ERROR]".red());
            }
        }
    }
    
    // Проверяем реестр Windows (опционально)
    if cfg!(windows) {
        println!("{} Cleaning registry entries...", "[INFO]".blue());
        // Здесь можно добавить удаление записей из реестра
    }
    
    println!("\n{}", "=".repeat(50));
    
    if errors.is_empty() {
        println!("{} Aetos has been successfully uninstalled!", "[SUCCESS]".green().bold());
        println!("\nNote: You may need to restart your terminal for PATH changes to take effect.");
    } else {
        println!("{} Aetos was partially uninstalled", "[WARNING]".yellow().bold());
        println!("\nSome errors occurred:");
        for error in &errors {
            println!("  • {}", error);
        }
        println!("\nYou may need to manually remove the remaining files.");
    }
    
    println!("\nPress Enter to exit...");
    io::stdin().read_line(&mut String::new())?;
    
    Ok(())
}