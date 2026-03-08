#!/usr/bin/env python3
"""
scan-repos.py — Scan a workspace for repositories and output repositories.jsonl

Walks top-level directories looking for package manifests (package.json,
requirements.txt, go.mod, etc.), extracts metadata, and writes structured
JSONL output.

Usage:
    python3 setup/scripts/scan-repos.py /path/to/workspace
    python3 setup/scripts/scan-repos.py /path/to/workspace --output .ai/context/repositories.jsonl
    python3 setup/scripts/scan-repos.py /path/to/workspace --exclude ai-dx-toolkit,node_modules
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


# Package manifest files and what they indicate
MANIFEST_FILES = {
    "package.json": "node",
    "requirements.txt": "python",
    "Pipfile": "python",
    "pyproject.toml": "python",
    "go.mod": "go",
    "Cargo.toml": "rust",
    "pom.xml": "java",
    "build.gradle": "java",
    "build.gradle.kts": "kotlin",
    "Gemfile": "ruby",
    "mix.exs": "elixir",
    "composer.json": "php",
}

# Framework detection files
FRAMEWORK_INDICATORS = {
    "angular.json": "Angular",
    "next.config.js": "Next.js",
    "next.config.ts": "Next.js",
    "next.config.mjs": "Next.js",
    "nuxt.config.js": "Nuxt.js",
    "nuxt.config.ts": "Nuxt.js",
    "vite.config.js": "Vite",
    "vite.config.ts": "Vite",
    "svelte.config.js": "SvelteKit",
    "gatsby-config.js": "Gatsby",
    "remix.config.js": "Remix",
    "serverless.yml": "Serverless",
    "serverless.yaml": "Serverless",
    "template.yaml": "SAM",
    "samconfig.toml": "SAM",
    "Dockerfile": "Docker",
    "docker-compose.yml": "Docker Compose",
    "docker-compose.yaml": "Docker Compose",
}

# Repo type classification keywords
TYPE_KEYWORDS = {
    "ui": ["ui", "frontend", "app", "portal", "dashboard", "web"],
    "api": ["api", "service", "svc", "server", "backend", "gateway"],
    "lib": ["lib", "library", "shared", "common", "core", "utils", "utility"],
    "batch": ["batch", "job", "worker", "processor", "scheduler", "cron"],
    "cli": ["cli", "tool", "script"],
    "e2e": ["e2e", "test", "automation", "qa"],
    "infra": ["infra", "devops", "deploy", "terraform", "cdk", "pipeline", "jenkins", "codebuild"],
    "docs": ["docs", "documentation", "wiki"],
    "config": ["config", "configuration", "settings"],
}

DEFAULT_EXCLUDES = {"ai-dx-toolkit", "node_modules", ".git", "__pycache__", ".venv", "venv"}


def classify_repo_type(name: str, description: str = "") -> str:
    """Classify repo type based on name and description."""
    text = f"{name} {description}".lower()
    for repo_type, keywords in TYPE_KEYWORDS.items():
        for keyword in keywords:
            if keyword in text:
                return repo_type
    return "unknown"


def detect_tech_stack(repo_path: Path) -> List[str]:
    """Detect technologies used in a repo."""
    stack = []

    # Check frameworks
    for indicator_file, framework in FRAMEWORK_INDICATORS.items():
        if (repo_path / indicator_file).exists():
            stack.append(framework)

    # Check package.json for additional info
    pkg_path = repo_path / "package.json"
    if pkg_path.exists():
        try:
            with open(pkg_path, "r") as f:
                pkg = json.load(f)
            deps = {**pkg.get("dependencies", {}), **pkg.get("devDependencies", {})}

            if "@angular/core" in deps:
                if "Angular" not in stack:
                    stack.append("Angular")
            if "react" in deps:
                stack.append("React")
            if "vue" in deps:
                stack.append("Vue")
            if "express" in deps:
                stack.append("Express")
            if "fastify" in deps:
                stack.append("Fastify")
            if "nestjs" in deps or "@nestjs/core" in deps:
                stack.append("NestJS")
            if "typescript" in deps:
                stack.append("TypeScript")

        except (json.JSONDecodeError, OSError):
            pass

    # Check for TypeScript config
    if (repo_path / "tsconfig.json").exists() and "TypeScript" not in stack:
        stack.append("TypeScript")

    return stack if stack else ["Unknown"]


def detect_dependencies(repo_path: Path, all_repo_names: List[str]) -> List[str]:
    """Detect dependencies on other repos in the workspace."""
    deps = []
    pkg_path = repo_path / "package.json"

    if pkg_path.exists():
        try:
            with open(pkg_path, "r") as f:
                pkg = json.load(f)
            all_deps = {**pkg.get("dependencies", {}), **pkg.get("devDependencies", {})}

            for dep_name in all_deps:
                # Check if any workspace repo name appears in the dependency
                clean_dep = dep_name.split("/")[-1]  # handle scoped packages
                for repo_name in all_repo_names:
                    if repo_name in clean_dep or clean_dep in repo_name:
                        if repo_name != repo_path.name:
                            deps.append(repo_name)
        except (json.JSONDecodeError, OSError):
            pass

    return list(set(deps))


def read_description(repo_path: Path) -> str:
    """Read repo description from package.json or README."""
    # Try package.json first
    pkg_path = repo_path / "package.json"
    if pkg_path.exists():
        try:
            with open(pkg_path, "r") as f:
                pkg = json.load(f)
            desc = pkg.get("description", "")
            if desc:
                return desc
        except (json.JSONDecodeError, OSError):
            pass

    # Try README.md — first non-empty, non-heading line
    readme_path = repo_path / "README.md"
    if readme_path.exists():
        try:
            with open(readme_path, "r") as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#") and not line.startswith("!") and not line.startswith("---"):
                        return line[:200]  # Truncate long descriptions
        except OSError:
            pass

    return ""


def scan_workspace(workspace_path: Path, excludes: set) -> List[Dict[str, Any]]:
    """Scan workspace for repositories."""
    repos = []

    # Get all top-level directories
    try:
        entries = sorted(workspace_path.iterdir())
    except OSError as e:
        print(f"Error reading workspace: {e}", file=sys.stderr)
        return repos

    dirs = [e for e in entries if e.is_dir() and e.name not in excludes and not e.name.startswith(".")]

    # First pass: collect names for dependency detection
    all_repo_names = [d.name for d in dirs]

    # Second pass: scan each directory
    for dir_path in dirs:
        # Check if it has any manifest file
        runtime = None
        for manifest, rt in MANIFEST_FILES.items():
            if (dir_path / manifest).exists():
                runtime = rt
                break

        if runtime is None:
            # Check if it has a README (might still be a docs/config repo)
            if not (dir_path / "README.md").exists():
                continue

        name = dir_path.name
        description = read_description(dir_path)
        repo_type = classify_repo_type(name, description)
        tech_stack = detect_tech_stack(dir_path)
        dependencies = detect_dependencies(dir_path, all_repo_names)

        repo = {
            "name": name,
            "description": description,
            "type": repo_type,
            "tech_stack": tech_stack,
        }

        if dependencies:
            repo["dependencies"] = dependencies

        if runtime:
            repo["runtime"] = runtime

        repos.append(repo)

    return repos


def main():
    parser = argparse.ArgumentParser(
        description="Scan a workspace for repositories and output repositories.jsonl"
    )
    parser.add_argument("workspace", help="Path to the workspace root directory")
    parser.add_argument(
        "--output", "-o",
        help="Output file path (default: stdout)",
        default=None,
    )
    parser.add_argument(
        "--exclude", "-e",
        help=f"Comma-separated list of directory names to exclude (default: {','.join(sorted(DEFAULT_EXCLUDES))})",
        default=",".join(sorted(DEFAULT_EXCLUDES)),
    )

    args = parser.parse_args()
    workspace = Path(args.workspace).resolve()

    if not workspace.is_dir():
        print(f"Error: {workspace} is not a directory", file=sys.stderr)
        sys.exit(1)

    excludes = set(args.exclude.split(",")) if args.exclude else DEFAULT_EXCLUDES

    repos = scan_workspace(workspace, excludes)

    # Build JSONL output
    lines = [json.dumps(repo, ensure_ascii=False) for repo in repos]
    output = "\n".join(lines) + "\n" if lines else ""

    if args.output:
        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w") as f:
            f.write(output)
        print(f"Wrote {len(repos)} repositories to {output_path}")
    else:
        sys.stdout.write(output)
        print(f"\n# {len(repos)} repositories found", file=sys.stderr)


if __name__ == "__main__":
    main()
