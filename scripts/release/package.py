#!/usr/bin/env python3
"""Packs one release target: `cpl-<target>.tar.gz` (`.zip` on Windows) holding
bin/cpl, stdlib/, README.md and the licenses.

Archives are reproducible: entries are sorted, owned by 0:0 and stamped with
$SOURCE_DATE_EPOCH (else the HEAD commit time), so a rebuild of the same
commit and binary gives the same bytes.

    scripts/release/package.py <target> <cpl binary> <out dir>
"""
import gzip
import io
import os
import subprocess
import sys
import tarfile
import time
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def stamp():
    if os.environ.get("SOURCE_DATE_EPOCH"):
        return int(os.environ["SOURCE_DATE_EPOCH"])
    out = subprocess.run(["git", "log", "-1", "--format=%ct"], cwd=ROOT, capture_output=True, text=True)
    return int(out.stdout.strip() or 0)


def entries(binary, windows):
    exe = "cpl.exe" if windows else "cpl"
    yield f"bin/{exe}", Path(binary).read_bytes(), True
    for name in ["README.md", "LICENSE-MIT", "LICENSE-APACHE"]:
        yield name, (ROOT / name).read_bytes(), False
    for path in sorted((ROOT / "stdlib").rglob("*")):
        if path.is_file():
            yield path.relative_to(ROOT).as_posix(), path.read_bytes(), False


def pack(target, binary, out_dir):
    windows = "windows" in target
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    when = stamp()
    files = sorted(entries(binary, windows))
    if windows:
        out = out_dir / f"cpl-{target}.zip"
        with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
            date = max((1980, 1, 1, 0, 0, 0), tuple(time.gmtime(when)[:6]))
            for name, data, executable in files:
                info = zipfile.ZipInfo(name, date)
                info.external_attr = (0o755 if executable else 0o644) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                z.writestr(info, data)
        return out
    out = out_dir / f"cpl-{target}.tar.gz"
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.PAX_FORMAT) as tar:
        dirs = sorted({str(Path(name).parent) for name, _, _ in files} - {"."})
        for name in dirs:
            info = tarfile.TarInfo(name)
            info.type, info.mode, info.mtime = tarfile.DIRTYPE, 0o755, when
            tar.addfile(info)
        for name, data, executable in files:
            info = tarfile.TarInfo(name)
            info.size, info.mode, info.mtime = len(data), 0o755 if executable else 0o644, when
            tar.addfile(info, io.BytesIO(data))
    with open(out, "wb") as f, gzip.GzipFile(fileobj=f, mode="wb", mtime=0, filename="") as gz:
        gz.write(raw.getvalue())
    return out


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__.strip().splitlines()[-1].strip())
    print(pack(*sys.argv[1:]))
