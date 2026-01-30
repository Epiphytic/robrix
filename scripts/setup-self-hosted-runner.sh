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
#   --labels LABELS    Comma-separated labels (default: self-hosted,robrix,linux,arm64)
#   --work-dir DIR     Work directory (default: _work)
#   --token TOKEN      Runner registration token (auto-fetched via gh if not provided)
#   --repo OWNER/REPO  Repository (default: epiphytic/robrix)
#   --replace          Replace existing runner with same name
#   --unattended       Run setup without prompts
#   --help             Show this help message
#
# Prerequisites:
#   - gh (GitHub CLI) - for automatic token generation
#   - curl
#   - tar (Linux/macOS) or unzip (Windows)
#
# The script will automatically:
#   1. Detect your platform (OS and architecture)
#   2. Generate a runner registration token via gh CLI
#   3. Download and configure the GitHub Actions runner
#   4. Set up appropriate labels for the runner
#

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

REPO="${GITHUB_RUNNER_REPO:-epiphytic/robrix}"
REPO_URL="https://github.com/${REPO}"
RUNNER_VERSION="${GITHUB_RUNNER_VERSION:-2.321.0}"
RUNNER_NAME="${GITHUB_RUNNER_NAME:-robrix-runner-$(hostname -s)}"
RUNNER_LABELS="${GITHUB_RUNNER_LABELS:-}" # Will be auto-set based on platform
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
	head -35 "$0" | tail -30 | sed 's/^#//' | sed 's/^ //'
	exit 0
}

check_gh_cli() {
	if ! command -v gh &>/dev/null; then
		log_error "GitHub CLI (gh) is not installed."
		echo ""
		echo "Install it from: https://cli.github.com/"
		echo ""
		echo "  macOS:   brew install gh"
		echo "  Linux:   See https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
		echo "  Windows: winget install GitHub.cli"
		echo ""
		exit 1
	fi

	# Check if authenticated
	if ! gh auth status &>/dev/null; then
		log_error "GitHub CLI is not authenticated."
		echo ""
		echo "Run: gh auth login"
		echo ""
		exit 1
	fi
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

get_runner_token() {
	log_info "Fetching runner registration token via gh CLI..."
	local token
	token=$(gh api "repos/${REPO}/actions/runners/registration-token" -X POST --jq '.token' 2>/dev/null | tr -d '\n\r')

	if [[ -z "$token" ]]; then
		log_error "Failed to get runner registration token."
		echo ""
		echo "Make sure you have admin access to the repository: ${REPO}"
		echo "Or provide a token manually with --token"
		echo ""
		exit 1
	fi

	# Return token without any trailing whitespace
	printf '%s' "$token"
}

generate_labels() {
	local os="$1"
	local arch="$2"

	# Base labels
	local labels="self-hosted,robrix"

	# Add OS label
	case "$os" in
	linux) labels="${labels},linux" ;;
	osx) labels="${labels},macos" ;;
	win) labels="${labels},windows" ;;
	esac

	# Add architecture label
	labels="${labels},${arch}"

	echo "$labels"
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
	--repo)
		REPO="$2"
		REPO_URL="https://github.com/${REPO}"
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
	log_info "Repository: ${REPO}"

	# Auto-generate labels if not provided
	if [[ -z "$RUNNER_LABELS" ]]; then
		RUNNER_LABELS=$(generate_labels "$os" "$arch")
	fi

	log_info "Runner name: ${RUNNER_NAME}"
	log_info "Runner labels: ${RUNNER_LABELS}"

	# Get token via gh CLI if not provided
	if [[ -z "$RUNNER_TOKEN" ]]; then
		check_gh_cli
		RUNNER_TOKEN=$(get_runner_token)
		log_success "Runner token obtained successfully"
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
	echo "    ./config.sh remove --token \$(gh api repos/${REPO}/actions/runners/remove-token -X POST --jq '.token')"
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
