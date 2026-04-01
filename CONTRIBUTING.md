# Contributing to CogniPace

Thank you for your interest in contributing to CogniPace! This document provides guidelines and instructions for contributing.

## 🌟 Ways to Contribute

### Code Contributions
1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**
4. **Write or update tests**
5. **Commit with clear messages**: `git commit -m "feat: add amazing feature"`
6. **Push to your fork**: `git push origin feature/amazing-feature`
7. **Open a Pull Request**

### Non-Code Contributions
- **Documentation**: Improve README, write tutorials, fix typos
- **Bug Reports**: Use GitHub Issues with detailed reproduction steps
- **Feature Requests**: Describe the problem your feature would solve
- **Testing**: Help test unreleased features and report issues
- **Translation**: Help localize CogniPace to other languages

## 🛠️ Development Setup

### Prerequisites
- Rust 1.70+
- Python 3.10+ (for ML components)
- Node.js 18+ (for dashboard UI)
- Webcam (for eye tracking development)

### Quick Setup
```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/cognipace.git
cd cognipace

# Add upstream remote
git remote add upstream https://github.com/cognipace/cognipace.git

# Run setup script
./scripts/setup.sh

# Verify setup
cargo test
```

## 📝 Coding Standards

### Rust
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Document public APIs with `///` comments

### Python
- Follow [PEP 8](https://pep8.org/)
- Use type hints
- Run `black` and `isort` before committing
- Write docstrings for all public functions

### Commit Messages
We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new feature
fix: fix bug in existing feature
docs: update documentation
refactor: code refactoring
test: add or update tests
chore: maintenance tasks
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p cognipace-core

# Run with coverage
cargo tarpaulin --out Html

# Run Python tests
cd python && pytest
```

## 📋 Pull Request Process

1. **Update documentation** if you change behavior
2. **Add tests** for new features
3. **Update CHANGELOG.md** with your changes
4. **Ensure CI passes** (tests, linting, build)
5. **Request review** from maintainers

### PR Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No clippy warnings
- [ ] Code formatted with cargo fmt

## 🏷️ Issue Labels

- `good first issue` - Great for newcomers
- `bug` - Something isn't working
- `enhancement` - New feature or improvement
- `documentation` - Documentation improvements
- `help wanted` - Extra attention needed
- `question` - Further information requested

## 💬 Communication

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, show-and-tell
- **Discord**: Real-time chat with the community

## 📜 Code of Conduct

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) to understand the standards we expect from our community.

## 🎯 Areas Needing Help

| Area | Priority | Skills Needed |
|------|----------|---------------|
| Linux sensor integration | 🔴 High | evdev, DBus, X11/Wayland |
| ML model training | 🔴 High | PyTorch, time-series |
| Windows sensor integration | 🟡 Medium | Win32 API, UWP |
| Browser extension | 🟡 Medium | WebExtensions API |
| Dashboard UI | 🟡 Medium | Svelte, Tauri |
| Documentation | 🟢 Ongoing | Technical writing |

## 🙏 Thank You

Every contribution, no matter how small, helps make CogniPace better for everyone. We appreciate your time and effort!
