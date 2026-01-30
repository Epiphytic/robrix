#!/bin/bash
# Setup macOS Code Signing for GitHub Actions
# This script exports your Developer ID certificate and configures GitHub secrets
#
# Prerequisites:
# - macOS with Keychain Access
# - Developer ID Application certificate installed in your keychain
# - GitHub CLI (gh) installed and authenticated
# - Repository write access
#
# Usage: ./scripts/setup-macos-signing.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== macOS Code Signing Setup for GitHub Actions ===${NC}"
echo

# Check prerequisites
check_prerequisites() {
	echo -e "${YELLOW}Checking prerequisites...${NC}"

	if [[ "$(uname)" != "Darwin" ]]; then
		echo -e "${RED}Error: This script must be run on macOS${NC}"
		exit 1
	fi

	if ! command -v gh &>/dev/null; then
		echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
		echo "Install with: brew install gh"
		exit 1
	fi

	if ! gh auth status &>/dev/null; then
		echo -e "${RED}Error: GitHub CLI is not authenticated${NC}"
		echo "Run: gh auth login"
		exit 1
	fi

	echo -e "${GREEN}Prerequisites OK${NC}"
	echo
}

# Get repository info
get_repo_info() {
	REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")
	if [[ -z "$REPO" ]]; then
		echo -e "${RED}Error: Could not determine repository. Run from within a git repo.${NC}"
		exit 1
	fi
	echo -e "${BLUE}Repository: ${REPO}${NC}"
	echo
}

# List available Developer ID certificates
list_certificates() {
	echo -e "${YELLOW}Available Developer ID Application certificates:${NC}"
	echo
	security find-identity -v -p codesigning | grep "Developer ID Application" || {
		echo -e "${RED}No Developer ID Application certificates found in keychain${NC}"
		echo "Please install your Developer ID certificate first."
		exit 1
	}
	echo
}

# Export certificate
export_certificate() {
	echo -e "${YELLOW}Enter the Common Name of your Developer ID certificate${NC}"
	echo -e "(e.g., 'Developer ID Application: Your Name (TEAMID)')"
	read -p "> " CERT_NAME

	if [[ -z "$CERT_NAME" ]]; then
		echo -e "${RED}Certificate name cannot be empty${NC}"
		exit 1
	fi

	# Create temp directory
	TEMP_DIR=$(mktemp -d)
	P12_FILE="$TEMP_DIR/developer_id.p12"

	echo
	echo -e "${YELLOW}Enter a password for the exported certificate:${NC}"
	read -s -p "> " CERT_PASSWORD
	echo

	if [[ -z "$CERT_PASSWORD" ]]; then
		echo -e "${RED}Password cannot be empty${NC}"
		rm -rf "$TEMP_DIR"
		exit 1
	fi

	echo -e "${YELLOW}Confirm password:${NC}"
	read -s -p "> " CERT_PASSWORD_CONFIRM
	echo

	if [[ "$CERT_PASSWORD" != "$CERT_PASSWORD_CONFIRM" ]]; then
		echo -e "${RED}Passwords do not match${NC}"
		rm -rf "$TEMP_DIR"
		exit 1
	fi

	echo
	echo -e "${YELLOW}Exporting certificate...${NC}"
	echo -e "(You may be prompted for your keychain password)"
	echo

	# Export the certificate
	security export -t identities -f pkcs12 -k ~/Library/Keychains/login.keychain-db \
		-P "$CERT_PASSWORD" -o "$P12_FILE" 2>/dev/null ||
		security export -t identities -f pkcs12 -k ~/Library/Keychains/login.keychain \
			-P "$CERT_PASSWORD" -o "$P12_FILE" 2>/dev/null || {
		# Try finding and exporting specific certificate
		security find-certificate -c "$CERT_NAME" -p >"$TEMP_DIR/cert.pem" 2>/dev/null
		security find-key -c "$CERT_NAME" -p >"$TEMP_DIR/key.pem" 2>/dev/null

		if [[ -s "$TEMP_DIR/cert.pem" ]] && [[ -s "$TEMP_DIR/key.pem" ]]; then
			openssl pkcs12 -export -out "$P12_FILE" \
				-inkey "$TEMP_DIR/key.pem" \
				-in "$TEMP_DIR/cert.pem" \
				-password "pass:$CERT_PASSWORD"
		else
			echo -e "${RED}Failed to export certificate${NC}"
			echo "Try exporting manually from Keychain Access:"
			echo "1. Open Keychain Access"
			echo "2. Find your Developer ID Application certificate"
			echo "3. Right-click > Export"
			echo "4. Save as .p12 file"
			rm -rf "$TEMP_DIR"
			exit 1
		fi
	}

	if [[ ! -s "$P12_FILE" ]]; then
		echo -e "${RED}Certificate export failed or file is empty${NC}"
		rm -rf "$TEMP_DIR"
		exit 1
	fi

	# Base64 encode
	CERT_BASE64=$(base64 -i "$P12_FILE")

	echo -e "${GREEN}Certificate exported successfully${NC}"
	echo

	# Store for later use
	export EXPORTED_CERT_BASE64="$CERT_BASE64"
	export EXPORTED_CERT_PASSWORD="$CERT_PASSWORD"

	# Cleanup
	rm -rf "$TEMP_DIR"
}

# Get Apple ID credentials for notarization
get_apple_credentials() {
	echo -e "${YELLOW}=== Notarization Setup (Optional) ===${NC}"
	echo "Notarization is required for apps distributed outside the App Store."
	echo
	read -p "Do you want to set up notarization? (y/N) " SETUP_NOTARIZATION

	if [[ "$SETUP_NOTARIZATION" =~ ^[Yy]$ ]]; then
		echo
		echo -e "${YELLOW}Enter your Apple ID (email):${NC}"
		read -p "> " APPLE_ID

		echo
		echo -e "${YELLOW}Enter your App-Specific Password:${NC}"
		echo "(Generate at https://appleid.apple.com/account/manage > App-Specific Passwords)"
		read -s -p "> " APPLE_ID_PWD
		echo

		echo
		echo -e "${YELLOW}Enter your Apple Developer Team ID:${NC}"
		echo "(Found in Apple Developer Portal > Membership)"
		read -p "> " APPLE_TEAM_ID

		export SETUP_NOTARIZATION="yes"
	else
		export SETUP_NOTARIZATION="no"
	fi
	echo
}

# Generate keychain password
generate_keychain_password() {
	KEYCHAIN_PWD=$(openssl rand -base64 32)
	export GENERATED_KEYCHAIN_PWD="$KEYCHAIN_PWD"
}

# Set GitHub secrets
set_github_secrets() {
	echo -e "${YELLOW}=== Setting GitHub Secrets ===${NC}"
	echo -e "Repository: ${BLUE}${REPO}${NC}"
	echo

	echo "Setting MACOS_CERTIFICATE..."
	echo "$EXPORTED_CERT_BASE64" | gh secret set MACOS_CERTIFICATE --repo "$REPO"
	echo -e "${GREEN}✓ MACOS_CERTIFICATE${NC}"

	echo "Setting MACOS_CERTIFICATE_PWD..."
	echo "$EXPORTED_CERT_PASSWORD" | gh secret set MACOS_CERTIFICATE_PWD --repo "$REPO"
	echo -e "${GREEN}✓ MACOS_CERTIFICATE_PWD${NC}"

	echo "Setting MACOS_KEYCHAIN_PWD..."
	echo "$GENERATED_KEYCHAIN_PWD" | gh secret set MACOS_KEYCHAIN_PWD --repo "$REPO"
	echo -e "${GREEN}✓ MACOS_KEYCHAIN_PWD${NC}"

	if [[ "$SETUP_NOTARIZATION" == "yes" ]]; then
		echo "Setting APPLE_ID..."
		echo "$APPLE_ID" | gh secret set APPLE_ID --repo "$REPO"
		echo -e "${GREEN}✓ APPLE_ID${NC}"

		echo "Setting APPLE_ID_PWD..."
		echo "$APPLE_ID_PWD" | gh secret set APPLE_ID_PWD --repo "$REPO"
		echo -e "${GREEN}✓ APPLE_ID_PWD${NC}"

		echo "Setting APPLE_TEAM_ID..."
		echo "$APPLE_TEAM_ID" | gh secret set APPLE_TEAM_ID --repo "$REPO"
		echo -e "${GREEN}✓ APPLE_TEAM_ID${NC}"
	fi

	echo
	echo -e "${GREEN}=== All secrets configured successfully! ===${NC}"
}

# Verify secrets
verify_secrets() {
	echo
	echo -e "${YELLOW}Verifying secrets...${NC}"
	SECRETS=$(gh secret list --repo "$REPO" 2>/dev/null || echo "")

	REQUIRED_SECRETS="MACOS_CERTIFICATE MACOS_CERTIFICATE_PWD MACOS_KEYCHAIN_PWD"

	for secret in $REQUIRED_SECRETS; do
		if echo "$SECRETS" | grep -q "$secret"; then
			echo -e "${GREEN}✓ $secret is set${NC}"
		else
			echo -e "${RED}✗ $secret is missing${NC}"
		fi
	done

	if [[ "$SETUP_NOTARIZATION" == "yes" ]]; then
		NOTARIZATION_SECRETS="APPLE_ID APPLE_ID_PWD APPLE_TEAM_ID"
		for secret in $NOTARIZATION_SECRETS; do
			if echo "$SECRETS" | grep -q "$secret"; then
				echo -e "${GREEN}✓ $secret is set${NC}"
			else
				echo -e "${RED}✗ $secret is missing${NC}"
			fi
		done
	fi
}

# Main flow
main() {
	check_prerequisites
	get_repo_info
	list_certificates
	export_certificate
	get_apple_credentials
	generate_keychain_password
	set_github_secrets
	verify_secrets

	echo
	echo -e "${GREEN}=== Setup Complete! ===${NC}"
	echo
	echo "Your next release build will automatically sign macOS binaries."
	echo "To test, create a new release tag:"
	echo "  git tag v0.1.2 && git push origin v0.1.2"
	echo
}

main "$@"
