#!/usr/bin/env python3
"""
Security Scanner

Scans TypeScript/JavaScript code for common security vulnerabilities:
- Missing authentication middleware
- Missing authorization middleware
- SQL injection risks
- Hardcoded secrets
- Stack trace exposure

Usage:
    python scan.py <file_or_directory>
    python scan.py src/api/routes/
    python scan.py --json  # Output as JSON

Exit codes:
    0 - No issues found
    1 - Issues found
    2 - Error during scan
"""

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import List, Optional

# Security patterns to detect
PATTERNS = {
    "missing_auth_middleware": {
        "severity": "critical",
        "pattern": r"router\.(get|post|put|patch|delete)\s*\(\s*['\"][^'\"]+['\"]\s*,\s*(?!.*verifyToken)",
        "message": "Route missing authentication middleware",
        "recommendation": "Add authentication middleware as first route handler"
    },
    "missing_permissions_middleware": {
        "severity": "high",
        "pattern": r"router\.(post|put|patch|delete)\s*\([^)]+verifyToken[^)]+(?!.*checkPermission|.*validateRequiredPrivilege)",
        "message": "Route has auth but missing permissions check",
        "recommendation": "Add permissions middleware after authentication"
    },
    "sql_injection": {
        "severity": "critical",
        "pattern": r"(query|execute)\s*\(\s*[`'\"].*\$\{.*\}|.*\+\s*\w+\s*\+",
        "message": "Possible SQL injection - string concatenation in query",
        "recommendation": "Use parameterized queries with $1, $2 placeholders"
    },
    "hardcoded_secret": {
        "severity": "critical",
        "pattern": r"(password|secret|api_?key|token|credential)\s*[=:]\s*['\"][^'\"]{8,}['\"]",
        "message": "Hardcoded secret detected",
        "recommendation": "Move to environment variable"
    },
    "stack_trace_exposure": {
        "severity": "high",
        "pattern": r"res\.(json|send)\s*\([^)]*error\.(stack|message)",
        "message": "Error details exposed to client",
        "recommendation": "Log error server-side, return generic message to client"
    },
    "missing_data_isolation": {
        "severity": "high",
        "pattern": r"\.where\s*\(\s*\{[^}]*participant_id[^}]*\}[^}]*(?!agreement_id|org_id|tenant_id)",
        "message": "Query missing data isolation filter",
        "recommendation": "Add tenant/org isolation filter to query"
    },
    "console_log_in_prod": {
        "severity": "medium",
        "pattern": r"console\.(log|error|warn)\s*\([^)]*\)",
        "message": "Console statement found - use logger instead",
        "recommendation": "Replace with logger.info/error/warn"
    }
}

@dataclass
class Issue:
    severity: str
    file: str
    line: int
    message: str
    recommendation: str
    code_snippet: str

def scan_file(file_path: Path) -> List[Issue]:
    """Scan a single file for security issues."""
    issues = []
    
    try:
        content = file_path.read_text(encoding='utf-8')
        lines = content.split('\n')
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return issues
    
    for pattern_name, pattern_info in PATTERNS.items():
        regex = re.compile(pattern_info["pattern"], re.IGNORECASE | re.MULTILINE)
        
        for i, line in enumerate(lines, 1):
            if regex.search(line):
                issues.append(Issue(
                    severity=pattern_info["severity"],
                    file=str(file_path),
                    line=i,
                    message=pattern_info["message"],
                    recommendation=pattern_info["recommendation"],
                    code_snippet=line.strip()[:100]
                ))
    
    return issues

def scan_directory(dir_path: Path, extensions: tuple = ('.ts', '.js')) -> List[Issue]:
    """Recursively scan directory for security issues."""
    issues = []
    
    for file_path in dir_path.rglob('*'):
        if file_path.suffix in extensions and 'node_modules' not in str(file_path):
            issues.extend(scan_file(file_path))
    
    return issues

def format_output(issues: List[Issue], as_json: bool = False) -> str:
    """Format issues for output."""
    if as_json:
        return json.dumps([asdict(i) for i in issues], indent=2)
    
    if not issues:
        return "✅ No security issues found"
    
    severity_icons = {
        "critical": "🔴",
        "high": "🟠",
        "medium": "🟡",
        "low": "🔵"
    }
    
    output = ["# Security Scan Results\n"]
    output.append(f"**Issues Found**: {len(issues)}\n")
    
    # Group by severity
    critical = [i for i in issues if i.severity == "critical"]
    high = [i for i in issues if i.severity == "high"]
    medium = [i for i in issues if i.severity == "medium"]
    
    output.append(f"- 🔴 Critical: {len(critical)}")
    output.append(f"- 🟠 High: {len(high)}")
    output.append(f"- 🟡 Medium: {len(medium)}\n")
    
    output.append("## Issues\n")
    output.append("| Severity | Location | Issue | Recommendation |")
    output.append("|----------|----------|-------|----------------|")
    
    for issue in sorted(issues, key=lambda x: ["critical", "high", "medium", "low"].index(x.severity)):
        icon = severity_icons.get(issue.severity, "⚪")
        location = f"{issue.file}:{issue.line}"
        output.append(f"| {icon} {issue.severity} | {location} | {issue.message} | {issue.recommendation} |")
    
    # Verdict
    output.append("\n## Verdict")
    if critical:
        output.append("❌ **BLOCKED** - Critical security issues must be fixed before merge")
    elif high:
        output.append("⚠️ **REVIEW** - High severity issues should be addressed")
    else:
        output.append("✅ **PASS** - No blocking issues")
    
    return "\n".join(output)

def main():
    parser = argparse.ArgumentParser(description="Security scanner")
    parser.add_argument("path", nargs="?", default=".", help="File or directory to scan")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    args = parser.parse_args()
    
    target = Path(args.path)
    
    if not target.exists():
        print(f"Error: {target} does not exist", file=sys.stderr)
        sys.exit(2)
    
    if target.is_file():
        issues = scan_file(target)
    else:
        issues = scan_directory(target)
    
    print(format_output(issues, args.json))
    
    # Exit with code 1 if critical or high issues found
    critical_or_high = any(i.severity in ("critical", "high") for i in issues)
    sys.exit(1 if critical_or_high else 0)

if __name__ == "__main__":
    main()
