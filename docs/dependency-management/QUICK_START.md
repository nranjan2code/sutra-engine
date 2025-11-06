# Dependency Management Quick Start Guide

Get started with Sutra's dependency management system in 5 minutes.

## 🚀 Quick Setup

### 1. Install Required Tools

```bash
# Python tools
pip install pip-audit pip-licenses

# Rust tools (optional but recommended)
cargo install cargo-audit cargo-outdated

# Node tools are included with npm
```

### 2. Run Your First Scan

#### Option A: Local Command Line
```bash
# Make script executable (first time only)
chmod +x scripts/scan-dependencies.sh

# Run scan
./scripts/scan-dependencies.sh
```

#### Option B: Control Center UI
```bash
# Start services
sutra start

# Open browser
open http://localhost:9000

# Navigate to Dependencies tab
# Click "Run Scan" button
```

## 📊 Understanding the Results

### Health Score
- **80-100**: Excellent - No critical issues
- **60-79**: Good - Minor issues to address
- **40-59**: Fair - Several updates needed
- **0-39**: Poor - Immediate action required

### Vulnerability Severities
- 🔴 **Critical**: Fix immediately (blocks PRs)
- 🟠 **High**: Fix within 24 hours
- 🟡 **Medium**: Fix within 1 week
- 🟢 **Low**: Fix in next release
- ℹ️ **Info**: For awareness only

## 🔧 Common Tasks

### Check for Vulnerabilities
```bash
# Quick vulnerability check
./scripts/scan-dependencies.sh | grep -E "(CRITICAL|HIGH)"

# Or via API
curl http://localhost:8000/api/dependencies/vulnerabilities?severity=critical
```

### Generate SBOM
```bash
# Via Control Center UI
# Click "Download SBOM" button

# Or via API
curl http://localhost:8000/api/dependencies/sbom > sbom.json
```

### Update Dependencies

#### Python
```bash
# Update specific package
pip install --upgrade package-name

# Update all outdated
pip list --outdated --format=freeze | grep -v '^\-e' | cut -d = -f 1 | xargs -n1 pip install -U
```

#### Rust
```bash
# Update all dependencies
cargo update

# Update specific dependency
cargo update -p dependency-name
```

#### Node.js
```bash
# Update all dependencies
npm update

# Check outdated first
npm outdated

# Update specific package
npm install package-name@latest
```

## 🤖 Automation

### Enable GitHub Actions
```bash
# The workflow is already created
git add .github/workflows/dependency-security.yml
git commit -m "Enable dependency scanning"
git push
```

### Enable Dependabot
```bash
# Configuration is already created
git add .github/dependabot.yml
git commit -m "Enable Dependabot"
git push
```

### Schedule Regular Scans
```bash
# Add to crontab for daily scans
crontab -e

# Add this line (daily at 2 AM)
0 2 * * * cd /path/to/sutra-models && ./scripts/scan-dependencies.sh >> logs/dep-scan.log 2>&1
```

## 🚨 Responding to Vulnerabilities

### Critical Vulnerability Found

1. **Don't Panic** - Most have fixes available
2. **Check Details** - Review the vulnerability description
3. **Update Package** - Install the fixed version
4. **Verify Fix** - Re-run scan to confirm
5. **Document** - Note the fix in your changelog

### Example Response
```bash
# 1. Identify the vulnerable package
./scripts/scan-dependencies.sh | grep CRITICAL

# 2. Check available fix
pip show vulnerable-package
pip index versions vulnerable-package

# 3. Update to fixed version
pip install vulnerable-package==fixed-version

# 4. Verify the fix
./scripts/scan-dependencies.sh | grep vulnerable-package

# 5. Commit the change
git add requirements.txt
git commit -m "Security: Fix critical vulnerability in vulnerable-package (CVE-2024-xxxxx)"
```

## 📋 Best Practices

### Daily Workflow
1. ✅ Check Control Center dashboard each morning
2. ✅ Review any Dependabot PRs
3. ✅ Address critical vulnerabilities immediately
4. ✅ Schedule updates for high vulnerabilities

### Weekly Workflow
1. ✅ Review all Dependabot PRs
2. ✅ Update development dependencies
3. ✅ Check license compliance
4. ✅ Generate and review SBOM

### Monthly Workflow
1. ✅ Full dependency audit
2. ✅ Clean up unused dependencies
3. ✅ Update scanner tools
4. ✅ Review security policies

## 🔍 Advanced Usage

### Custom Security Policy
Create `.security-policy.yml`:
```yaml
rules:
  - name: no-beta-versions
    pattern: ".*beta.*"
    action: warn
    
  - name: require-recent-updates
    max-age-days: 365
    action: error
```

### API Integration
```python
import requests

# Get dependency summary
response = requests.get("http://localhost:8000/api/dependencies/summary")
summary = response.json()

print(f"Health Score: {summary['health_score']}")
print(f"Vulnerabilities: {summary['vulnerable_dependencies']}")
```

### CI/CD Integration
```yaml
# In your CI pipeline
- name: Dependency Check
  run: |
    ./scripts/scan-dependencies.sh
    if [ $? -ne 0 ]; then
      echo "Critical vulnerabilities found!"
      exit 1
    fi
```

## 🆘 Troubleshooting

### Scanner Not Found
```bash
# Reinstall tools
pip install --upgrade pip-audit pip-licenses
cargo install --force cargo-audit
```

### Scan Takes Too Long
```bash
# Run scan for specific package only
cd packages/sutra-core
pip-audit
```

### Permission Denied
```bash
# Fix script permissions
chmod +x scripts/scan-dependencies.sh
```

### No Output/Empty Results
```bash
# Check if in correct directory
pwd  # Should be in project root

# Verify package files exist
ls -la requirements*.txt package.json Cargo.toml
```

## 📚 Further Reading

- [Full Architecture Documentation](./ARCHITECTURE.md)
- [Design Documentation](./DESIGN.md)
- [API Reference](./API_REFERENCE.md)
- [Security Best Practices](../security/SECURITY_BEST_PRACTICES.md)

## 💡 Tips & Tricks

1. **Use the `-v` flag** for verbose output in local scans
2. **Set up Slack notifications** for critical vulnerabilities
3. **Pin versions in production** to avoid surprise updates
4. **Test updates in staging** before production
5. **Keep scanner tools updated** for latest vulnerability data

## 🎯 Next Steps

1. ✅ Complete your first scan
2. ✅ Fix any critical vulnerabilities
3. ✅ Enable GitHub Actions workflow
4. ✅ Configure Dependabot
5. ✅ Set up regular scanning schedule

---

**Need Help?** Check the [troubleshooting guide](./TROUBLESHOOTING.md) or open an issue on GitHub.