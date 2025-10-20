#!/usr/bin/env python3
"""
Comprehensive verification script to ensure complete Ollama removal.
Scans entire codebase for any remaining Ollama dependencies.
"""

import os
import re
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple

# Directories to scan
SCAN_DIRS = [
    "packages/sutra-hybrid",
    "packages/sutra-storage", 
    "packages/sutra-api",
    "packages/sutra-core",
    "packages/sutra-bulk-ingester",
]

# Files to scan
SCAN_FILES = [
    "docker-compose-grid.yml",
    "docker-compose-with-ingester.yml",
    "WARP.md",
]

# Ollama-related patterns to search for
OLLAMA_PATTERNS = [
    r"ollama",
    r"OLLAMA", 
    r"11434",
    r"OllamaNLP",
    r"OllamaEmbedding",
    r"SUTRA_OLLAMA_URL",
    r"SUTRA_EMBEDDING_MODEL.*nomic-embed-text",
]

def scan_file(file_path: Path) -> List[Tuple[int, str]]:
    """Scan a single file for Ollama patterns."""
    matches = []
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line_num, line in enumerate(f, 1):
                for pattern in OLLAMA_PATTERNS:
                    if re.search(pattern, line, re.IGNORECASE):
                        matches.append((line_num, line.strip()))
                        break  # Only record each line once
    except (UnicodeDecodeError, PermissionError):
        # Skip binary files or permission denied
        pass
    
    return matches

def scan_directory(base_dir: Path, scan_dir: str) -> Dict[str, List[Tuple[int, str]]]:
    """Scan all Python and configuration files in a directory."""
    results = {}
    dir_path = base_dir / scan_dir
    
    if not dir_path.exists():
        return results
    
    # File extensions to scan
    extensions = ['.py', '.rs', '.yml', '.yaml', '.md', '.toml', '.txt', '.sh']
    
    for file_path in dir_path.rglob('*'):
        if file_path.is_file() and file_path.suffix in extensions:
            # Skip node_modules and other irrelevant directories
            if any(part in str(file_path) for part in ['node_modules', '__pycache__', '.git', 'target']):
                continue
                
            matches = scan_file(file_path)
            if matches:
                relative_path = str(file_path.relative_to(base_dir))
                results[relative_path] = matches
    
    return results

def main():
    """Main verification function."""
    print("🔍 Comprehensive Ollama Removal Verification")
    print("=" * 50)
    
    base_dir = Path(".")
    all_results = {}
    
    # Scan directories
    for scan_dir in SCAN_DIRS:
        print(f"\n📁 Scanning {scan_dir}...")
        results = scan_directory(base_dir, scan_dir)
        all_results.update(results)
    
    # Scan individual files
    for scan_file_name in SCAN_FILES:
        file_path = base_dir / scan_file_name
        if file_path.exists():
            print(f"\n📄 Scanning {scan_file_name}...")
            matches = scan_file(file_path)
            if matches:
                all_results[scan_file_name] = matches
    
    # Report results
    print(f"\n📊 Scan Results")
    print("=" * 50)
    
    if not all_results:
        print("✅ NO Ollama references found! System is clean.")
        return True
    
    print(f"⚠️  Found Ollama references in {len(all_results)} files:")
    
    for file_path, matches in all_results.items():
        print(f"\n📍 {file_path}:")
        for line_num, line_content in matches:
            print(f"   Line {line_num}: {line_content}")
    
    # Categorize issues
    print(f"\n📋 Issue Categories:")
    
    active_runtime_issues = []
    documentation_issues = []
    test_issues = []
    
    for file_path, matches in all_results.items():
        if any(ext in file_path for ext in ['.py', '.rs', '.yml', '.yaml']):
            if 'test' in file_path.lower():
                test_issues.append(file_path)
            else:
                active_runtime_issues.append(file_path)
        else:
            documentation_issues.append(file_path)
    
    if active_runtime_issues:
        print(f"🚨 CRITICAL: {len(active_runtime_issues)} runtime files need cleanup:")
        for file_path in active_runtime_issues:
            print(f"   - {file_path}")
    
    if test_issues:
        print(f"🧪 TESTS: {len(test_issues)} test files need cleanup:")
        for file_path in test_issues:
            print(f"   - {file_path}")
    
    if documentation_issues:
        print(f"📖 DOCS: {len(documentation_issues)} documentation files need cleanup:")
        for file_path in documentation_issues:
            print(f"   - {file_path}")
    
    return len(active_runtime_issues) == 0

def check_environment_defaults():
    """Check Python code for problematic environment variable defaults."""
    print(f"\n🔧 Checking Environment Variable Defaults")
    print("-" * 40)
    
    base_dir = Path(".")
    issues_found = False
    
    # Patterns for problematic defaults
    env_patterns = [
        (r'os\.getenv.*SUTRA_EMBEDDING_PROVIDER.*ollama', "SUTRA_EMBEDDING_PROVIDER should default to 'service'"),
        (r'os\.getenv.*SUTRA_OLLAMA_URL', "SUTRA_OLLAMA_URL should be removed"),
        (r'os\.getenv.*11434', "Port 11434 references should be removed"),
        (r'OllamaNLP.*\(', "OllamaNLPProcessor usage should be removed"),
    ]
    
    for scan_dir in SCAN_DIRS:
        dir_path = base_dir / scan_dir
        if not dir_path.exists():
            continue
            
        for file_path in dir_path.rglob('*.py'):
            if 'node_modules' in str(file_path) or '__pycache__' in str(file_path):
                continue
                
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                    for pattern, issue_desc in env_patterns:
                        if re.search(pattern, content, re.IGNORECASE):
                            print(f"⚠️  {file_path.relative_to(base_dir)}: {issue_desc}")
                            issues_found = True
            except (UnicodeDecodeError, PermissionError):
                pass
    
    if not issues_found:
        print("✅ No problematic environment variable defaults found!")
    
    return not issues_found

if __name__ == "__main__":
    runtime_clean = main()
    env_clean = check_environment_defaults()
    
    print(f"\n🏁 Final Result")
    print("=" * 50)
    
    if runtime_clean and env_clean:
        print("🎉 SUCCESS: System is completely Ollama-free!")
        exit(0)
    else:
        print("❌ FAILED: Ollama dependencies still found. Clean up required.")
        exit(1)