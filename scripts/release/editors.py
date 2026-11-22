#!/usr/bin/env python3
"""Packs `editors.tar.gz`: the Neovim, Helix and Emacs setups, the Zed dev
extension and the tree-sitter grammar, as committed, under `cp-lang-editors/`.
Reproducible like package.py: sorted entries, 0:0, the HEAD commit time.

    scripts/release/editors.py <out dir>
"""
import gzip
import io
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PARTS = ["editors/README.md", "editors/neovim", "editors/helix", "editors/emacs", "editors/zed", "editors/tree-sitter-cp"]


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout


def main(out_dir):
    from package import stamp

    when = stamp()
    files = sorted(f for f in git("ls-files", "-z", "--", *PARTS).split("\0") if f)
    missing = [p for p in PARTS if not any(f == p or f.startswith(p + "/") for f in files)]
    if missing:
        sys.exit(f"editors.py: nothing committed under {', '.join(missing)}")
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.PAX_FORMAT) as tar:
        dirs = set()
        for name in files:
            parent = Path(name).parent
            while parent != Path("."):
                dirs.add(parent.as_posix())
                parent = parent.parent
        for name in sorted(dirs):
            info = tarfile.TarInfo("cp-lang-" + name)
            info.type, info.mode, info.mtime = tarfile.DIRTYPE, 0o755, when
            tar.addfile(info)
        for name in files:
            path = ROOT / name
            data = path.read_bytes()
            info = tarfile.TarInfo("cp-lang-" + name)
            info.size, info.mtime = len(data), when
            info.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
            tar.addfile(info, io.BytesIO(data))
    out = Path(out_dir) / "editors.tar.gz"
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "wb") as f, gzip.GzipFile(fileobj=f, mode="wb", mtime=0, filename="") as gz:
        gz.write(raw.getvalue())
    print(out)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__.strip().splitlines()[-1].strip())
    sys.path.insert(0, str(Path(__file__).parent))
    main(sys.argv[1])
