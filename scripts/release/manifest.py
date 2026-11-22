#!/usr/bin/env python3
"""Writes a release's `latest.json` and `SHA256SUMS` over every file in a
folder of release assets, and `<archive>.sha256` beside each archive.
`cpl self`, the launcher and the installers read `latest.json`; people check
downloads with `sha256sum -c SHA256SUMS`.

    scripts/release/manifest.py <dir> --toolchain ID --version V
        --channel stable|edge --commit SHA [--date YYYY-MM-DD] [--tag TAG]...
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

ARCHIVE = re.compile(r"^cpl-(?P<target>[a-z0-9_]+-[a-z0-9_-]+?)\.(tar\.gz|zip)$")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--toolchain", required=True)
    ap.add_argument("--version", required=True)
    ap.add_argument("--channel", required=True, choices=["stable", "edge"])
    ap.add_argument("--commit", required=True)
    ap.add_argument("--date", default="")
    ap.add_argument("--tag", action="append", default=[])
    args = ap.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", args.toolchain):
        ap.error(f"bad toolchain id {args.toolchain!r}")
    folder = Path(args.dir)
    sums, assets, others = [], {}, {}
    for path in sorted(folder.iterdir()):
        if not path.is_file() or path.name in ("latest.json", "SHA256SUMS") or path.suffix == ".sha256":
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        sums.append(f"{digest}  {path.name}\n")
        entry = {"file": path.name, "sha256": digest, "size": path.stat().st_size}
        match = ARCHIVE.match(path.name)
        if match:
            assets[match["target"]] = entry
            (folder / f"{path.name}.sha256").write_text(f"{digest}  {path.name}\n")
        else:
            others[path.name] = entry
    if not assets:
        ap.error(f"no cpl-<target> archives in {folder}")
    manifest = {
        "toolchain": args.toolchain,
        "version": args.version,
        "channel": args.channel,
        "commit": args.commit,
        "date": args.date,
        "tags": args.tag,
        "assets": assets,
        "files": others,
    }
    (folder / "latest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    digest = hashlib.sha256((folder / "latest.json").read_bytes()).hexdigest()
    sums.append(f"{digest}  latest.json\n")
    (folder / "SHA256SUMS").write_text("".join(sorted(sums, key=lambda s: s.split()[1])))


if __name__ == "__main__":
    main()
