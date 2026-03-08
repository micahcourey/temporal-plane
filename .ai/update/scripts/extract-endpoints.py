#!/usr/bin/env python3
"""
extract-endpoints.py — Scan API route files and output endpoints.jsonl

Auto-discovers Express/NestJS route files across workspace repositories,
extracts HTTP endpoints with methods and paths, detects privilege/permission
references near routes, and writes structured JSONL output.

Usage:
    # Scan workspace and print to stdout (preview)
    python3 setup/scripts/extract-endpoints.py /path/to/workspace

    # Write to file
    python3 setup/scripts/extract-endpoints.py /path/to/workspace \
        --output .ai/context/endpoints.jsonl

    # Exclude specific repos
    python3 setup/scripts/extract-endpoints.py /path/to/workspace \
        --output .ai/context/endpoints.jsonl \
        --exclude ai-dx-toolkit,node_modules

    # Only scan specific repos (comma-separated)
    python3 setup/scripts/extract-endpoints.py /path/to/workspace \
        --only agreement-management-api,idm-api,storage-api
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple


# ---------------------------------------------------------------------------
# Route file discovery patterns
# ---------------------------------------------------------------------------

ROUTE_FILE_GLOBS = [
    "**/*.routes.ts",
    "**/*.routes.js",
    "**/*.router.ts",
    "**/*.router.js",
    "**/routes.ts",
    "**/routes.js",
    "**/*.controller.ts",
    "**/*.controller.js",
]

# Directories to always skip when scanning for route files
SKIP_DIRS = {
    "node_modules", "dist", "build", ".git", "coverage",
    "test", "tests", "__tests__", "spec", "specs",
    "e2e", "fixtures", "mocks", "__mocks__",
}


# ---------------------------------------------------------------------------
# Route extraction patterns
# ---------------------------------------------------------------------------

# Express: router.get('/path', ...) or router.route('/path').get(...)
EXPRESS_ROUTE_RE = re.compile(
    r"router\.(get|post|put|delete|patch)\s*\(\s*['\"]([^'\"]+)['\"]",
    re.IGNORECASE,
)

# Express: app.get('/path', ...) — sometimes used directly
APP_ROUTE_RE = re.compile(
    r"app\.(get|post|put|delete|patch)\s*\(\s*['\"]([^'\"]+)['\"]",
    re.IGNORECASE,
)

# NestJS decorators: @Get('/path'), @Post(), etc.
NESTJS_ROUTE_RE = re.compile(
    r"@(Get|Post|Put|Delete|Patch)\s*\(\s*['\"]?([^'\")\s]*)['\"]?\s*\)",
)

# NestJS controller decorator: @Controller('/base')
NESTJS_CONTROLLER_RE = re.compile(
    r"@Controller\s*\(\s*['\"]([^'\"]+)['\"]",
)


# ---------------------------------------------------------------------------
# Base path detection patterns
# ---------------------------------------------------------------------------

# app.use('/base', router) or app.use('/base', require('./routes'))
APP_USE_RE = re.compile(
    r"app\.use\s*\(\s*['\"]([^'\"]+)['\"]",
)

# serverless.yml path pattern: path: /api/v1/resource/{proxy+}
SERVERLESS_PATH_RE = re.compile(
    r"^\s+path:\s*['\"]?(/[^\s'\"]+)",
    re.MULTILINE,
)


# ---------------------------------------------------------------------------
# Privilege / permission detection
# ---------------------------------------------------------------------------

# Look for UPPER_CASE identifiers near routes (likely privilege constants)
PRIVILEGE_RE = re.compile(r"['\"]([A-Z][A-Z_]{4,})['\"]")

# Common false positives to exclude
PRIVILEGE_EXCLUDES = {
    "EXPRESS", "CONFIG", "ROUTER", "DEFAULT", "ERROR", "DELETE",
    "PATCH", "QUERY", "PARAM", "SECURE", "PUBLIC", "INDEX",
    "STRING", "NUMBER", "BOOLEAN", "OBJECT", "ARRAY", "METHOD",
    "CONTENT", "ACCEPT", "ORIGIN", "HEADER", "COOKIE", "TOKEN",
    "BEARER", "BASIC", "HTTPS", "MULTIPART", "URLENCODED",
    "APPLICATION", "CHARSET", "BUFFER", "STREAM", "ASYNC",
    "AWAIT", "PROMISE", "EXPORT", "IMPORT", "MODULE", "REQUIRE",
    "FUNCTION", "RETURN", "CLASS", "INTERFACE", "CONST", "TYPE",
    "UNDEFINED", "NULLISH", "PRIVATE", "PROTECTED", "READONLY",
    "STATIC", "ABSTRACT", "IMPLEMENTS", "EXTENDS", "THROW",
    "CATCH", "FINALLY", "SWITCH", "BREAK", "CONTINUE", "WHILE",
    "PLAIN", "SUCCESS", "FAILED", "STATUS", "RESPONSE", "REQUEST",
    "MIDDLEWARE", "HANDLER", "CONTROLLER", "SERVICE", "MODEL",
    "ENTITY", "SCHEMA", "TABLE", "COLUMN", "FIELD", "RECORD",
    "CREATE", "UPDATE", "INSERT", "SELECT", "WHERE", "ORDER",
    "LIMIT", "OFFSET", "COUNT", "GROUP", "HAVING", "INNER",
    "OUTER", "CROSS", "UNION", "EXIST",
}


# ---------------------------------------------------------------------------
# Discovery & extraction
# ---------------------------------------------------------------------------

def find_route_files(repo_path: Path) -> List[Path]:
    """Find all route/controller files in a repo, skipping irrelevant dirs."""
    route_files = []
    for root, dirs, files in os.walk(repo_path):
        # Prune skip directories
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]

        root_path = Path(root)
        for fname in files:
            fpath = root_path / fname
            # Match against route file patterns
            if any(fpath.match(pattern) for pattern in [
                "*.routes.ts", "*.routes.js",
                "*.router.ts", "*.router.js",
                "*.controller.ts", "*.controller.js",
            ]):
                route_files.append(fpath)
            # Also match routes.ts/routes.js directly
            elif fname in ("routes.ts", "routes.js"):
                route_files.append(fpath)
    return sorted(route_files)


def detect_base_path_from_app(repo_path: Path) -> Optional[str]:
    """Try to detect base path from app.ts/index.ts app.use() calls."""
    candidates = [
        repo_path / "src" / "app.ts",
        repo_path / "src" / "app.js",
        repo_path / "src" / "index.ts",
        repo_path / "src" / "index.js",
        repo_path / "app.ts",
        repo_path / "app.js",
    ]
    for candidate in candidates:
        if candidate.exists():
            try:
                content = candidate.read_text(errors="replace")
                matches = APP_USE_RE.findall(content)
                # Find the most specific base path (usually the API prefix)
                api_paths = [m for m in matches if "/api" in m.lower()
                             or "/v1" in m or "/v2" in m
                             or "/secure" in m]
                if api_paths:
                    return api_paths[0].rstrip("/")
                # Fall back to first non-trivial path
                for m in matches:
                    if m not in ("/", "/health", "/healthcheck", "/status"):
                        return m.rstrip("/")
            except Exception:
                pass
    return None


def detect_base_path_from_serverless(repo_path: Path) -> Optional[str]:
    """Try to detect base path from serverless.yml or SAM template."""
    candidates = [
        repo_path / "serverless.yml",
        repo_path / "serverless.yaml",
        repo_path / "template.yaml",
        repo_path / "template.yml",
    ]
    for candidate in candidates:
        if candidate.exists():
            try:
                content = candidate.read_text(errors="replace")
                paths = SERVERLESS_PATH_RE.findall(content)
                if paths:
                    # Extract common prefix from all paths
                    parts_list = [p.split("/") for p in paths]
                    if parts_list:
                        common = []
                        for i, part in enumerate(parts_list[0]):
                            if all(
                                len(pl) > i and pl[i] == part
                                for pl in parts_list
                            ) and not part.startswith("{"):
                                common.append(part)
                            else:
                                break
                        if common:
                            return "/".join(common).rstrip("/") or None
            except Exception:
                pass
    return None


def infer_base_path_from_name(repo_name: str) -> str:
    """Infer a base path from the repo name as a fallback."""
    # Strip common suffixes
    name = repo_name
    for suffix in ("-api", "-svc", "-service", "-server", "-backend"):
        if name.endswith(suffix):
            name = name[: -len(suffix)]
            break
    return f"/{name}"


def detect_base_path(repo_path: Path) -> str:
    """Detect the API base path for a repo using multiple strategies."""
    # Try app.use() detection first
    base = detect_base_path_from_app(repo_path)
    if base:
        return base

    # Try serverless.yml
    base = detect_base_path_from_serverless(repo_path)
    if base:
        return base

    # Fall back to repo name inference
    return infer_base_path_from_name(repo_path.name)


def extract_privilege_near(content: str, pos: int, window: int = 300) -> Optional[str]:
    """Look for a privilege/permission constant near a route definition."""
    nearby = content[pos: pos + window]
    for match in PRIVILEGE_RE.finditer(nearby):
        candidate = match.group(1)
        if candidate not in PRIVILEGE_EXCLUDES and len(candidate) > 5:
            return candidate
    return None


def description_from_path(path: str) -> str:
    """Generate a human-readable description from a route path."""
    parts = [p for p in path.split("/") if p and not p.startswith(":")]
    if not parts:
        return path
    desc_parts = [p.replace("-", " ").replace("_", " ") for p in parts]
    return " ".join(desc_parts).title()


def extract_express_routes(
    content: str, base_path: str
) -> List[Dict[str, Any]]:
    """Extract Express-style routes from file content."""
    endpoints = []
    for match in EXPRESS_ROUTE_RE.finditer(content):
        method = match.group(1).upper()
        path = match.group(2)
        privilege = extract_privilege_near(content, match.end())

        # Normalize path joining
        if path.startswith("/"):
            full_path = base_path + path
        else:
            full_path = base_path + "/" + path

        ep: Dict[str, Any] = {
            "method": method,
            "path": full_path,
            "description": description_from_path(path),
            "auth_required": True,
        }
        if privilege:
            ep["privilege"] = privilege
        endpoints.append(ep)

    # Also check app.METHOD patterns
    for match in APP_ROUTE_RE.finditer(content):
        method = match.group(1).upper()
        path = match.group(2)
        privilege = extract_privilege_near(content, match.end())

        if path.startswith("/"):
            full_path = base_path + path
        else:
            full_path = base_path + "/" + path

        ep = {
            "method": method,
            "path": full_path,
            "description": description_from_path(path),
            "auth_required": True,
        }
        if privilege:
            ep["privilege"] = privilege
        endpoints.append(ep)

    return endpoints


def extract_nestjs_routes(
    content: str, base_path: str
) -> List[Dict[str, Any]]:
    """Extract NestJS-style routes from file content."""
    endpoints = []

    # Detect controller-level base path
    ctrl_match = NESTJS_CONTROLLER_RE.search(content)
    ctrl_base = ""
    if ctrl_match:
        ctrl_base = ctrl_match.group(1).rstrip("/")

    for match in NESTJS_ROUTE_RE.finditer(content):
        method = match.group(1).upper()
        path = match.group(2) or ""
        privilege = extract_privilege_near(content, match.end())

        # Build full path
        parts = [base_path, ctrl_base]
        if path:
            parts.append(path if path.startswith("/") else "/" + path)
        full_path = "".join(parts) or "/"

        ep: Dict[str, Any] = {
            "method": method,
            "path": full_path,
            "description": description_from_path(path or ctrl_base),
            "auth_required": True,
        }
        if privilege:
            ep["privilege"] = privilege
        endpoints.append(ep)

    return endpoints


def detect_framework(content: str) -> str:
    """Detect whether a file uses Express or NestJS patterns."""
    has_express = bool(EXPRESS_ROUTE_RE.search(content) or APP_ROUTE_RE.search(content))
    has_nestjs = bool(NESTJS_ROUTE_RE.search(content))

    if has_nestjs and not has_express:
        return "nestjs"
    return "express"  # Default to Express


def is_api_repo(repo_path: Path) -> bool:
    """Check if a repo looks like an API service."""
    name = repo_path.name.lower()
    # Explicit API indicators in name
    if any(kw in name for kw in ("api", "svc", "service", "server", "backend")):
        return True
    # Has src/ with route-like files
    src = repo_path / "src"
    if src.is_dir():
        route_files = find_route_files(repo_path)
        if route_files:
            return True
    return False


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def scan_workspace(
    workspace: Path,
    exclude: Set[str],
    only: Optional[Set[str]] = None,
) -> List[Dict[str, Any]]:
    """Scan all API repos in workspace and extract endpoints."""
    all_endpoints: List[Dict[str, Any]] = []

    # Enumerate top-level directories
    repos = sorted(
        d for d in workspace.iterdir()
        if d.is_dir()
        and not d.name.startswith(".")
        and d.name not in exclude
    )

    if only:
        repos = [r for r in repos if r.name in only]

    for repo_path in repos:
        if not is_api_repo(repo_path):
            continue

        service_name = repo_path.name
        base_path = detect_base_path(repo_path)
        route_files = find_route_files(repo_path)

        if not route_files:
            continue

        repo_endpoints = []
        for route_file in route_files:
            try:
                content = route_file.read_text(errors="replace")
            except Exception:
                continue

            framework = detect_framework(content)
            if framework == "nestjs":
                eps = extract_nestjs_routes(content, base_path)
            else:
                eps = extract_express_routes(content, base_path)

            # Tag with service name and source file
            for ep in eps:
                ep["service"] = service_name
                # Determine auth from path segments
                path_lower = ep["path"].lower()
                if "/public" in path_lower or "/health" in path_lower:
                    ep["auth_required"] = False

            repo_endpoints.extend(eps)

        if repo_endpoints:
            print(
                f"  {service_name}: {len(repo_endpoints)} endpoints "
                f"(base: {base_path}, {len(route_files)} route files)",
                file=sys.stderr,
            )
        all_endpoints.extend(repo_endpoints)

    return all_endpoints


def deduplicate(endpoints: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Remove duplicate endpoints (same service + method + path)."""
    seen: Set[Tuple[str, str, str]] = set()
    unique = []
    for ep in endpoints:
        key = (ep["service"], ep["method"], ep["path"])
        if key not in seen:
            seen.add(key)
            unique.append(ep)
    return unique


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Scan workspace API repos for HTTP endpoints → JSONL",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s /path/to/workspace
  %(prog)s /path/to/workspace --output .ai/context/endpoints.jsonl
  %(prog)s /path/to/workspace --exclude ai-dx-toolkit,docs
  %(prog)s /path/to/workspace --only agreement-management-api,idm-api
        """,
    )
    parser.add_argument(
        "workspace",
        help="Path to the workspace root containing repo directories",
    )
    parser.add_argument(
        "--output", "-o",
        help="Output file path (default: stdout)",
    )
    parser.add_argument(
        "--exclude", "-e",
        default="",
        help="Comma-separated repo names to exclude",
    )
    parser.add_argument(
        "--only",
        default="",
        help="Comma-separated repo names to scan (ignores all others)",
    )

    args = parser.parse_args()
    workspace = Path(args.workspace).resolve()

    if not workspace.is_dir():
        print(f"Error: {workspace} is not a directory", file=sys.stderr)
        sys.exit(1)

    # Build exclude set
    exclude = {e.strip() for e in args.exclude.split(",") if e.strip()}
    exclude.update({".git", "node_modules", "__pycache__"})

    # Build only set
    only = None
    if args.only:
        only = {o.strip() for o in args.only.split(",") if o.strip()}

    print(f"Scanning {workspace} for API endpoints...", file=sys.stderr)
    endpoints = scan_workspace(workspace, exclude, only)

    # Deduplicate and sort
    endpoints = deduplicate(endpoints)
    endpoints.sort(key=lambda x: (x["service"], x["path"], x["method"]))

    # Output
    lines = [json.dumps(ep, ensure_ascii=False) for ep in endpoints]
    output_text = "\n".join(lines) + "\n" if lines else ""

    if args.output:
        out_path = Path(args.output)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(output_text)
        print(f"Wrote {len(endpoints)} endpoints → {args.output}", file=sys.stderr)
    else:
        sys.stdout.write(output_text)

    # Summary
    services = sorted(set(ep["service"] for ep in endpoints))
    print(
        f"\n✅ {len(endpoints)} unique endpoints across {len(services)} services",
        file=sys.stderr,
    )
    from collections import Counter

    svc_counts = Counter(ep["service"] for ep in endpoints)
    for svc, count in sorted(svc_counts.items()):
        print(f"  {svc}: {count} endpoints", file=sys.stderr)


if __name__ == "__main__":
    main()
