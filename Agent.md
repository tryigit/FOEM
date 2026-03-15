# FOEM Project Standards and Contributor Agent Guidelines

This document defines the engineering standards, coding conventions, and contribution protocols for the FOEM project. All contributors, whether human or automated, must adhere to these guidelines to maintain codebase quality and consistency.

---

## Table of Contents

1. [Project Identity](#project-identity)
2. [Language and Localization Policy](#language-and-localization-policy)
3. [Code Standards](#code-standards)
4. [Architecture Principles](#architecture-principles)
5. [Safety-First Development](#safety-first-development)
6. [Contribution Protocol](#contribution-protocol)
7. [Documentation Standards](#documentation-standards)
8. [Review Criteria](#review-criteria)

---

## Project Identity

**FOEM** -- Free Open Ecosystem for Mobile Devices.

FOEM is a cross-platform desktop application for advanced mobile device management and repair. It provides low-level access to bootloaders, diagnostic ports, flash protocols, and device partitions across all major chipset platforms (Qualcomm, MediaTek, Unisoc, Samsung Exynos, Huawei HiSilicon).

The project serves the right-to-repair community, independent security researchers, and academic institutions. It is not a commercial product and must never be monetized without explicit written authorization from the project author.

---

## Language and Localization Policy

### Development Language

All source code, comments, variable names, function names, commit messages, documentation, log messages, and internal identifiers must be written in **English**. No exceptions.

This policy exists to:
- Ensure universal readability across the international contributor base.
- Prevent encoding issues and character-set ambiguities in toolchains.
- Maintain consistency when automated tools (linters, code review, CI) process the codebase.

### User-Facing Text

All strings displayed to the end user must be sourced from the localization system (`locales/*.json`). Hardcoded user-facing strings in source code are not acceptable. The English locale file (`locales/en.json`) is the canonical reference.

Refer to `TRANSLATION_GUIDE.md` for the complete localization contribution process.

---

## Code Standards

### Language: Rust

- **Edition:** 2021.
- **Toolchain:** Stable. No nightly-only features.
- **Formatting:** All code must pass `cargo fmt` with default settings. No custom rustfmt configuration overrides.
- **Linting:** All code must pass `cargo clippy` with no warnings. Treat warnings as errors in CI.
- **Naming:** Follow standard Rust conventions.
  - `snake_case` for functions, methods, variables, and modules.
  - `CamelCase` for types, traits, and enums.
  - `SCREAMING_SNAKE_CASE` for constants and statics.
- **Error Handling:** Use `Result<T, E>` for fallible operations. Avoid `unwrap()` and `expect()` in production code paths. Panics are acceptable only in truly unrecoverable situations (e.g., startup configuration failure).
- **Unsafe Code:** Minimize. Any `unsafe` block must include a `// SAFETY:` comment explaining why the invariants are upheld.

### Dependencies

- Keep the dependency count as low as reasonably possible.
- Every new dependency must be justified in the pull request description.
- All dependencies must be checked against the GitHub Advisory Database before inclusion.
- Prefer well-maintained, widely-used crates with active security response histories.

### Comments

- Write comments to explain **why**, not **what**. The code itself should be clear enough to explain what it does.
- Do not leave commented-out code in the repository. Use version control history instead.
- Do not use decorative comment blocks, ASCII art, or emoji in comments.

---

## Architecture Principles

### Separation of Concerns

The application is layered:

1. **UI Layer** (eframe/egui) -- Rendering and user interaction only.
2. **Logic Layer** (feature modules) -- Business rules, command construction, result interpretation.
3. **Safety Layer** -- Auto-backup enforcement, CRC validation, timeout management.
4. **Communication Layer** -- ADB/Fastboot wrappers, protocol handlers (EDL, BROM, Diag).

Each layer communicates strictly downward. The UI layer must never issue raw device commands.

### Thread Safety

- All device I/O runs on background threads. The GUI thread must never block on I/O.
- Shared state between GUI and worker threads uses `Arc<Mutex<T>>` or `Arc<AtomicBool>`.
- Worker threads must not hold GUI-related types.
- Long operations must check a cancellation flag at regular intervals.

### Module Independence

Feature modules (`bootloader.rs`, `repair.rs`, `network.rs`, `flash.rs`, `hardware_test.rs`, `tools.rs`) should be as independent as possible. Cross-module dependencies must go through the shared helpers in `features/mod.rs`.

---

## Safety-First Development

FOEM interacts with hardware at a level where mistakes cause permanent damage. Every contributor must internalize this reality.

### Mandatory Auto-Backup

Any operation that modifies device partitions, EFS data, NV items, or boot images must trigger an automatic backup of the affected data before proceeding. This is not optional. The backup system is documented in `ARCHITECTURE_AND_SECURITY.md`.

### Timeout Enforcement

Every device communication call must have a timeout. No operation is permitted to run indefinitely. Timeout values must be calibrated to the operation type and documented.

### CRC and Integrity Checks

All data transfers to the device must include integrity verification where the protocol supports it. Backup files must be verified immediately after creation.

### Destructive Operation Confirmation

Operations that erase, overwrite, or wipe device data must:
1. Display a clear warning describing the consequences.
2. Require explicit user confirmation.
3. Create an auto-backup (per the policy above).
4. Log the operation details (timestamp, device serial, operation type, files involved).

---

## Contribution Protocol

### Branch Naming

- Features: `feature/<short-description>`
- Bug fixes: `fix/<short-description>`
- Localizations: `localization/<language-code>`
- Documentation: `docs/<short-description>`

### Commit Messages

- Use imperative mood: "Add EFS backup module", not "Added EFS backup module" or "Adds EFS backup module".
- Keep the first line under 72 characters.
- Reference issue numbers where applicable: "Fix timeout handling in EDL flash (#42)".
- Do not use emoji in commit messages.

### Pull Request Requirements

Every pull request must include:
1. A clear description of the change and its motivation.
2. Confirmation that `cargo fmt`, `cargo clippy`, and `cargo build --release` pass without errors or warnings.
3. If adding a new dependency: justification and advisory database check results.
4. If adding user-facing text: corresponding entries in `locales/en.json`.
5. If modifying device communication: description of testing performed (device model, firmware version, protocol used).

### Issue Reporting

When filing an issue, include:
- Device manufacturer, model, and firmware version.
- Operating system and FOEM version.
- Exact steps to reproduce the problem.
- Relevant log output.
- Screenshots of the UI state, if applicable.

---

## Documentation Standards

### Style

- Write in clear, direct English. Avoid jargon where a plain term exists.
- Use formal register. This is a technical document, not a conversation.
- Do not use emoji, exclamation marks, or informal language.
- Use present tense: "The system creates a backup", not "The system will create a backup".
- Use active voice: "The worker thread sends the command", not "The command is sent by the worker thread".

### Structure

- Every documentation file must have a table of contents if it exceeds one screen of content.
- Use ATX-style headings (`#`, `##`, `###`). Do not use setext-style (underline) headings.
- Use fenced code blocks with language identifiers for all code samples.
- Tables must use the pipe (`|`) syntax with left-aligned columns by default.

### Required Documentation

The following files must be maintained in the repository root:

| File                            | Purpose                                              |
| :------------------------------ | :--------------------------------------------------- |
| `README.md`                     | Project overview, features, setup, legal disclaimer   |
| `LICENSE`                       | Full license text                                     |
| `SECURITY.md`                   | Vulnerability reporting policy                        |
| `ARCHITECTURE_AND_SECURITY.md`  | Technical architecture and safety protocols            |
| `TRANSLATION_GUIDE.md`          | Localization contributor guide                        |
| `Agent.md`                      | This file: project standards and contributor guidelines |

---

## Review Criteria

Pull requests are evaluated against the following checklist:

- [ ] Code compiles without errors or warnings (`cargo build --release`).
- [ ] Code passes formatting check (`cargo fmt --check`).
- [ ] Code passes linting (`cargo clippy`).
- [ ] No hardcoded user-facing strings (all text sourced from locale files).
- [ ] No `unwrap()` or `expect()` in production code paths without justification.
- [ ] No `unsafe` blocks without `// SAFETY:` comments.
- [ ] No commented-out code.
- [ ] No emoji in code, comments, documentation, or commit messages.
- [ ] Destructive operations trigger auto-backup.
- [ ] All device I/O runs on background threads.
- [ ] All device communication calls have timeouts.
- [ ] New dependencies are justified and checked against advisory databases.
- [ ] New user-facing strings are added to `locales/en.json`.
- [ ] Documentation is updated if the change affects architecture, features, or contributor workflows.

---

This document is a living standard. It evolves as the project matures. Proposed changes to these guidelines follow the same pull request process as code changes.
