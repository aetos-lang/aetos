#!/bin/bash

# Aetos Language Compiler Installer
# Version: 0.3.0
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logo
echo -e "${PURPLE}"
cat << 'EOF'
    ___       __    ______  ____
   /   | ____/ /   / __ ) \/ / /
  / /| |/ __  /   / __  |\  / / 
 / ___ / /_/ /   / /_/ / / / /  
/_/  |_\__,_/   /_____/ /_/_/   
                                 
EOF
echo -e "${NC}"
echo -e "${CYAN}Aetos Language Compiler & Visual Editor${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo -e "${YELLOW}⚠  Warning: Running as root is not recommended.${NC}"
    echo -e "${YELLOW}   Consider running as a regular user.${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 1
    fi
fi

# Check for Rust
echo -e "${BLUE}🔍 Checking for Rust installation...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed.${NC}"
    echo "Installing Rust via rustup..."
    
    # Download and run rustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Add cargo to PATH
    source "$HOME/.cargo/env"
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Failed to install Rust. Please install manually:${NC}"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Rust installed successfully.${NC}"
else
    echo -e "${GREEN}✅ Rust is already installed.${NC}"
fi

# Update Rust if needed
echo -e "${BLUE}🔄 Updating Rust toolchain...${NC}"
rustup update

# Install system dependencies for visual editor (Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${BLUE}📦 Installing system dependencies for visual editor...${NC}"
    
    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        sudo apt-get update
        sudo apt-get install -y \
            libxcb-render0-dev \
            libxcb-shape0-dev \
            libxcb-xfixes0-dev \
            libxkbcommon-dev \
            libssl-dev \
            pkg-config \
            build-essential
    elif command -v dnf &> /dev/null; then
        # Fedora
        sudo dnf install -y \
            libxcb-devel \
            libxkbcommon-devel \
            openssl-devel \
            gcc-c++
    elif command -v pacman &> /dev/null; then
        # Arch
        sudo pacman -S --noconfirm \
            libxcb \
            xkbcommon \
            openssl \
            gcc \
            pkgconf
    elif command -v zypper &> /dev/null; then
        # openSUSE
        sudo zypper install -y \
            libxcb-devel \
            xkbcommon-devel \
            libopenssl-devel \
            gcc-c++
    else
        echo -e "${YELLOW}⚠  Could not detect package manager.${NC}"
        echo -e "${YELLOW}   Please install these dependencies manually:${NC}"
        echo "   - libxcb"
        echo "   - libxkbcommon"
        echo "   - openssl"
        echo "   - pkg-config"
    fi
fi

# Build the project
echo -e "${BLUE}🏗️  Building Aetos compiler...${NC}"
echo "This may take a few minutes..."

if cargo build --release 2>&1 | tee build.log; then
    echo -e "${GREEN}✅ Compiler build successful.${NC}"
else
    echo -e "${RED}❌ Compiler build failed. See build.log for details.${NC}"
    exit 1
fi

# Build visual editor
echo -e "${BLUE}🎨 Building visual editor...${NC}"
if cargo build --release --bin aetos-visual-editor 2>&1 | tee -a build.log; then
    echo -e "${GREEN}✅ Visual editor build successful.${NC}"
else
    echo -e "${YELLOW}⚠  Visual editor build may have issues (checking if binary exists)...${NC}"
fi

# Create installation directories
echo -e "${BLUE}📁 Creating installation directories...${NC}"
INSTALL_DIR="${HOME}/.aetos"
BIN_DIR="${HOME}/.local/bin"
EXAMPLES_DIR="${INSTALL_DIR}/examples"
CONFIG_DIR="${INSTALL_DIR}/config"
ASSETS_DIR="${INSTALL_DIR}/assets"
ICONS_DIR="${HOME}/.local/share/icons/hicolor"

mkdir -p "${INSTALL_DIR}"
mkdir -p "${BIN_DIR}"
mkdir -p "${EXAMPLES_DIR}"
mkdir -p "${CONFIG_DIR}"
mkdir -p "${ASSETS_DIR}"
mkdir -p "${ICONS_DIR}/256x256/apps"
mkdir -p "${ICONS_DIR}/128x128/apps"
mkdir -p "${ICONS_DIR}/64x64/apps"
mkdir -p "${ICONS_DIR}/48x48/apps"
mkdir -p "${ICONS_DIR}/32x32/apps"

# Copy binaries
echo -e "${BLUE}📦 Installing binaries...${NC}"

# Main compiler
if [ -f "target/release/aetosc" ]; then
    cp target/release/aetosc "${BIN_DIR}/"
    chmod +x "${BIN_DIR}/aetosc"
    echo -e "${GREEN}✅ Compiler installed to ${BIN_DIR}/aetosc${NC}"
else
    echo -e "${RED}❌ Compiler binary not found at target/release/aetosc${NC}"
    exit 1
fi

# Visual editor
if [ -f "target/release/aetos-visual-editor" ]; then
    cp target/release/aetos-visual-editor "${BIN_DIR}/"
    chmod +x "${BIN_DIR}/aetos-visual-editor"
    echo -e "${GREEN}✅ Visual editor installed to ${BIN_DIR}/aetos-visual-editor${NC}"
else
    echo -e "${YELLOW}⚠  Visual editor binary not found. Skipping...${NC}"
fi

# Create icons
echo -e "${BLUE}🎨 Creating icons...${NC}"

# Compiler icon (iconfile.png)
cat > "${ASSETS_DIR}/iconfile.png.base64" << 'EOF'
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
QNL8qBWWAAAAAElFTkSuQmCC
EOF

base64 -d "${ASSETS_DIR}/iconfile.png.base64" > "${ASSETS_DIR}/iconfile.png"
rm "${ASSETS_DIR}/iconfile.png.base64"

# Visual editor icon (icon.png)
cat > "${ASSETS_DIR}/icon.png.base64" << 'EOF'
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
EOF

base64 -d "${ASSETS_DIR}/icon.png.base64" > "${ASSETS_DIR}/icon.png"
rm "${ASSETS_DIR}/icon.png.base64"

# Copy icons to system directories
cp "${ASSETS_DIR}/iconfile.png" "${ICONS_DIR}/256x256/apps/aetosc.png"
cp "${ASSETS_DIR}/iconfile.png" "${ICONS_DIR}/128x128/apps/aetosc.png"
cp "${ASSETS_DIR}/iconfile.png" "${ICONS_DIR}/64x64/apps/aetosc.png"
cp "${ASSETS_DIR}/iconfile.png" "${ICONS_DIR}/48x48/apps/aetosc.png"
cp "${ASSETS_DIR}/iconfile.png" "${ICONS_DIR}/32x32/apps/aetosc.png"

if [ -f "${BIN_DIR}/aetos-visual-editor" ]; then
    cp "${ASSETS_DIR}/icon.png" "${ICONS_DIR}/256x256/apps/aetos-visual-editor.png"
    cp "${ASSETS_DIR}/icon.png" "${ICONS_DIR}/128x128/apps/aetos-visual-editor.png"
    cp "${ASSETS_DIR}/icon.png" "${ICONS_DIR}/64x64/apps/aetos-visual-editor.png"
    cp "${ASSETS_DIR}/icon.png" "${ICONS_DIR}/48x48/apps/aetos-visual-editor.png"
    cp "${ASSETS_DIR}/icon.png" "${ICONS_DIR}/32x32/apps/aetos-visual-editor.png"
fi

echo -e "${GREEN}✅ Icons created and installed.${NC}"

# Copy examples
echo -e "${BLUE}📚 Installing examples...${NC}"
if [ -d "examples" ]; then
    cp -r examples/* "${EXAMPLES_DIR}/" 2>/dev/null || true
    echo -e "${GREEN}✅ Examples installed to ${EXAMPLES_DIR}/${NC}"
else
    # Create basic examples
    cat > "${EXAMPLES_DIR}/hello.aetos" << 'EOF'
// Hello World example
fn main() -> i32 {
    print_string("Hello, Aetos!");
    0
}
EOF

    cat > "${EXAMPLES_DIR}/calculator.aetos" << 'EOF'
// Simple calculator
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() -> i32 {
    let x: i32 = 5;
    let y: i32 = 3;
    let result: i32 = add(x, y);
    print_i32(result);
    0
}
EOF

    cat > "${EXAMPLES_DIR}/graphics.aetos" << 'EOF'
// Graphics demo
fn main() -> i32 {
    init_graphics(800, 600, "AetOS Demo");
    
    clear_screen(30, 30, 60);
    
    // Draw a red circle
    draw_circle(400, 300, 50, 255, 0, 0);
    
    // Draw a green rectangle
    draw_rect(200, 200, 100, 100, 0, 255, 0);
    
    // Draw a white line
    draw_line(100, 100, 700, 500, 255, 255, 255);
    
    render();
    sleep(3000);
    
    0
}
EOF
    echo -e "${GREEN}✅ Basic examples created.${NC}"
fi

# Create configuration
echo -e "${BLUE}⚙️  Creating configuration...${NC}"
cat > "${INSTALL_DIR}/config.toml" << EOF
# Aetos Configuration File
# Version: 0.3.0

[general]
name = "Aetos Compiler"
version = "0.3.0"
author = "Aetos Team"

[compiler]
default_width = 800
default_height = 600
optimization = true
debug_info = false

[paths]
home = "${INSTALL_DIR}"
examples = "${EXAMPLES_DIR}"
binaries = "${BIN_DIR}"

[editor]
theme = "dark"
font_size = 14
tab_size = 4

[features]
graphics = true
wasm = false
llvm = false
visual_editor = true
EOF

# Create update script
echo -e "${BLUE}📝 Creating update script...${NC}"
cat > "${BIN_DIR}/aetos-update" << 'EOF'
#!/bin/bash
set -e

echo "🔄 Updating Aetos compiler..."

# Check if we're in the aetos directory
if [ -f "Cargo.toml" ] && grep -q "aetos-compiler" "Cargo.toml"; then
    echo "📥 Pulling latest changes..."
    git pull origin main || echo "⚠  Could not pull changes, continuing with local build..."
    
    echo "🏗️  Rebuilding compiler..."
    cargo build --release
    
    echo "🏗️  Rebuilding visual editor..."
    cargo build --release --bin aetos-visual-editor 2>/dev/null || echo "⚠  Visual editor build failed"
    
    echo "📦 Reinstalling..."
    cp target/release/aetosc "${HOME}/.local/bin/" 2>/dev/null || true
    cp target/release/aetos-visual-editor "${HOME}/.local/bin/" 2>/dev/null || true
    
    echo "✅ Update complete!"
else
    # Try to find the aetos directory
    if [ -d "${HOME}/.aetos/source" ]; then
        cd "${HOME}/.aetos/source"
        git pull origin main
        cargo build --release
        cargo build --release --bin aetos-visual-editor
        cp target/release/aetosc "${HOME}/.local/bin/"
        cp target/release/aetos-visual-editor "${HOME}/.local/bin/" 2>/dev/null || true
        echo "✅ Update complete!"
    else
        echo "❌ Could not find Aetos source directory."
        echo "   Please run the installer again or clone the repository manually:"
        echo "   git clone https://github.com/aetos-lang/aetos.git"
        exit 1
    fi
fi
EOF

chmod +x "${BIN_DIR}/aetos-update"

# Create uninstall script
echo -e "${BLUE}🗑️  Creating uninstall script...${NC}"
cat > "${BIN_DIR}/aetos-uninstall" << 'EOF'
#!/bin/bash
set -e

echo "🗑️  Uninstalling Aetos compiler..."

# Remove binaries
rm -f "${HOME}/.local/bin/aetosc"
rm -f "${HOME}/.local/bin/aetos-visual-editor" 2>/dev/null || true
rm -f "${HOME}/.local/bin/aetos-update"
rm -f "${HOME}/.local/bin/aetos-uninstall"

# Remove configuration
rm -rf "${HOME}/.aetos"

# Remove icons
rm -f "${HOME}/.local/share/icons/hicolor/256x256/apps/aetosc.png"
rm -f "${HOME}/.local/share/icons/hicolor/128x128/apps/aetosc.png"
rm -f "${HOME}/.local/share/icons/hicolor/64x64/apps/aetosc.png"
rm -f "${HOME}/.local/share/icons/hicolor/48x48/apps/aetosc.png"
rm -f "${HOME}/.local/share/icons/hicolor/32x32/apps/aetosc.png"
rm -f "${HOME}/.local/share/icons/hicolor/256x256/apps/aetos-visual-editor.png" 2>/dev/null || true
rm -f "${HOME}/.local/share/icons/hicolor/128x128/apps/aetos-visual-editor.png" 2>/dev/null || true
rm -f "${HOME}/.local/share/icons/hicolor/64x64/apps/aetos-visual-editor.png" 2>/dev/null || true
rm -f "${HOME}/.local/share/icons/hicolor/48x48/apps/aetos-visual-editor.png" 2>/dev/null || true
rm -f "${HOME}/.local/share/icons/hicolor/32x32/apps/aetos-visual-editor.png" 2>/dev/null || true

# Remove desktop files
rm -f "${HOME}/.local/share/applications/aetosc.desktop" 2>/dev/null || true
rm -f "${HOME}/.local/share/applications/aetos-visual-editor.desktop" 2>/dev/null || true

# Remove from shell config
for rc_file in ".bashrc" ".zshrc" ".profile"; do
    if [ -f "${HOME}/${rc_file}" ]; then
        sed -i '/# Aetos compiler/d' "${HOME}/${rc_file}"
        sed -i '/export PATH.*.local.bin.*aetos/d' "${HOME}/${rc_file}"
        sed -i '/alias aetos-/d' "${HOME}/${rc_file}"
        sed -i '/source .*.aetos.*completion/d' "${HOME}/${rc_file}"
    fi
done

echo "✅ Aetos has been uninstalled."
echo "   You may want to remove the source directory if you cloned it manually."
EOF

chmod +x "${BIN_DIR}/aetos-uninstall"

# Create bash completion
echo -e "${BLUE}✨ Creating bash completion...${NC}"
cat > "${INSTALL_DIR}/completion.bash" << 'EOF'
# Aetos compiler bash completion
_aetosc_completion() {
    local cur prev words cword
    _init_completion || return

    case $prev in
        aetosc)
            COMPREPLY=($(compgen -W "run graphics compile check ide help version" -- "$cur"))
            return
            ;;
        run|graphics|compile|check)
            COMPREPLY=($(compgen -f -X '!*.aetos' -- "$cur"))
            return
            ;;
        -W|--width|-H|--height)
            COMPREPLY=($(compgen -W "320 640 800 1024 1280 1920" -- "$cur"))
            return
            ;;
    esac

    if [[ $cur == -* ]]; then
        COMPREPLY=($(compgen -W "-h --help -v --version -W --width -H --height" -- "$cur"))
    fi
}

complete -F _aetosc_completion aetosc

# Aliases for convenience
alias aetos-run='aetosc run'
alias aetos-graphics='aetosc graphics'
alias aetos-check='aetosc check'
alias aetos-ide='aetosc ide'
alias aetos-help='echo "Aetos commands: aetosc, aetos-update, aetos-uninstall"'
EOF

# Create desktop files
echo -e "${BLUE}🖥️  Creating desktop integration...${NC}"
if [ -d "${HOME}/.local/share/applications" ] && [ -n "$DISPLAY" ]; then
    # Compiler desktop file
    cat > "${HOME}/.local/share/applications/aetosc.desktop" << EOF
[Desktop Entry]
Name=Aetos Compiler
Comment=Command-line compiler for Aetos language
Exec=x-terminal-emulator -e "aetosc ide"
Icon=aetosc
Terminal=true
Type=Application
Categories=Development;Utility;
Keywords=compiler;programming;language;terminal;
EOF
    
    # Visual editor desktop file (if installed)
    if [ -f "${BIN_DIR}/aetos-visual-editor" ]; then
        cat > "${HOME}/.local/share/applications/aetos-visual-editor.desktop" << EOF
[Desktop Entry]
Name=Aetos Visual Editor
Comment=Visual programming editor for Aetos language
Exec=aetos-visual-editor
Icon=aetos-visual-editor
Terminal=false
Type=Application
Categories=Development;IDE;
Keywords=aetos;visual;programming;editor;node;
StartupWMClass=aetos-visual-editor
EOF
    fi
    
    echo -e "${GREEN}✅ Desktop files created.${NC}"
    
    # Update icon cache
    if command -v gtk-update-icon-cache &> /dev/null; then
        gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor"
    fi
fi

# Add to PATH if not already
echo -e "${BLUE}🔧 Setting up environment...${NC}"
SHELL_RC="${HOME}/.bashrc"
if [ -n "${ZSH_VERSION}" ]; then
    SHELL_RC="${HOME}/.zshrc"
fi

if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo -e "${YELLOW}📝 Adding ${BIN_DIR} to PATH in ${SHELL_RC}${NC}"
    
    # Add Aetos section to shell rc
    cat >> "${SHELL_RC}" << EOF

# Aetos compiler
export PATH="${BIN_DIR}:\$PATH"
if [ -f "${INSTALL_DIR}/completion.bash" ]; then
    source "${INSTALL_DIR}/completion.bash"
fi
EOF
    
    # Also add to .profile for login shells
    if [ -f "${HOME}/.profile" ] && ! grep -q "${BIN_DIR}" "${HOME}/.profile"; then
        echo "export PATH=\"${BIN_DIR}:\$PATH\"" >> "${HOME}/.profile"
    fi
    
    echo -e "${YELLOW}⚠  Please restart your terminal or run:${NC}"
    echo -e "${YELLOW}   source ${SHELL_RC}${NC}"
else
    echo -e "${GREEN}✅ ${BIN_DIR} is already in PATH.${NC}"
fi

# Create welcome message
cat > "${INSTALL_DIR}/welcome.txt" << EOF
╔════════════════════════════════════════════════╗
║        Aetos Installation Complete! 🎉        ║
╚════════════════════════════════════════════════╝

📋 Installed components:
  • aetosc              - Main compiler (terminal)
  • aetos-visual-editor - Visual node editor (GUI)
  • aetos-update        - Update utility
  • aetos-uninstall     - Uninstall script
  • Examples            - Sample Aetos programs
  • Icons               - Desktop icons for both apps

🚀 Quick start:
  # Run examples in terminal
  aetosc run ${EXAMPLES_DIR}/hello.aetos
  aetosc graphics ${EXAMPLES_DIR}/graphics.aetos
  
  # Start visual editor (GUI)
  aetos-visual-editor
  
  # Interactive terminal IDE
  aetosc ide

🛠️  Useful commands:
  aetos-update          - Update to latest version
  aetos-uninstall       - Remove Aetos completely
  aetosc help           - Show compiler help
  aetos-help            - Quick command reference

📖 Documentation:
  Examples folder:      ${EXAMPLES_DIR}/
  Config file:          ${INSTALL_DIR}/config.toml
  Icons:                ${INSTALL_DIR}/assets/

🖥️  Desktop Integration:
  • Terminal compiler appears as "Aetos Compiler"
  • Visual editor appears as "Aetos Visual Editor"
  • Both have custom icons

Need help? Issues? Visit: https://github.com/aetos-lang/aetos
EOF

# Show welcome message
echo ""
cat "${INSTALL_DIR}/welcome.txt"
echo ""

# Verify installation
echo -e "${BLUE}✅ Verification...${NC}"
if command -v aetosc &> /dev/null; then
    echo -e "${GREEN}✓ aetosc command is available${NC}"
else
    echo -e "${RED}✗ aetosc command not found in PATH${NC}"
fi

if [ -f "${BIN_DIR}/aetosc" ]; then
    echo -e "${GREEN}✓ Compiler binary exists${NC}"
else
    echo -e "${RED}✗ Compiler binary missing${NC}"
fi

if [ -f "${BIN_DIR}/aetos-visual-editor" ]; then
    if command -v aetos-visual-editor &> /dev/null; then
        echo -e "${GREEN}✓ Visual editor command is available${NC}"
    else
        echo -e "${YELLOW}⚠  Visual editor installed but may need PATH update${NC}"
    fi
    echo -e "${GREEN}✓ Visual editor binary exists${NC}"
else
    echo -e "${YELLOW}⚠  Visual editor not installed (build may have failed)${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Installation complete!${NC}"
echo ""
echo -e "${YELLOW}Important:${NC} If commands don't work immediately, restart your terminal or run:"
echo -e "      ${BLUE}source ${SHELL_RC}${NC}"
echo ""
echo -e "${CYAN}Tip:${NC} Look for 'Aetos Compiler' and 'Aetos Visual Editor' in your application menu!"
echo ""
echo -e "${PURPLE}Happy coding with Aetos! 💻🎨${NC}"