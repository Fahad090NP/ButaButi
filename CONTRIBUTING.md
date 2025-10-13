# Contributing to Butabuti Embroidery Library

Thank you for your interest in contributing to Butabuti! This document provides guidelines for contributing to the project.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:

   ```sh
   git clone https://github.com/Fahad090NP/Butabuti.git
   cd Butabuti
   ```

3. **Create a branch** for your changes:

   ```sh
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Love ❤️
- Rust (latest)
- Git
- A code editor (VS Code recommended)

### Building the Project

```sh
cargo build
```

### Running Tests

```sh
.\validate.ps1 # Must pass with zero warnings
```

## Code Guidelines

### Coding Standards

1. **Follow Rust conventions** - Use `cargo fmt` and `cargo clippy`
2. **Write tests** - All new features must include tests
3. **Document public APIs** - Use doc comments (`///`) for public items
4. **Handle errors gracefully** - Return `Result`, don't panic in library code
5. **Keep functions focused** - Small, single-purpose functions
6. **Prefer small file names** - Use compound names for clarity (e.g., `renderer.rs` over `stitch_renderer.rs`)
7. **Always validate before commit** - Run `.\validate.ps1` (or `cargo test --lib && cargo clippy -- -D warnings && cargo fmt`) before pushing

### File Naming Conventions

**Single words acceptable when:**

- Parent folder provides full context: `formats/registry.rs`
- Universally understood: `error.rs`, `constants.rs`, `utils.rs`
- No ambiguity: `pattern.rs` in `core/`

**Rationale**: Descriptive names improve searchability, reduce false positives in grep/IDE search, and make intent immediately clear.

### File Organization Principles

**Current files are well-organized** - avoid unnecessary refactoring unless clear benefit.

### Automation Policy

**NEVER use scripts to automate:**

- Documentation generation (markdown files, wikis, changelogs)
- Code file creation from templates
- API documentation extraction
- Release notes compilation

**Scripts ONLY for:**

- Build processes (cargo, wasm-pack)
- Testing (cargo test, validate.ps1)
- Formatting/linting (cargo fmt, cargo clippy)
- Deployment (wasm/build.ps1)

**Instead of automation, add TODO items:**

```markdown
- [ ] Update wiki documentation for new feature X
- [x] Document format Y in Format-Support.md
- [ ] Add example for use case Z to Examples.md
```

### Import Guidelines

Use the new module structure:

```rust
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::Result;
use crate::formats::io::readers;
```

### Testing Requirements

- Add unit tests in `#[cfg(test)]` modules
- Aim for high code coverage
- Test edge cases (empty patterns, invalid data, etc.)
- All tests must pass: `cargo test --lib`

## Pull Request Process

1. **Update tests** - Add tests for new features
2. **Update documentation** - Keep README and docs current
3. **Run quality checks**:

   ```sh
   .\validate.ps1
   ```

4. **Write clear commit messages**:

   ```txt
   Add SVG writer with gradient support

   - Implements SVG export with scalable vector graphics
   - Adds automatic viewBox calculation
   - Includes 3 comprehensive tests
   ```

5. **Create pull request** with description of changes

6. **Address review feedback** promptly

## Adding New Features

### Adding a Format Reader

1. Create `src/formats/io/readers/formatname.rs`
2. Implement `pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>`
3. Export in `src/formats/io/readers.rs`
4. Add tests (basic read + round-trip if writer exists)

### Adding a Format Writer

1. Create `src/formats/io/writers/formatname.rs`
2. Implement `pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()>`
3. Export in `src/formats/io/writers.rs`
4. Add tests (basic write + round-trip)

## Code Review Checklist

Before submitting, ensure:

- [ ] All tests pass (`.\validate.ps1`)
- [ ] Public APIs are documented
- [ ] New features have tests
- [ ] Commit messages are clear (prefer capital case like 'New Feature Added')
- [ ] No breaking changes (or clearly documented)

## Getting Help

- **Issues**: Open an issue on GitHub for bugs or questions
- **Discussions**: Use GitHub Discussions for general questions
- **Documentation**: Check README.md and inline documentation

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Help others learn and grow

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Butabuti! 🌸
