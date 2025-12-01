#!/usr/bin/env bash

# Orders Workspace Validation Suite
# Runs checks, feature-matrix tests, and build to ensure nothing is broken.
# Usage: ./validate_all.sh

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() {
    echo -e "${BLUE}==== $1 ====${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════╗"
echo "║          Orders Validation Suite             ║"
echo "║      Tests | Checks | Feature Matrix         ║"
echo "╚══════════════════════════════════════════════╝"
echo -e "${NC}"

# Run from repo root
cd "$(dirname "$0")"

start_time=$(date +%s)

failures=()
warns=()

run_required() {
    local desc="$1"
    shift
    print_step "$desc"
    if "$@"; then
        print_success "$desc"
    else
        print_error "$desc failed"
        failures+=("$desc")
    fi
}

run_warn() {
    local desc="$1"
    shift
    print_step "$desc"
    if "$@"; then
        print_success "$desc"
    else
        print_warning "$desc had issues (continuing)"
        warns+=("$desc")
    fi
}

# 1) Code checks
run_required "cargo check (all targets)" cargo check --all-targets
run_warn "cargo clippy (all targets, all features)" cargo clippy --all-targets --all-features -- -D warnings

# 2) Tests (feature matrix)
run_required "orders-types tests" cargo test -p orders-types
run_required "orders-repo tests (memory)" cargo test -p orders-repo
run_required "orders-repo tests (sqlite)" cargo test -p orders-repo --features sqlite
run_required "orders-hex tests" cargo test -p orders-hex
run_required "orders-app tests (sqlite default)" cargo test -p orders-app
run_required "orders-app tests (memory feature)" cargo test -p orders-app --no-default-features --features memory

# 3) Build verification
run_required "release build (sqlite default)" cargo build --release
run_required "release build (memory feature)" cargo build --release --no-default-features --features memory

end_time=$(date +%s)
duration=$((end_time - start_time))
minutes=$((duration / 60))
seconds=$((duration % 60))

echo
echo -e "${BLUE}╔══════════════════════════════════════════════╗"
echo "║               VALIDATION SUMMARY             ║"
echo -e "╚══════════════════════════════════════════════╝${NC}"

if [ ${#failures[@]} -eq 0 ]; then
    print_success "All required steps passed in ${minutes}m ${seconds}s"
else
    print_error "Failures: ${failures[*]}"
    exit 1
fi

if [ ${#warns[@]} -gt 0 ]; then
    print_warning "Warnings: ${warns[*]}"
fi

echo "Steps run:"
echo "  • cargo check, clippy (warn only)"
echo "  • tests: orders-types, orders-repo (memory/sqlite), orders-hex, orders-app (sqlite/memory)"
echo "  • release builds: default sqlite, memory feature"

print_success "Orders workspace is healthy."
