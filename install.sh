#!/usr/bin/env bash
set -euo pipefail

PREFIX=/usr/local
YES=0
UNINSTALL=0
PM=

usage() {
    cat <<EOF
Usage: ./install.sh [--yes] [--prefix DIR] [--uninstall] [--help]

  --yes       Skip confirmation prompts.
  --prefix    Install directory (default: /usr/local).
  --uninstall Remove the installed binary.
  --help      Show this message.
EOF
}

err() {
    echo "install.sh: $*" >&2
}

abort() {
    err "$@"
    exit 1
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --yes)
                YES=1
                shift
                ;;
            --prefix)
                if [[ $# -lt 2 ]]; then
                    abort "--prefix requires an argument"
                fi
                PREFIX="$2"
                shift 2
                ;;
            --uninstall)
                UNINSTALL=1
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                usage >&2
                exit 2
                ;;
        esac
    done
}

refuse_root() {
    if [[ $EUID -eq 0 ]]; then
        abort "run as your normal user; the script uses sudo only where needed"
    fi
}

check_repo() {
    if [[ ! -f Makefile ]] || [[ ! -f host/Cargo.toml ]]; then
        abort "run from a full clone"
    fi
}

detect_distro() {
    local id="" id_like=""
    if [[ -f /etc/os-release ]]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        id="$ID"
        id_like="${ID_LIKE:-}"
    fi

    if [[ " $id " == *" debian "* ]] || [[ " $id " == *" ubuntu "* ]]; then
        PM=apt
    elif [[ " $id_like " == *" debian "* ]] || [[ " $id_like " == *" ubuntu "* ]]; then
        PM=apt
    elif [[ " $id " == *" arch "* ]]; then
        PM=pacman
    elif [[ " $id_like " == *" arch "* ]]; then
        PM=pacman
    else
        abort "unsupported distro — follow the manual steps in README.md (Linux section)"
    fi
}

check_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        err "cargo not found. Install Rust with:"
        err "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    if ! cargo --version >/dev/null 2>&1; then
        err "cargo is installed but does not run. Fix with:"
        err "  rustup default stable"
        exit 1
    fi
}

prompt() {
    local response
    if [[ $YES -eq 1 ]]; then
        return 0
    fi
    printf "%s [y/N] " "$1"
    read -r response
    [[ "$response" == "y" || "$response" == "Y" ]]
}

install_deps() {
    local -a cmd
    if [[ "$PM" == "apt" ]]; then
        cmd=(sudo apt install -y build-essential pkg-config nasm libimobiledevice-utils usbmuxd libxkbcommon-dev libwayland-dev libxcb1-dev libxkbcommon-x11-dev)
    else
        cmd=(sudo pacman -S --needed --noconfirm base-devel pkgconf nasm libimobiledevice usbmuxd libxkbcommon wayland libxcb libxkbcommon-x11)
    fi

    echo "Will install dependencies:"
    printf "  %s\n" "${cmd[*]}"
    if ! prompt "Install with sudo $PM?"; then
        abort "dependency install cancelled"
    fi

    echo "+ ${cmd[*]}"
    "${cmd[@]}"
}

build() {
    echo "+ make host-release"
    make host-release
}

install_binary() {
    local -a cmd
    if [[ "$PREFIX" == "$HOME" || "$PREFIX" == "$HOME"/* ]]; then
        cmd=(make install-host PREFIX="$PREFIX")
    else
        cmd=(sudo make install-host PREFIX="$PREFIX")
    fi
    echo "+ ${cmd[*]}"
    "${cmd[@]}"
}

do_uninstall() {
    local target
    target="$PREFIX/bin/ioscpy"
    if [[ ! -e "$target" ]]; then
        echo "nothing installed at $target"
        exit 0
    fi

    local -a cmd
    if [[ "$PREFIX" == "$HOME" || "$PREFIX" == "$HOME"/* ]]; then
        cmd=(make uninstall-host PREFIX="$PREFIX")
    else
        cmd=(sudo make uninstall-host PREFIX="$PREFIX")
    fi
    echo "+ ${cmd[*]}"
    "${cmd[@]}"

    if [[ -e "$target" ]]; then
        err "$target still exists"
    else
        echo "removed $target"
    fi
    echo "Distro packages and usbmuxd service were not removed."
    exit 0
}

enable_usbmuxd() {
    if systemctl is-active --quiet usbmuxd 2>/dev/null; then
        echo "usbmuxd is already running"
        return
    fi

    if ! command -v systemctl >/dev/null 2>&1; then
        err "start usbmuxd manually before connecting a device"
        return
    fi

    local -a cmd=(sudo systemctl enable --now usbmuxd)
    if prompt "Enable and start usbmuxd with systemctl?"; then
        echo "+ ${cmd[*]}"
        "${cmd[@]}"
    else
        err "usbmuxd not started"
    fi
}

verify() {
    local ioscpy_path
    ioscpy_path="$PREFIX/bin/ioscpy"
    echo "+ \"$ioscpy_path\" --version"
    "$ioscpy_path" --version

    if ! command -v idevice_id >/dev/null 2>&1; then
        err "idevice_id not found on PATH; install libimobiledevice-utils or libimobiledevice"
    fi

    echo
    echo "installed: $ioscpy_path"
    echo "plug in the iPhone and run 'idevice_id -l'"
    echo "then run 'ioscpy'"
}

main() {
    parse_args "$@"
    cd "$(dirname "$0")"
    refuse_root
    check_repo

    if [[ $UNINSTALL -eq 1 ]]; then
        do_uninstall
    fi

    detect_distro
    check_rust
    install_deps
    build
    install_binary
    enable_usbmuxd
    verify
}

main "$@"
