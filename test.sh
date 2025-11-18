#!/bin/bash

# J3S Music Upload - Automated Test Script
# This script runs comprehensive tests on all upload methods

set -e

echo "=========================================="
echo "J3S Music Upload - Automated Test Suite"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Running: ${test_name}${NC}"
    if eval "$command" > /tmp/test_output.log 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        echo "Error output:"
        tail -20 /tmp/test_output.log
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    echo ""
}

# Test 1: Code compilation check
echo ""
echo "Test 1: Checking code compilation..."
run_test "Cargo check" "cargo check"

# Test 2: Unit tests
echo ""
echo "Test 2: Running unit tests..."
run_test "Unit tests" "cargo test"

# Test 3: Release build
echo ""
echo "Test 3: Building release version..."
run_test "Release build" "cargo build --release"

# Test 4: Check for security issues (if cargo-audit is installed)
echo ""
echo "Test 4: Security audit (optional)..."
if command -v cargo-audit &> /dev/null; then
    run_test "Security audit" "cargo audit"
else
    echo -e "${YELLOW}⊘ SKIPPED (cargo-audit not installed)${NC}"
fi

# Summary
echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "Tests passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests failed: ${RED}${TESTS_FAILED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    echo ""
    echo "What was tested:"
    echo "  ✓ YouTube download handler (URL validation, yt-dlp args, player_client=android)"
    echo "  ✓ Spotify download handler (URL validation, spotdl args)"
    echo "  ✓ File upload handler (path sanitization, extension validation)"
    echo "  ✓ User-specific library paths"
    echo "  ✓ Cross-filesystem file moves (copy+remove pattern)"
    echo "  ✓ Ferric enable/disable toggle"
    echo ""
    echo "Next steps:"
    echo "  1. Add your user to docker group: sudo usermod -aG docker \$USER && newgrp docker"
    echo "  2. Rebuild Docker: docker-compose build"
    echo "  3. Restart services: docker-compose up -d"
    echo "  4. Test uploads through the web UI"
    exit 0
else
    echo -e "${RED}Some tests failed. Please review the errors above.${NC}"
    exit 1
fi
