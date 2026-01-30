#!/usr/bin/env bash
#
# Self-Hosted GitHub Actions Runner Setup Script
# ==============================================
# This script configures a self-hosted runner for the robrix repository.
# Use self-hosted runners for expensive operations like multi-platform builds and releases.
#
# Usage:
#   ./scripts/setup-self-hosted-runner.sh [OPTIONS]
#
# Options:
#   --name NAME        Runner name (default: robrix-runner-<hostname>)
#   --labels LABELS    Comma-separated labels (default: self-hosted,robrix)
#   --work-dir DIR     Work directory (default: _work)
#   --token TOKEN      Runner registration token (or set GITHUB_RUNNER_TOKEN env var)
#   --url URL          Repository URL (default: https://github.com/epiphytic/robrix)
#   --replace          Replace existing runner with same name
#   --unattended       Run setup without prompts
#   --help             Show this help message
#
# Prerequisites:
#   - curl
#   - tar (Linux/macOS) or unzip (Windows)
#   - jq (for token generation via API)
#
# To get a runner token:
#   1. Go to https://github.com/epiphytic/robrix/settings/actions/runners/new
#   2. Copy the token from the configuration command
#   OR
#   3. Use GitHub CLI: gh api repos/epiphytic/robrix/actions/runners/registration-token -X POST --jq '.token'
#

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

REPO_URL="${GITHUB_RUNNER_REPO_URL:-https://github.com/epiphytic/robrix}"
RUNNER_VERSION="${GITHUB_RUNNER_VERSION:-2.321.0}"
RUNNER_NAME="${GITHUB_RUNNER_NAME:-robrix-runner-$(hostname -s)}"
RUNNER_LABELS="${GITHUB_RUNNER_LABELS:-self-hosted,robrix}"
RUNNER_WORK_DIR="${GITHUB_RUNNER_WORK_DIR:-_work}"
RUNNER_TOKEN="${GITHUB_RUNNER_TOKEN:-}"
REPLACE_EXISTING=false
UNATTENDED=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
	echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
	echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
	echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
	echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
	head -40 "$0" | tail -35 | sed 's/^#//' | sed 's/^ //'
	exit 0
}

detect_os() {
	case "$(uname -s)" in
	Linux*) echo "linux" ;;
	Darwin*) echo "osx" ;;
	MINGW* | MSYS* | CYGWIN*) echo "win" ;;
	*) echo "unknown" ;;
	esac
}

detect_arch() {
	case "$(uname -m)" in
	x86_64 | amd64) echo "x64" ;;
	aarch64 | arm64) echo "arm64" ;;
	armv7l) echo "arm" ;;
	*) echo "x64" ;;
	esac
}

get_runner_url() {
	local os="$1"
	local arch="$2"
	local version="$3"

	local ext="tar.gz"
	if [[ "$os" == "win" ]]; then
		ext="zip"
	fi

	echo "https://github.com/actions/runner/releases/download/v${version}/actions-runner-${os}-${arch}-${version}.${ext}"
}

# ============================================================================
# Parse Arguments
# ============================================================================

while [[ $# -gt 0 ]]; do
	case $1 in
	--name)
		RUNNER_NAME="$2"
		shift 2
		;;
	--labels)
		RUNNER_LABELS="$2"
		shift 2
		;;
	--work-dir)
		RUNNER_WORK_DIR="$2"
		shift 2
		;;
	--token)
		RUNNER_TOKEN="$2"
		shift 2
		;;
	--url)
		REPO_URL="$2"
		shift 2
		;;
	--replace)
		REPLACE_EXISTING=true
		shift
		;;
	--unattended)
		UNATTENDED=true
		shift
		;;
	--help | -h)
		show_help
		;;
	*)
		log_error "Unknown option: $1"
		exit 1
		;;
	esac
done

# ============================================================================
# Main Setup
# ============================================================================

main() {
	log_info "GitHub Actions Self-Hosted Runner Setup"
	log_info "========================================"

	# Detect platform
	local os arch
	os=$(detect_os)
	arch=$(detect_arch)

	if [[ "$os" == "unknown" ]]; then
		log_error "Unsupported operating system"
		exit 1
	fi

	log_info "Detected platform: ${os}-${arch}"
	log_info "Runner name: ${RUNNER_NAME}"
	log_info "Runner labels: ${RUNNER_LABELS}"
	log_info "Repository: ${REPO_URL}"

	# Check for token
	if [[ -z "$RUNNER_TOKEN" ]]; then
		log_warn "No runner token provided."
		echo ""
		echo "To get a runner token:"
		echo "  1. Visit: ${REPO_URL}/settings/actions/runners/new"
		echo "  2. Copy the token from the configuration command"
		echo ""
		echo "Or use GitHub CLI:"
		echo "  gh api repos/epiphytic/robrix/actions/runners/registration-token -X POST --jq '.token'"
		echo ""

		if [[ "$UNATTENDED" == true ]]; then
			log_error "Cannot proceed without token in unattended mode"
			exit 1
		fi

		read -rp "Enter runner registration token: " RUNNER_TOKEN

		if [[ -z "$RUNNER_TOKEN" ]]; then
			log_error "Token is required"
			exit 1
		fi
	fi

	# Create runner directory
	local runner_dir="${HOME}/actions-runner-${RUNNER_NAME}"

	if [[ -d "$runner_dir" ]]; then
		if [[ "$REPLACE_EXISTING" == true ]]; then
			log_warn "Removing existing runner directory: ${runner_dir}"
			rm -rf "$runner_dir"
		else
			log_error "Runner directory already exists: ${runner_dir}"
			log_error "Use --replace to remove and recreate"
			exit 1
		fi
	fi

	mkdir -p "$runner_dir"
	cd "$runner_dir"

	# Download runner
	local runner_url
	runner_url=$(get_runner_url "$os" "$arch" "$RUNNER_VERSION")
	local runner_archive="actions-runner.tar.gz"

	if [[ "$os" == "win" ]]; then
		runner_archive="actions-runner.zip"
	fi

	log_info "Downloading runner from: ${runner_url}"
	curl -fsSL -o "$runner_archive" "$runner_url"

	# Extract runner
	log_info "Extracting runner..."
	if [[ "$os" == "win" ]]; then
		unzip -q "$runner_archive"
	else
		tar xzf "$runner_archive"
	fi
	rm "$runner_archive"

	# Configure runner
	log_info "Configuring runner..."

	local config_args=(
		--url "$REPO_URL"
		--token "$RUNNER_TOKEN"
		--name "$RUNNER_NAME"
		--labels "$RUNNER_LABELS"
		--work "$RUNNER_WORK_DIR"
	)

	if [[ "$REPLACE_EXISTING" == true ]]; then
		config_args+=(--replace)
	fi

	if [[ "$UNATTENDED" == true ]]; then
		config_args+=(--unattended)
	fi

	./config.sh "${config_args[@]}"

	log_success "Runner configured successfully!"
	echo ""
	log_info "Runner directory: ${runner_dir}"
	echo ""

	# Print start instructions
	echo "=========================================="
	echo "To start the runner:"
	echo ""
	echo "  Interactive mode:"
	echo "    cd ${runner_dir}"
	echo "    ./run.sh"
	echo ""
	echo "  As a service (Linux/macOS):"
	echo "    cd ${runner_dir}"
	echo "    sudo ./svc.sh install"
	echo "    sudo ./svc.sh start"
	echo ""
	echo "  To check service status:"
	echo "    sudo ./svc.sh status"
	echo ""
	echo "  To uninstall:"
	echo "    sudo ./svc.sh stop"
	echo "    sudo ./svc.sh uninstall"
	echo "    ./config.sh remove --token YOUR_TOKEN"
	echo "=========================================="

	# Optionally start the runner
	if [[ "$UNATTENDED" == false ]]; then
		echo ""
		read -rp "Start runner now? (y/N): " start_now
		if [[ "$start_now" =~ ^[Yy]$ ]]; then
			log_info "Starting runner..."
			./run.sh
		fi
	fi
}

main "$@"
