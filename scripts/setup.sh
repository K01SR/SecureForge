#!/usr/bin/env bash
# SecureForge — One-shot Linux setup + run script
# Usage: bash scripts/setup.sh [cli|gui|server|test]
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
die()   { echo -e "${RED}[ERR]${NC}   $*"; exit 1; }
header(){ echo -e "\n${BOLD}${CYAN}== $* ==${NC}\n"; }

MODE="${1:-cli}"
header "SecureForge Setup — mode: $MODE"

# Python venv
if [[ ! -d ".venv" ]]; then
    python3 -m venv .venv
fi
source .venv/bin/activate
pip install -q -r pipeline/requirements.txt
ok "Python pipeline ready"

# Build
case "$MODE" in
    cli|test)
        cargo build --release --package sih149-cli
        ok "CLI built: ./target/release/sih149"
        ;;
    gui)
        cargo install tauri-cli --version "^2.0" --quiet 2>/dev/null || true
        cd src-ui && npm install --silent && cd ..
        ;;
    server)
        cargo build --release --package secureforge-desktop
        ok "Server binary ready"
        ;;
esac

# Run
case "$MODE" in
    cli)
        header "Drive Info"
        ./target/release/sih149 info || warn "lsblk requires Linux"
        header "Test Recovery Demo"
        bash tests/fixtures/create_test_images.sh 2>/dev/null || true
        mkdir -p /tmp/sf_output
        ./target/release/sih149 recover \
            --source tests/fixtures/mixed_with_jpeg.dd \
            --output /tmp/sf_output --types jpg,png,pdf --min-confidence 30 || true
        ok "Output: /tmp/sf_output/"
        ;;
    gui)
        cargo tauri dev
        ;;
    server)
        PORT="${2:-7878}"
        info "Starting on http://localhost:$PORT"
        ./target/release/secureforge-desktop --server --port "$PORT"
        ;;
    test)
        cargo test --workspace
        python3 pipeline/report_gen.py --help > /dev/null && ok "report_gen.py OK"
        python3 pipeline/classify.py   --help > /dev/null && ok "classify.py OK"
        python3 pipeline/timestamp.py  --help > /dev/null && ok "timestamp.py OK"
        ok "All tests passed!"
        ;;
    *)
        die "Usage: bash scripts/setup.sh [cli|gui|server|test]"
        ;;
esac
