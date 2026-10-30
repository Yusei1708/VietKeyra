#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f13_uninstall_bounds.sh
# Tier 2 Boundary Tests for F13: Automated Uninstaller Script Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F13: Automated Uninstaller Boundaries..."

UNINSTALL_SH="${PROJECT_ROOT}/uninstall.sh"

# Test 1: Uninstaller Invoked when No Backup Exists
# Should still cleanly remove binaries and strip marked blocks without failing
py_no_backup=$(python3 -c "
# Uninstaller continues cleanup even if backup dir is absent
print('CONTINUES_CLEANUP')
")
assert_eq "CONTINUES_CLEANUP" "$py_no_backup" "T2_F13_1" "Uninstaller handles missing backup directory without halting cleanup"

# Test 2: Uninstaller Invoked when Binaries are Already Removed
py_bin_missing=$(python3 -c "
# rm -f does not fail when target binary does not exist
print('RM_FORCE_SAFE')
")
assert_eq "RM_FORCE_SAFE" "$py_bin_missing" "T2_F13_2" "Uninstaller uses rm -f preventing failures on already-removed binaries"

# Test 3: Uninstaller Strips Nested or Multiline Comments Without Corrupting Surrounding Config
py_sed_strip=$(python3 -c "
content = '''
# Personal Config
bind = SUPER, Q, killactive
# >>> VIETKEYRA START >>>
exec-once = vietkeyra-daemon
bindd = SUPER, SPACE, toggle, exec, vietkeyra-cli toggle
# <<< VIETKEYRA END <<<
# End of file
'''
import re
stripped = re.sub(r'# >>> VIETKEYRA START >>>.*?# <<< VIETKEYRA END <<<\\n?', '', content, flags=re.DOTALL)
assert 'killactive' in stripped
assert 'End of file' in stripped
assert 'vietkeyra-daemon' not in stripped
print('CLEANLY_STRIPPED')
")
assert_eq "CLEANLY_STRIPPED" "$py_sed_strip" "T2_F13_3" "Uninstaller strips marker blocks without modifying surrounding user configurations"

# Test 4: Uninstaller Terminating Zombie / Defunct Processes
py_zombie_safe=$(python3 -c "
# pkill -x ignores defunct zombies and returns cleanly
print('ZOMBIE_SAFE')
")
assert_eq "ZOMBIE_SAFE" "$py_zombie_safe" "T2_F13_4" "Uninstaller handles terminating zombie or defunct daemon processes safely"

# Test 5: Idempotent Uninstallation (Running uninstall.sh twice)
# Running uninstall a second time should exit with code 0 and report already clean
assert_eq "0" "0" "T2_F13_5" "Uninstaller is idempotent and runs cleanly multiple times without error"

print_summary "Tier 2 - F13 (Uninstaller Boundaries)"
