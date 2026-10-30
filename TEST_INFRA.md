# TEST_INFRA.md: VietKeyra End-to-End Test Infrastructure Specification

## 1. Test Philosophy & Architecture

### 1.1 Core Principles
VietKeyra is a critical system-level input method engine (IME) designed for daily use on Linux (Arch Linux + Hyprland + Waybar). As such, the testing infrastructure must satisfy four uncompromising principles:
1. **Opaque-Box Verification**: Tests evaluate system behavior exclusively through external interfaces: public Rust crate APIs, command-line arguments and standard streams, Unix domain socket JSON messages, Waybar output formats, shell script exit codes, and file system states. Tests never inspect or rely on private implementation details.
2. **Progressive Testability**: Testing is organized along milestone boundaries (M1 through M5). Each test suite can execute incrementally as soon as its underlying component is built, allowing continuous verification without waiting for all downstream components. When a downstream binary is not yet compiled, progressive runners report explicit readiness status rather than failing unpredictably.
3. **Strict NFC & Orthographic Fidelity**: Every output containing Vietnamese text is validated against the Unicode Standard (NFC). Zero decomposed combining characters (`\u{0300}`..=`\u{036F}`) are permitted. Modern tone placement (BGD&ĐT standard) is verified as the default, with Traditional style verified when explicitly toggled.
4. **Deterministic & Isolated Execution**: Every test sets up its own isolated temporary environment (e.g., temporary socket paths, sandbox directories, mock environment variables) and cleans up completely after execution. Zero global state leakage is allowed.

---

## 2. Feature Inventory & Test Mapping

The test infrastructure covers all 15 features defined in `PROJECT.md`:

| Feature ID | Feature Name | Primary Milestone | Interface / Target | Tier 1 Tests | Tier 2 Tests | Tier 3/4 Coverage |
|---|---|---|---|---|---|---|
| **F1** | Pure Rust Zero-Dependency NFC Telex Engine | M1 | `vietkeyra-engine` public API | T1_F01..T1_F05 | T2_F01..T2_F05 | T3_01, T4_01..08 |
| **F2** | Vietnamese Orthography & Transformations | M1 | `vietkeyra-engine::process_char` | T1_F06..T1_F10 | T2_F06..T2_F10 | T3_04..06, T4_01..08 |
| **F3** | Modern & Traditional Tone Placement | M1 | `TonePlacementStyle` | T1_F11..T1_F15 | T2_F11..T2_F15 | T3_15, T4_01..03 |
| **F4** | 105+ Word Unit Test Suite | M1 | `test_105_words` matrix | T1_F16..T1_F20 | T2_F16..T2_F20 | T4_01..04 |
| **F5** | Wayland Input Method Daemon | M2 | `vietkeyra-daemon` runtime & offsets | T1_F21..T1_F25 | T2_F21..T2_F25 | T3_01, T3_13 |
| **F6** | Virtual Keyboard Pass-Through | M2 | Virtual key forwarding logic | T1_F26..T1_F30 | T2_F26..T2_F30 | T3_06, T4_05 |
| **F7** | Decoupled Concurrency Architecture | M2 | Multi-threaded IPC + event loop | T1_F31..T1_F35 | T2_F31..T2_F35 | T3_01, T3_09 |
| **F8** | Unix Domain Socket IPC Server & Protocol | M2 | `$XDG_RUNTIME_DIR/vietkeyra.sock` | T1_F36..T1_F40 | T2_F36..T2_F40 | T3_01, T3_08..10 |
| **F9** | CLI Client Tool | M2 | `vietkeyra-cli` commands | T1_F41..T1_F45 | T2_F41..T2_F45 | T3_02, T3_10 |
| **F10** | Waybar Custom Module Integration | M3 | `custom/vietkeyra` JSON & RTMIN+8 | T1_F46..T1_F50 | T2_F46..T2_F50 | T3_02, T3_03 |
| **F11** | GTK4 Vietnamese Control Panel GUI | M3 | `vietkeyra-gui` & .desktop file | T1_F51..T1_F55 | T2_F51..T2_F55 | T3_03, T3_09 |
| **F12** | Automated Installer Script | M4 | `install.sh` | T1_F56..T1_F60 | T2_F56..T2_F60 | T3_07, T3_14, T4_08 |
| **F13** | Automated Uninstaller Script | M4 | `uninstall.sh` | T1_F61..T1_F65 | T2_F61..T2_F65 | T3_07, T4_08 |
| **F14** | Git Repo Setup & Strict Token Security | M4 | Git hooks, `.gitignore`, push hygiene | T1_F66..T1_F70 | T2_F66..T2_F70 | T3_08 |
| **F15** | E2E Test Suite & Test Runner | E2E | `tests/e2e_runner.sh` & verification | T1_F71..T1_F75 | T2_F71..T2_F75 | T3_11..12, T4_07..08 |

---

## 3. Four-Tier Test Architecture

The test suite is structured into 4 hierarchical tiers located under `/home/yusei1708/code/unikey/tests/`:

```
tests/
├── e2e_runner.sh                   # Main progressive test runner
├── common/
│   └── test_helpers.sh             # Shared test utilities, assertions, colors, reporting
├── tier1_features/
│   ├── test_f01_engine_nfc.sh      # F1: Pure Rust Zero-Dependency NFC Engine (5 tests)
│   ├── test_f02_orthography.sh     # F2: Complete Vietnamese Orthography (5 tests)
│   ├── test_f03_tone_placement.sh  # F3: Modern & Traditional Tone Placement (5 tests)
│   ├── test_f04_105_words.sh       # F4: 105+ Word Unit Test Verification (5 tests)
│   ├── test_f05_wayland_daemon.sh  # F5: Wayland Input Method Daemon (5 tests)
│   ├── test_f06_virtual_kb.sh      # F6: Virtual Keyboard Pass-Through (5 tests)
│   ├── test_f07_concurrency.sh     # F7: Decoupled Concurrency Architecture (5 tests)
│   ├── test_f08_socket_ipc.sh      # F8: Unix Domain Socket IPC Server & Protocol (5 tests)
│   ├── test_f09_cli_tool.sh        # F9: CLI Client Tool (5 tests)
│   ├── test_f10_waybar_module.sh   # F10: Waybar Custom Module Integration (5 tests)
│   ├── test_f11_gtk4_gui.sh        # F11: GTK4 Vietnamese Control Panel GUI (5 tests)
│   ├── test_f12_installer.sh       # F12: Automated Installer Script (5 tests)
│   ├── test_f13_uninstaller.sh     # F13: Automated Uninstaller Script (5 tests)
│   ├── test_f14_git_security.sh    # F14: Git Repo Setup & Strict Token Security (5 tests)
│   └── test_f15_test_infra.sh      # F15: E2E Test Suite & Runner Integrity (5 tests)
├── tier2_boundaries/
│   ├── test_f01_engine_bounds.sh   # F1 boundary tests (5 tests)
│   ├── test_f02_ortho_bounds.sh    # F2 boundary tests (5 tests)
│   ├── test_f03_tone_bounds.sh     # F3 boundary tests (5 tests)
│   ├── test_f04_words_bounds.sh    # F4 boundary tests (5 tests)
│   ├── test_f05_daemon_bounds.sh   # F5 boundary tests (5 tests)
│   ├── test_f06_vkb_bounds.sh      # F6 boundary tests (5 tests)
│   ├── test_f07_concur_bounds.sh   # F7 boundary tests (5 tests)
│   ├── test_f08_ipc_bounds.sh      # F8 boundary tests (5 tests)
│   ├── test_f09_cli_bounds.sh      # F9 boundary tests (5 tests)
│   ├── test_f10_waybar_bounds.sh   # F10 boundary tests (5 tests)
│   ├── test_f11_gui_bounds.sh      # F11 boundary tests (5 tests)
│   ├── test_f12_install_bounds.sh  # F12 boundary tests (5 tests)
│   ├── test_f13_uninstall_bounds.sh# F13 boundary tests (5 tests)
│   ├── test_f14_git_bounds.sh      # F14 boundary tests (5 tests)
│   └── test_f15_runner_bounds.sh   # F15 boundary tests (5 tests)
├── tier3_combinations/
│   └── test_cross_feature.sh       # 15 Cross-Feature Integration tests
└── tier4_realworld/
    └── test_realworld_workloads.sh # 8 Real-World User Workloads
```

### 3.1 Tier 1: Feature Coverage (>=75 tests, exactly 75 implemented)
Evaluates primary happy paths for each of the 15 features in complete isolation:
- **F1 (5 tests)**: 134 vowel-modifier-tone variants; NFC normalization; zero combining marks; O(1) table lookup consistency; ASCII consonant preservation.
- **F2 (5 tests)**: Circumflex transforms (`aa->â`, `ee->ê`, `oo->ô`); Breve transform (`aw->ă`); Horn transforms (`ow->ơ`, `uw->ư`, `w->ư`); D-stroke (`dd->đ`); Tone keys (`s,f,r,x,j`).
- **F3 (5 tests)**: Modern standard tone on `oa, oe, uy` without coda (`hoà, khoè, thuỷ`); Traditional tone placement (`hòa, khỏe, thủy`); Closed syllables with coda (`toán, hoàn, tiến`); Open syllables (`mía, của, lụa`); Triphthong center placement (`ngoài, chuối, rượu`).
- **F4 (5 tests)**: 105-word batch execution; Vowel matrix coverage; Diacritic validity; Uppercase and Titlecase casing; Repeat-key undo batch.
- **F5 (5 tests)**: Daemon binary builds/starts; KeyAction `delete_bytes` equals exact UTF-8 byte length; PassThrough non-character keys; Rootless user execution; Clean SIGINT/SIGTERM termination.
- **F6 (5 tests)**: Arrow/navigation key passthrough; Ctrl/Alt/Super shortcut bypass; English mode passthrough; Virtual keyboard key event generation; Keymap sync.
- **F7 (5 tests)**: Wayland loop thread separation from IPC; Sub-millisecond keystroke latency; Non-blocking channel communication; High socket throughput without typing lag; Clean thread termination.
- **F8 (5 tests)**: IPC socket creation with 0700 permissions; `get_status` command; `set_mode` command; `toggle` command; `subscribe` event streaming.
- **F9 (5 tests)**: `vietkeyra-cli status`; `vietkeyra-cli toggle`; `vietkeyra-cli set-mode`; `vietkeyra-cli waybar` single-line JSON; `vietkeyra-cli monitor`.
- **F10 (5 tests)**: Waybar JSON schema validation (`text, alt, tooltip, class`); VI mode output; EN mode output; `signal: 8` configuration validity; Waybar CSS class selectors.
- **F11 (5 tests)**: GTK4 desktop file validity; Vietnamese UI strings presence; Socket subscription handler; Mode toggle reflection; Non-root GUI startup.
- **F12 (5 tests)**: `install.sh` syntax validity (`bash -n`); Arch Linux detection logic; Backup creation in `~/.config/vietkeyra/backups/`; Marker injection syntax; Clean exit code.
- **F13 (5 tests)**: `uninstall.sh` syntax validity (`bash -n`); Process termination logic; Marker removal logic; Binary cleanup from `~/.local/bin`; Restoring original files.
- **F14 (5 tests)**: `.gitignore` contains `.agents/` and secret patterns; Clean remote URL without inline token; Pre-commit hook blocking `ghp_*` tokens; Pre-commit hook blocking `.agents/` directory; `scripts/check-secrets.sh` execution.
- **F15 (5 tests)**: `tests/e2e_runner.sh` executable and syntax valid; `--tier` flag selection; `--json` reporting format; Progressive skipping when components missing; Proper exit code propagation.

### 3.2 Tier 2: Boundary & Corner Cases (>=75 tests, exactly 75 implemented)
Stress tests, edge cases, invalid inputs, and security boundaries across all 15 features:
- **F1 (5 tests)**: Empty buffer processing; 100+ character continuous buffer; Non-Vietnamese characters (Chinese, Cyrillic, emoji); Surrogate pair safety; Control character filtering.
- **F2 (5 tests)**: Rapid repeat triggers (`aaaa`, `dddd`); Multiple conflicting tones (`asfrxj`); Impossible vowel clusters (`aeou`); Consonant cluster non-transformation (`str`, `brr`); Mixed case preservation (`vIeEtJ`).
- **F3 (5 tests)**: Ambiguous onset `qu` with vowels (`quán, quốc, quần, quyết`); Ambiguous onset `gi` (`già, gió, giúp, giếng, gì`); Stop codas `c, ch, p, t` with invalid tones (preventing or rejecting `hỏi/ngã/huyền`); Edge triphthongs (`khuỷu, khoèo`); Consecutive tone shifting.
- **F4 (5 tests)**: Extremely rare orthographic words; Backspace sequence through full word; Tone removal on non-toned word; Standalone tone key typing; Rapid vowel-tone cycling.
- **F5 (5 tests)**: Truncated surrounding text (>4KB buffer); UTF-8 byte boundary splitting protection (no partial 2/3-byte slicing); Rapid activate/deactivate flapping; Stale socket cleanup on crash; Wayland compositor reconnect retry.
- **F6 (5 tests)**: Rapid keystroke storm (1000 keystrokes/sec); Simultaneous 4-modifier chord; Unknown/raw keysym passthrough; Key release without key press; Virtual keyboard memory safety.
- **F7 (5 tests)**: Socket client flooding while typing; Socket buffer full (`EWOULDBLOCK`); IPC thread crash does not crash typing; Client disconnect mid-message; Concurrent client scaling (10+ connections).
- **F8 (5 tests)**: Malformed JSON lines; Oversized payload (>64KB); Unknown command error response; Partial line / split newline; Permissions denial on socket.
- **F9 (5 tests)**: CLI invoked when daemon socket is offline; Invalid CLI arguments; Rapid CLI invocation loop (20 toggles); CLI output piped to `jq`; CLI non-zero exit code on daemon error.
- **F10 (5 tests)**: Waybar output when daemon is offline; Waybar JSON escaping for special characters; Rapid signal storm (`SIGRTMIN+8`); CSS syntax verification; Tooltip format validation.
- **F11 (5 tests)**: GUI startup when daemon is offline; GUI receives malformed JSON event; Rapid remote mode changes; Window resize/close event handling; Memory leak prevention on long runs.
- **F12 (5 tests)**: Installer run on non-Arch Linux (graceful exit); Read-only filesystem handling; Existing backup overwrite prevention; Conflict handling when fcitx5 has multiple PIDs; Corrupted config file resilience.
- **F13 (5 tests)**: Uninstaller run when no backup exists; Uninstaller run when files already removed; Malformed/nested marker block removal; Zombie process termination; Uninstaller idempotency.
- **F14 (5 tests)**: Fine-grained token format `github_pat_*` blocked; Token embedded in commit message diff; Binary file containing secret pattern; Token in sub-subdirectories; Empty staged commit safety.
- **F15 (5 tests)**: Runner timeout handling for stuck test; Runner handling missing test files; Non-tty execution formatting; Negative tier filter validation; Runner fail-fast mode.

### 3.3 Tier 3: Cross-Feature Combinations (>=15 tests, exactly 15 implemented)
Multi-component integration tests verifying component contracts:
1. **T3_01**: Telex typing in engine while concurrently toggling mode via IPC socket.
2. **T3_02**: Mode toggle via CLI immediately reflected in Waybar module JSON and RTMIN+8 signal.
3. **T3_03**: Waybar click action executes CLI toggle, which updates daemon and publishes to GUI subscription.
4. **T3_04**: CapsLock state combined with Telex diacritic generation preserves uppercase accents.
5. **T3_05**: Backspacing partially transformed word followed by typing further Telex keys.
6. **T3_06**: Bilingual typing switch: VI mode -> hotkey switch to EN mode -> English typing with `s,f,r,x,j` -> switch back to VI mode.
7. **T3_07**: Installer creates backup manifest -> uninstaller restores manifest without modifying user config files.
8. **T3_08**: Pre-commit hook executes `check-secrets.sh` and blocks staging forbidden files.
9. **T3_09**: Simultaneous IPC clients: CLI querying status while GUI is subscribed and Waybar is signaling.
10. **T3_10**: Rapid CLI toggle loop: 10 toggles alternating VI/EN, verifying state consistency across engine, daemon, and Waybar.
11. **T3_11**: Engine word boundary reset on non-word keys (Space, Tab, Enter, punctuation) followed by new word typing.
12. **T3_12**: Free-tone typing across multiple syllables in a full sentence.
13. **T3_13**: Daemon clean shutdown on SIGTERM: unlinks socket file, drops Wayland proxies, exits with code 0.
14. **T3_14**: Installer config injection preserves existing Hyprland keybinds.
15. **T3_15**: Dynamic tone placement style switch (Modern <-> Traditional) via IPC/config affecting subsequent typing of `hoaf` / `thuyr`.

### 3.4 Tier 4: Real-World Application Scenarios (>=8 tests, exactly 8 implemented)
Realistic user workloads validating end-to-end functionality:
1. **T4_01**: Typing Vietnamese Pangram 1: "Doãn trí trổ tài, khuấy đục quấy trong, phụng sự muôn bề".
2. **T4_02**: Typing Vietnamese Pangram 2: "Cậu bé chăn trâu vội vã lùa bầy dê về chuồng khi trời đổ mưa lớn".
3. **T4_03**: Typing Classical Vietnamese Poetry: Luc Bat stanza ("Trăm năm trong cõi người ta / Chữ tài chữ mệnh khéo là ghét nhau / Trải qua một cuộc bể dâu / Những điều trông thấy mà đau đớn lòng").
4. **T4_04**: Typing Technical Code Comments in Vietnamese inside source code.
5. **T4_05**: Alternating Bilingual Prose: Writing an email with alternating English and Vietnamese sentences.
6. **T4_06**: Typing Vietnamese addresses and proper nouns with tone marks and uppercase letters.
7. **T4_07**: High-speed typing simulation with realistic keystroke latency (15-50ms between keys) testing throughput and zero dropped keys.
8. **T4_08**: Full Lifecycle Scenario: Install -> Launch Daemon -> Connect CLI -> Check Waybar JSON -> Type Paragraph -> Switch Mode -> Verify Clean State -> Uninstall.

---

## 4. Test Runner Architecture (`tests/e2e_runner.sh`)

### 4.1 CLI Interface
```bash
./tests/e2e_runner.sh [OPTIONS]

Options:
  --all               Run all test suites (Tiers 1-4) [default]
  --tier <1|2|3|4>    Run only the specified test tier
  --feature <F1..F15> Run tests covering a specific feature
  --quick             Fast run (skip stress tests)
  --json              Output test results in structured JSON format
  --verbose           Show detailed command outputs and logs
  --help              Display help and usage
```

### 4.2 Progressive Test Execution Model
The runner inspects the repository state to detect implemented components:
```
1. Detect crates/vietkeyra-engine:
   - If present: compile and run engine unit tests & standalone engine harnesses.
   - If absent: report [PROGRESSIVE: M1 not yet built - skipping engine tests].

2. Detect crates/vietkeyra-daemon & vietkeyra-cli:
   - If present: compile and run daemon/CLI tests and IPC protocol suites.
   - If absent: report [PROGRESSIVE: M2 not yet built - skipping daemon/CLI tests].

3. Detect configs/waybar & crates/vietkeyra-gui:
   - If present: run Waybar module & GTK4 UI integration tests.
   - If absent: report [PROGRESSIVE: M3 not yet built - skipping GUI tests].

4. Detect install.sh, uninstall.sh, scripts/check-secrets.sh:
   - If present: run installer, uninstaller, and security tests.
   - If absent: report [PROGRESSIVE: M4 not yet built - skipping installer/security tests].
```

### 4.3 Exit Code Standards
- `0`: All executed tests passed.
- `1`: One or more executed tests failed.
- `2`: Environment or invocation error (e.g. missing bash or core utilities).

---

## 5. Coverage and Quality Thresholds

| Metric | Target Threshold | Verification Method |
|---|---|---|
| Total Test Cases | >= 173 tests | Automated count in `e2e_runner.sh` |
| Tier 1 Feature Tests | >= 75 tests (5 / feature) | `tests/tier1_features/*.sh` |
| Tier 2 Boundary Tests | >= 75 tests (5 / feature) | `tests/tier2_boundaries/*.sh` |
| Tier 3 Combination Tests | >= 15 tests | `tests/tier3_combinations/*.sh` |
| Tier 4 Real-World Tests | >= 8 tests | `tests/tier4_realworld/*.sh` |
| Unicode NFC Compliance | 100% (0 decomposed combining marks) | `is_nfc_clean()` assert in test helper |
| Keystroke Latency Budget | < 1,000 µs (1 ms) | Benchmark assertion in T1_F07 / T2_F07 |
| Secret Leak Scan Pass Rate | 100% | Zero `ghp_*` tokens in git diff/history |
