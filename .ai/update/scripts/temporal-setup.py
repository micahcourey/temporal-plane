#!/usr/bin/env python3
"""Bootstrap the Temporal Plane: create venv, install lance-context, init store."""

import argparse
import subprocess
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Bootstrap the Temporal Plane")
    parser.add_argument(
        "--store-path",
        default=".ai/temporal",
        help="Path to the temporal store directory (default: .ai/temporal)",
    )
    args = parser.parse_args()

    ai_dir = Path(args.store_path)
    ai_dir.mkdir(parents=True, exist_ok=True)

    venv_dir = ai_dir / ".venv"
    print(f"Creating virtual environment at {venv_dir}...")
    subprocess.run([sys.executable, "-m", "venv", str(venv_dir)], check=True)

    pip = venv_dir / "bin" / "pip"
    print("Installing lance-context...")
    subprocess.run([str(pip), "install", "-q", "lance-context>=0.2.4,<0.3.0"], check=True)

    python = venv_dir / "bin" / "python3"
    print("Initializing store...")
    subprocess.run(
        [str(python), str(ai_dir / "temporal.py"), "stats"],
        check=True,
    )

    print()
    print("Temporal Plane ready.")
    print(f"  Store:  {ai_dir / 'store.lance'}")
    print(f"  Python: {python}")
    print()
    print("Usage:")
    print(f"  {python} {ai_dir / 'temporal.py'} recall --limit 5")
    print(f"  {python} {ai_dir / 'temporal.py'} add --tag discovery \"Learned something new\"")


if __name__ == "__main__":
    main()
