# Production Dependency Lock Files

This directory contains pinned dependency versions for reproducible builds.

## Python Dependencies

Generate lock files:
```bash
# Activate virtual environment
source venv/bin/activate

# Generate lock file from pyproject.toml
pip freeze > requirements-lock.txt

# Or use pip-compile (recommended)
pip install pip-tools
pip-compile pyproject.toml -o requirements-lock.txt
```

Install from lock file:
```bash
pip install -r requirements-lock.txt
```

## JavaScript/TypeScript Dependencies  

Generate lock files:
```bash
# Using npm (creates package-lock.json)
npm install

# Using pnpm (creates pnpm-lock.yaml)  
pnpm install

# Using yarn (creates yarn.lock)
yarn install
```

Install from lock file:
```bash
npm ci  # Clean install from package-lock.json
# OR
pnpm install --frozen-lockfile
```

## Rust Dependencies

Cargo automatically generates `Cargo.lock` when building.

```bash
# Update dependencies
cargo update

# Commit Cargo.lock for reproducibility
git add Cargo.lock
git commit -m "chore: update Cargo.lock"
```

## Security Scanning

Run security audits regularly:
```bash
# Python
pip install safety
safety check -r requirements-lock.txt

# JavaScript
npm audit
npm audit fix

# Rust
cargo audit
cargo install cargo-audit  # if not installed
```

## Update Schedule

- **Security patches**: Apply immediately
- **Minor versions**: Monthly review
- **Major versions**: Quarterly review with testing

## Lock File Policy

- ✅ **ALWAYS commit lock files** to git
- ✅ **Review dependency changes** in PRs
- ✅ **Run security scans** before merging
- ✅ **Update regularly** to get security patches
- ❌ **Never manually edit** lock files
