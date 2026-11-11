#!/usr/bin/env python3
"""release.py - Release note extraction and release tagging utility.

Aligned with ai-agent-template release management standards.
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

VERSION_HEADING_REGEX_TEMPLATE = r"^## \[(?:v)?{version}\](?:[ \t]+-[ \t]+[^\n]+)?$"
CHANGELOG_NAME = "CHANGELOG.md"


def extract_changelog_section(content: str, version: str) -> str:
    """Return the markdown body of one version's changelog section."""
    numeric_version = re.escape(version.lstrip("v"))
    pattern = VERSION_HEADING_REGEX_TEMPLATE.format(version=numeric_version)
    heading = re.search(pattern, content, re.MULTILINE)

    if not heading:
        raise ValueError(
            f"CHANGELOG.md has no section for {version}. "
            f"Please update {CHANGELOG_NAME} before tagging."
        )

    remainder = content[heading.end() :]
    next_heading = re.search(r"^##[ \t]", remainder, re.MULTILINE)
    section = remainder[: next_heading.start()] if next_heading else remainder

    # Cut off trailing documentation footers or markdown comments
    section = re.split(
        r"^(?:<!--|\[[^\]]+\]:|---[ \t]*$)", section, maxsplit=1, flags=re.MULTILINE
    )[0]
    return section.strip()


def run_extract_notes(root_dir: Path, version: str) -> int:
    """Extract and print changelog notes for version to stdout."""
    changelog_path = root_dir / CHANGELOG_NAME
    if not changelog_path.is_file():
        print(f"Error: {CHANGELOG_NAME} not found in {root_dir}", file=sys.stderr)
        return 1

    try:
        content = changelog_path.read_text(encoding="utf-8")
        notes = extract_changelog_section(content, version)
        if not notes:
            print(f"Error: Section for {version} is empty.", file=sys.stderr)
            return 1
        print(notes)
        return 0
    except ValueError as err:
        print(f"Error: {err}", file=sys.stderr)
        return 1


def create_annotated_tag(root_dir: Path, tag: str) -> int:
    """Create an annotated git tag for tag name."""
    if not tag.startswith("v"):
        tag = f"v{tag}"

    # Verify git status is clean
    res = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=root_dir,
        capture_output=True,
        text=True,
    )
    if res.stdout.strip():
        print(
            "Error: Working tree is dirty. Commit all changes before tagging.",
            file=sys.stderr,
        )
        return 1

    tag_res = subprocess.run(
        ["git", "tag", "-a", tag, "-m", f"Release {tag}"],
        cwd=root_dir,
        capture_output=True,
        text=True,
    )
    if tag_res.returncode != 0:
        print(f"Error creating tag {tag}: {tag_res.stderr}", file=sys.stderr)
        return tag_res.returncode

    print(f"Created annotated tag {tag}.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Release tooling for lfr-rust-template."
    )
    parser.add_argument(
        "--extract-notes",
        metavar="TAG",
        help="Print version's changelog section to stdout and exit",
    )
    parser.add_argument(
        "--tag",
        metavar="TAG",
        help="Create an annotated tag pointing to current HEAD",
    )
    parser.add_argument(
        "--root",
        default=None,
        help="Repository root (defaults to parent directory of this script)",
    )

    args = parser.parse_args()
    root_dir = (
        Path(args.root).resolve()
        if args.root
        else Path(__file__).resolve().parent.parent
    )

    if args.extract_notes:
        return run_extract_notes(root_dir, args.extract_notes)

    if args.tag:
        return create_annotated_tag(root_dir, args.tag)

    parser.print_help()
    return 0


if __name__ == "__main__":
    sys.exit(main())
