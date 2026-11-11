#!/usr/bin/env python3
"""Doctor script: Scans repository for surviving template placeholders."""

import argparse
import os
import sys
from collections import namedtuple
from pathlib import Path

LITERAL_PLACEHOLDERS = ("REPLACE_ME",)
IGNORE_DIRS = {".git", "target", "node_modules", ".venv", ".cargo"}
EXEMPT_PATHS = {
    "scripts/doctor.py",
    ".gemini/prompts/setup-new-tool.md",
    ".gemini/prompts/update-distribution-channels.md",
}
MAX_FILE_BYTES = 1_000_000

Finding = namedtuple("Finding", ["path", "line", "detail"])


def is_exempt(rel_path: str) -> bool:
    """Return True if the file is legitimately allowed to carry placeholder tokens."""
    if rel_path in EXEMPT_PATHS or rel_path.endswith(".example"):
        return True
    return False


def iter_scannable_files(root_dir: Path):
    """Yield all readable, non-symlinked files under root_dir."""
    for dir_path, dir_names, file_names in os.walk(root_dir, followlinks=False):
        current_dir = Path(dir_path)
        rel_dir = current_dir.relative_to(root_dir).as_posix()

        dir_names[:] = [
            d
            for d in sorted(dir_names)
            if d not in IGNORE_DIRS and not (current_dir / d).is_symlink()
        ]

        for file_name in sorted(file_names):
            file_path = current_dir / file_name
            if not file_path.is_symlink():
                yield file_path


def scan_placeholders(root_dir: Path) -> list:
    """Find occurrences of unresolved template placeholders."""
    findings = []
    for file_path in iter_scannable_files(root_dir):
        rel_path = file_path.relative_to(root_dir).as_posix()
        if is_exempt(rel_path):
            continue

        try:
            if file_path.stat().st_size > MAX_FILE_BYTES:
                continue
            text = file_path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue

        for line_no, line in enumerate(text.splitlines(), start=1):
            for token in LITERAL_PLACEHOLDERS:
                if token in line:
                    findings.append(
                        Finding(
                            rel_path,
                            line_no,
                            f"unresolved template placeholder '{token}'",
                        )
                    )
    return findings


def run_doctor(root_dir: Path = None) -> list:
    """Run all verification checks on root_dir."""
    if root_dir is None:
        root_dir = Path(__file__).resolve().parent.parent
    return scan_placeholders(root_dir)


def main():
    parser = argparse.ArgumentParser(
        description="Verify repository has no surviving template placeholders."
    )
    parser.add_argument(
        "--dir", type=str, default=None, help="Repository root to check"
    )
    args = parser.parse_args()

    root_dir = (
        Path(args.dir).resolve()
        if args.dir
        else Path(__file__).resolve().parent.parent
    )
    findings = run_doctor(root_dir)

    if findings:
        print(
            f"Doctor found {len(findings)} unresolved placeholder(s):",
            file=sys.stderr,
        )
        for f in findings:
            print(f"  {f.path}:{f.line}: {f.detail}", file=sys.stderr)
        sys.exit(1)

    print("Doctor: no unresolved template placeholders found.")
    sys.exit(0)


if __name__ == "__main__":
    main()
