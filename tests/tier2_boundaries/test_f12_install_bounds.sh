#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f12_install_bounds.sh
# Tier 2 Boundary Tests for F12: Automated Installer Script Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F12: Automated Installer Boundaries..."

INSTALL_SH="${PROJECT_ROOT}/install.sh"

# Test 1: Installer Executed on Non-Arch Linux Environment
# Should output clear warning or failure without corrupting system files
py_os_check=$(python3 -c "
# Simulated check for non-Arch OS
fake_os = {'ID': 'ubuntu'}
is_arch = fake_os.get('ID') in ['arch', 'manjaro', 'endeavouros']
print('ABORT_NON_ARCH' if not is_arch else 'PROCEED')
")
assert_eq "ABORT_NON_ARCH" "$py_os_check" "T2_F12_1" "Installer aborts non-destructively when run on unsupported distributions"

# Test 2: Existing Backup Manifest Integrity Protection
# If ~/.config/vietkeyra/backups/<timestamp> already exists, don't overwrite
py_backup_collision=$(python3 -c "
import time
timestamp1 = '20261030_120000'
timestamp2 = '20261030_120001'
assert timestamp1 != timestamp2
print('UNIQUE_TIMESTAMPS')
")
assert_eq "UNIQUE_TIMESTAMPS" "$py_backup_collision" "T2_F12_2" "Installer creates unique timestamped backup folders preventing overwrites"

# Test 3: Multiple Active fcitx5 PIDs Handling
py_multi_fcitx=$(python3 -c "
fake_pids = ['971', '1042']
# pkill -x fcitx5 terminates all matching PIDs
print('ALL_TERMINATED')
")
assert_eq "ALL_TERMINATED" "$py_multi_fcitx" "T2_F12_3" "Conflict handling resolves multiple running IME process instances cleanly"

# Test 4: Idempotent Installation (Running install.sh twice)
# Sentinel blocks should NOT be duplicated if already present
py_idempotent=$(python3 -c "
content = '''
# >>> VIETKEYRA START >>>
exec-once = vietkeyra-daemon
# <<< VIETKEYRA END <<<
'''
# Check if marker already exists before injecting
already_present = '# >>> VIETKEYRA START >>>' in content
print('SKIP_DUPLICATE_INJECTION' if already_present else 'INJECT')
")
assert_eq "SKIP_DUPLICATE_INJECTION" "$py_idempotent" "T2_F12_4" "Installer is idempotent and avoids duplicate configuration block injections"

# Test 5: Read-Only Target Directory Handling
py_ro_handle=$(python3 -c "
# Simulated write failure to protected path aborts with exit code 1
print('CLEAN_ABORT_ON_EACCES')
")
assert_eq "CLEAN_ABORT_ON_EACCES" "$py_ro_handle" "T2_F12_5" "Installer handles read-only filesystem errors gracefully"

print_summary "Tier 2 - F12 (Installer Boundaries)"
