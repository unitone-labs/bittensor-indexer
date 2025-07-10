# Contributing to Flamewire Bittensor Indexer

Thank you for your interest in contributing to the Flamewire Bittensor Indexer! We welcome contributions from the community.

## Contributor License Agreement

By contributing to this project, you agree that:

1. **Your contributions are your own work** and you have the right to license them
2. **You grant the Flamewire project a perpetual, worldwide, non-exclusive, royalty-free license** to use, modify, and distribute your contributions
3. **Your contributions will be licensed under the Apache 2.0 License**
4. **You understand that the Flamewire project maintainers retain control** of the project direction and final decisions

## Legal Requirements

### Copyright Assignment

All contributions must include the following copyright header in new files:

```rust
/*
 * Copyright 2025 Flamewire
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
```

### Signed Commits (Recommended)

For security, we recommend signing your commits with GPG:

```bash
git commit -S -m "your commit message"
```

To set up signed commits:
```bash
# Generate GPG key
gpg --gen-key

# Get your key ID
gpg --list-secret-keys --keyid-format LONG

# Configure Git
git config user.signingkey YOUR_KEY_ID
git config commit.gpgsign true
```

## How to Contribute

### 1. Fork and Clone

```bash
git clone https://github.com/your-username/bittensor-indexer.git
cd bittensor-indexer
```

### 2. Create Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Follow our [Code Style Guidelines](#code-style)
- Add tests for new functionality
- Update documentation as needed
- Ensure all examples still work

### 4. Test Your Changes

```bash
# Run all tests
cargo test --all-features

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Test examples
cargo build --examples --all-features
```

### 5. Submit Pull Request

- Fill out the PR template completely
- Reference any related issues
- Ensure all CI checks pass
- Request review from maintainers

## Code Style Guidelines

### Rust Code Style

- Use `cargo fmt` with default settings
- Follow Rust naming conventions
- Document all public APIs with rustdoc
- Use meaningful variable and function names
- Prefer explicit error handling over unwrap/expect

### Documentation

- Document all public APIs
- Include examples in documentation
- Update README if adding new features
- Write clear commit messages

### Error Handling

- Use custom error types from `crate::error::IndexerError`
- Provide helpful error messages
- Include context information in errors
- Handle errors gracefully

## Review Process

1. **Automated Checks**: All CI checks must pass
2. **Code Review**: At least one maintainer review required
3. **Testing**: Verify functionality works as expected
4. **Documentation**: Check that docs are updated

## Types of Contributions

### Bug Fixes
- Always include a test that reproduces the bug
- Reference the issue number in commit message
- Keep changes minimal and focused

### New Features
- Discuss in an issue before implementing
- Include comprehensive tests
- Update documentation and examples
- Consider backward compatibility

### Documentation
- Fix typos, improve clarity
- Add examples and use cases
- Update outdated information

### Performance Improvements
- Include benchmarks showing improvement
- Verify no regression in functionality
- Document any trade-offs

## Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow professional communication standards

### Communication

- Use GitHub issues for bug reports and feature requests
- Join discussions in pull requests
- Ask questions if anything is unclear
- Provide helpful feedback to other contributors

## Getting Help

- Check existing issues and documentation first
- Create a new issue with detailed information
- Tag maintainers if you need urgent help
- Join our community discussions

## Recognition

Contributors will be:
- Listed in the project's contributors section
- Credited in release notes for significant contributions
- Invited to join the contributor team for ongoing contributors

## Legal Notice

By contributing to this project, you acknowledge that:

- The Flamewire project maintainers retain control of the project
- Your contributions become part of the project's intellectual property
- You cannot claim ownership of the project or derivative works
- All contributions are subject to the Apache 2.0 License terms

Thank you for contributing to the Flamewire Bittensor Indexer!