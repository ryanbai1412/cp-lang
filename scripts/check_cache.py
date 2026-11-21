#!/usr/bin/env python3
"""Input-hash cache helper for scripts/check.sh --fast.

Computes a deterministic SHA-256 digest over a list of files and directories
(sorted relative paths + file contents, ignoring build/cache artifacts). Used to
skip unchanged gate steps without dropping coverage.
"""

import hashlib
import os
import sys

IGNORE_DIRS = {".git", "__pycache__", "target", "dist", "_build", "node_modules", ".pytest_cache"}


def hash_paths(root, rel_paths):
    """Return a deterministic hex SHA-256 digest of the given files/directories under `root`."""
    h = hashlib.sha256()
    seen = set()
    files = []
    for rp in sorted(set(rel_paths)):
        full = os.path.join(root, rp)
        if not os.path.exists(full):
            h.update(f"MISSING:{rp}\n".encode("utf-8"))
            continue
        if os.path.isfile(full):
            norm = os.path.relpath(full, root).replace(os.sep, "/")
            if norm not in seen:
                seen.add(norm)
                files.append((norm, full))
        elif os.path.isdir(full):
            for dirpath, dirnames, filenames in os.walk(full):
                dirnames[:] = sorted(d for d in dirnames if d not in IGNORE_DIRS)
                for fn in sorted(filenames):
                    if fn.endswith((".pyc", ".pyo")):
                        continue
                    fp = os.path.join(dirpath, fn)
                    norm = os.path.relpath(fp, root).replace(os.sep, "/")
                    if norm not in seen:
                        seen.add(norm)
                        files.append((norm, fp))
    for norm, fp in sorted(files):
        h.update(f"FILE:{norm}\n".encode("utf-8"))
        try:
            with open(fp, "rb") as f:
                while True:
                    chunk = f.read(65536)
                    if not chunk:
                        break
                    h.update(chunk)
        except OSError:
            h.update(b"<UNREADABLE>")
        h.update(b"\n")
    return h.hexdigest()


def cache_file(cache_dir, step_name):
    return os.path.join(cache_dir, f"{step_name}.sha256")


def is_hit(cache_dir, step_name, digest):
    cf = cache_file(cache_dir, step_name)
    if not os.path.isfile(cf):
        return False
    try:
        with open(cf, encoding="utf-8") as f:
            return f.read().strip() == digest
    except OSError:
        return False


def save_hit(cache_dir, step_name, digest):
    os.makedirs(cache_dir, exist_ok=True)
    cf = cache_file(cache_dir, step_name)
    tmp = cf + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(digest + "\n")
    os.replace(tmp, cf)


def main(argv=None):
    args = list(sys.argv[1:] if argv is None else argv)
    if len(args) < 4 or args[0] not in ("hash", "check", "save"):
        print("usage: check_cache.py <hash|check|save> <cache_dir> <step> <path>...", file=sys.stderr)
        return 2
    cmd, cache_dir, step_name = args[0], args[1], args[2]
    paths = args[3:]
    root = os.getcwd()
    digest = hash_paths(root, paths)
    if cmd == "hash":
        print(digest)
        return 0
    if cmd == "check":
        return 0 if is_hit(cache_dir, step_name, digest) else 1
    save_hit(cache_dir, step_name, digest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
