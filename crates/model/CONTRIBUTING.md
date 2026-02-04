# Contributing to SutraWorks Model\n\nThank you for your interest in contributing to SutraWorks Model! \n\n**Status**: Production-ready codebase with 57/57 tests passing and zero TODOs.\nThis project is now deployment-ready for enterprise use.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **VSCode** (recommended): With Rust-Analyzer extension

### Setting Up Your Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/sutraworks-model
   cd sutraworks-model
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/sutraworks/model
   ```

4. Build the project:
   ```bash
   cargo build --all
   ```

5. Run tests to ensure everything works:
   ```bash
   cargo test --all
   # Should see: 57 tests passing
   ```

## 🏗️ Project Structure

The project is organized as a Cargo workspace with 9 crates:

```
crates/
├── sutra-core/          # Foundation: tensors, errors, traits (7 tests)
├── sutra-quantize/      # AWQ 4-bit quantization (2 tests)
├── sutra-peft/          # LoRA/QLoRA fine-tuning (5 tests)
├── sutra-rwkv/          # RWKV architecture (3 tests)
├── sutra-mamba/         # Mamba state space models (3 tests)
├── sutra-nesy/          # Neuro-symbolic AI (4 tests)
├── sutra-loader/        # Model loading (safetensors) (3 tests)
├── sutra-tokenizer/     # Tokenization (BPE, WordPiece, Unigram) (13 tests)
├── sutra-training/      # Training infrastructure (3 tests)
└── tests/               # Integration tests (5 tests)
```

## 📝 Development Workflow

### Using VSCode Tasks

The project includes pre-configured VSCode tasks (`.vscode/tasks.json`):

- **Build All (Release)**: `Cmd+Shift+B` - Default build task
- **Test All Crates**: Run all tests
- **Run Examples**: Individual tasks for each example
- **Clippy**: Run the linter
- **Format Code**: Auto-format with rustfmt

### Making Changes

1. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following our coding standards

3. Run tests frequently:
   ```bash
   cargo test --all
   ```

4. Format your code:
   ```bash
   cargo fmt --all
   ```

5. Run clippy to catch common mistakes:
   ```bash
   cargo clippy --all -- -D warnings
   ```

6. Commit your changes:
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

### Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add GPTQ quantization support
fix: correct memory calculation in QLoRA
docs: update README with new examples
test: add integration tests for tokenizer
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific crate
cargo test -p sutra-quantize

# Run tests in release mode (faster)
cargo test --all --release

# Run specific test
cargo test test_awq_quantization
```

### Writing Tests

All new functionality must include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = my_function(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Test Coverage

We aim for high test coverage. Check current coverage:
```bash
cargo tarpaulin --all --out Html
```

## 📚 Documentation

### Code Documentation

- Add doc comments to all public items:
  ```rust
  /// Performs AWQ quantization on the given weights.
  ///
  /// # Arguments
  ///
  /// * `weights` - The model weights to quantize
  /// * `bits` - Number of bits (typically 4)
  ///
  /// # Returns
  ///
  /// Quantized weights with compression metadata
  pub fn quantize(&self, weights: &Tensor, bits: u8) -> Result<QuantizedTensor> {
      // implementation
  }
  ```

- Generate and review docs:
  ```bash
  cargo doc --all --no-deps --open
  ```

### Examples

When adding new features, include a runnable example in `examples/`:

```rust
//! Example: Demonstrating new feature
//! 
//! Run with: cargo run --example my_feature --release

use sutra_core::*;

fn main() -> Result<()> {
    println!("=== My Feature Demo ===\n");
    
    // Demo code here
    
    Ok(())
}
```

## 🎯 Areas for Contribution

We welcome contributions in the following areas:

### High Priority
- 🔢 **Quantization Methods**: GPTQ, GGUF support
- 🏗️ **Architectures**: RetNet, Griffin implementations
- 📊 **Benchmarking**: Performance measurement suite
- 📖 **Documentation**: Tutorials, guides, API docs

### Medium Priority
- 🔄 **Model Converters**: PyTorch → safetensors
- 🎓 **Training**: Advanced optimizers, schedulers
- 🌐 **Data Loaders**: Efficient dataset handling
- 🧪 **Testing**: More integration tests

### Lower Priority
- 🎨 **UI/UX**: Web interface for demos
- 📦 **Packaging**: Distribution improvements
- 🌍 **I18n**: Internationalization
- 📱 **Mobile**: Deployment guides

## 🔍 Code Review Process

1. **Submit a Pull Request** against the `main` branch
2. **Describe your changes** clearly in the PR description
3. **Link related issues** if applicable
4. **Wait for CI** to pass (automated tests, clippy, formatting)
5. **Address review comments** from maintainers
6. **Squash commits** if requested before merging

### PR Checklist

Before submitting, ensure:

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Clippy passes (`cargo clippy --all -- -D warnings`)
- [ ] Documentation is updated
- [ ] New features include tests
- [ ] Commit messages follow convention
- [ ] Branch is up-to-date with `main`

## 🐛 Reporting Bugs

Found a bug? Please [open an issue](https://github.com/sutraworks/model/issues) with:

1. **Clear title** describing the problem
2. **Steps to reproduce** the bug
3. **Expected behavior** vs actual behavior
4. **Environment details** (OS, Rust version, etc.)
5. **Code sample** if applicable

Example:
```markdown
## Bug: AWQ quantization fails for large tensors

**Steps to reproduce:**
1. Create a tensor with shape [4096, 4096]
2. Call `quantizer.quantize(&tensor, None)`
3. Observe panic

**Expected:** Successful quantization
**Actual:** Thread panic with "out of memory"

**Environment:**
- OS: macOS 14.0
- Rust: 1.75.0
- RAM: 16GB
```

## 💡 Suggesting Features

Have an idea? [Open an issue](https://github.com/sutraworks/model/issues) with:

1. **Use case** - Why is this needed?
2. **Proposed solution** - How would it work?
3. **Alternatives** - Other approaches considered?
4. **Impact** - Who benefits from this?

## 🤝 Community

- **Discussions**: [GitHub Discussions](https://github.com/sutraworks/model/discussions)
- **Issues**: [GitHub Issues](https://github.com/sutraworks/model/issues)
- **Discord**: [Join our server](https://discord.gg/sutraworks) (coming soon)

## 📄 License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT or Apache-2.0).

## 🙏 Recognition

All contributors will be recognized in:
- README.md contributors section
- Release notes
- GitHub contributors page

Thank you for making SutraWorks Model better! 🎉
