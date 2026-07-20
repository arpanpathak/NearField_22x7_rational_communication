
# 🤝 Contributing to NearField_22x7_rational_communication

First off, thank you for considering contributing! 🎉

This project is open-source and aims to be a robust, low-power NFC solution for the Raspberry Pi. Whether you're fixing a bug, adding a feature, improving docs, or just asking a question — you're welcome here.

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Code Style](#code-style)
- [Pull Request Workflow](#pull-request-workflow)
- [Testing](#testing)
- [Adding a New Tag Type](#adding-a-new-tag-type)
- [Adding a New Display Backend](#adding-a-new-display-backend)
- [Commit Messages](#commit-messages)
- [Reporting Issues](#reporting-issues)

---

## Code of Conduct

Be respectful, constructive, and inclusive. Harassment, trolling, and
personal attacks will not be tolerated. This project follows the
[Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

---

## Getting Started

1. 🍴 **Fork** the repository on GitHub.
2. 🌿 **Clone** your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/NearField_22x7_rational_communication.git
   cd NearField_22x7_rational_communication
   ```
3. 🌱 **Create a feature branch**:
   ```bash
   git checkout -b feat/your-feature-name
   ```
4. ✏️ **Make your changes**.
5. 🧪 **Run tests**:
   ```bash
   cargo test
   cargo clippy   # if available
   ```
6. 📬 **Open a Pull Request** against the `main` branch.

---

## Development Environment

### Hardware you'll need for integration testing

- Raspberry Pi Zero 2WH (or any Pi with UART)
- PN532 NFC module
- A few NFC tags (Mifare Classic, NTAG, etc.)

### Software setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional tools (optional but recommended)
rustup component add clippy rustfmt

# Build
cargo build

# Run unit tests
cargo test
```

> 💡 You can develop and run **unit tests** without any hardware.
> Only integration tests require a physical PN532.

---

## Code Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/).
- Run `cargo fmt` before committing.
- Run `cargo clippy` and address any warnings.
- Use `cargo check` to verify compilation before pushing.

### Naming Conventions

| Item | Convention | Example |
|---|---|---|
| Modules | `snake_case` | `pn532.rs`, `ascii.rs` |
| Types | `PascalCase` | `TagInfo`, `DisplayBackend` |
| Functions | `snake_case` | `classify_tag_type()` |
| Constants | `SCREAMING_SNAKE_CASE` | `CMD_SAM_CONFIGURATION` |
| Error variants | `PascalCase` | `AppError::NoTag` |

### Documentation

- All public items **must** have doc comments (`///` or `//!`).
- Include code examples in doc comments for non-trivial functions.
- Module-level docs (`//!`) should explain *why* something exists, not just *what*.

```rust
/// Poll for a single NFC tag in the RF field.
///
/// Returns `Ok(TagInfo)` if a tag is present, or `Err(AppError::NoTag)` if
/// the RF field is empty.
///
/// ## Debounce
/// After detecting a tag, we ignore it for 1 second to prevent duplicates.
pub fn poll(&mut self) -> AppResult<TagInfo> {
    // ...
}
```

---

## Pull Request Workflow

### TL;DR

```
1. Fork → 2. Branch → 3. Code → 4. Test → 5. PR → 6. Review → 7. Merge
```

### Detailed

1. **Keep PRs focused.** One feature/fix per PR. Split large changes.
2. **Write a clear title and description.** Explain *what* and *why*.
3. **Link related issues.** Use `Closes #123` or `Ref #456`.
4. **Ensure CI passes.** All tests must pass, no regressions.
5. **Respond to review feedback.** Address comments or explain your reasoning.
6. **Squash commits if requested.** Keep history clean.

### PR Title Format

```
feat: add NTAG 424 DNA tag type support
fix: correct UART checksum calculation for long payloads
docs: update wiring diagram for Pi Zero 2WH
refactor: extract frame parser into separate module
test: add integration test for SAMConfiguration
```

---

## Testing

### Running tests

```bash
# All unit tests
cargo test

# Specific test
cargo test test_render_tag_output_size

# With logging (for integration tests)
RUST_LOG=debug cargo run --release
```

### Writing tests

- Unit tests go in a `#[cfg(test)] mod tests` block at the bottom of the source file.
- Name tests descriptively: `test_<module>_<behavior>`.
- Integration tests (hardware-dependent) should be documented in [`docs/TESTING.md`](docs/TESTING.md).

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_mifare_classic_1k() {
        let tag_type = TagInfo::classify_tag_type(&[0x00, 0x04], 0x08);
        assert_eq!(tag_type, "Mifare Classic 1K");
    }
}
```

---

## Adding a New Tag Type

1. Find the ATQA (2 bytes) and SAK (1 byte) for your tag.
2. Add a match arm in `src/nfc/tag.rs` → `classify_tag_type()`.
3. Add a unit test.
4. Open a PR!

Example:

```rust
// In src/nfc/tag.rs
(0x00, 0x04, 0x18) => "Mifare Classic 4K".into(),
```

---

## Adding a New Display Backend

1. Create a struct implementing `DisplayBackend` in `src/display/backend.rs`.
2. Implement `init()`, `display_frame()`, `clear()`, `name()`.
3. Add a match arm in `backend_from_config()`.
4. Document it in the module docs.
5. Open a PR!

---

## Commit Messages

Write meaningful commit messages:

```
feat: add NTAG 424 DNA tag type classification

- Add ATQA 0x44 + SAK 0x20 detection
- Add unit test for the new type
- Verified with physical NTAG 424 DNA tag

Closes #42
```

Conventional prefixes:

| Prefix | When to use |
|---|---|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `docs:` | Documentation only |
| `refactor:` | Code change with no functional change |
| `test:` | Adding or fixing tests |
| `chore:` | Build, CI, tooling |

---

## Reporting Issues

When opening an issue, please include:

- **Hardware:** Pi model, PN532 variant, wiring method
- **Software:** Rust version (`rustc --version`), OS version
- **Logs:** Run with `RUST_LOG=debug` and paste relevant output
- **Steps to reproduce:** What you did, what happened, what you expected

---

> 💡 **Not sure where to start?** Look for issues labelled `good first issue`
> or `help wanted`. Or open a discussion — we're friendly!
