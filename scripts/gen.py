"""Regenerate every committed generated file in gen.toml, or check none is stale.

    uv run python scripts/gen.py               rewrite the pre-commit and ci ones
    uv run python scripts/gen.py --check       fail if a pre-commit one is stale (commit tier)
    uv run python scripts/gen.py --check --ci  also the ci ones (slow, or need a tool)
    uv run python scripts/gen.py --only NAME   one entry
    uv run python scripts/gen.py --list        print each entry's files

A `check = "diff"` entry is checked by rerunning its writer and comparing the
files it owns, which are restored afterwards.
"""
import glob
import shutil
import subprocess
import sys
import time
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def entries():
    return tomllib.loads((ROOT / "gen.toml").read_text())["gen"]


def owned(entry):
    return sorted({Path(p) for pat in entry["files"] for p in glob.glob(str(ROOT / pat), recursive=True)
                   if Path(p).is_file()})


def command(cmd):
    exe = cmd[0]
    if exe.endswith(".py"):
        return [sys.executable, str(ROOT / exe), *cmd[1:]]
    if exe == "node" and len(cmd) > 1:
        return ["node", str(ROOT / cmd[1]), *cmd[2:]]
    return list(cmd)


def run(entry, cmd):
    if shutil.which(command(cmd)[0]) is None:
        return None, f"skipped: `{cmd[0]}` not installed"
    r = subprocess.run(command(cmd), cwd=ROOT / entry.get("cwd", "."), capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


def diff_check(entry):
    before = {p: p.read_bytes() for p in owned(entry)}
    code, out = run(entry, entry["write"])
    if code is None:
        return code, out
    after = owned(entry)
    stale = [p for p in after if before.get(p) != p.read_bytes()] + [p for p in before if p not in after]
    for p in after:
        if p not in before:
            p.unlink()
    for p, data in before.items():
        p.write_bytes(data)
    if code:
        return code, out
    return (1, "stale: " + ", ".join(str(p.relative_to(ROOT)) for p in stale)) if stale else (0, "")


def main(argv):
    check, ci = "--check" in argv, "--ci" in argv
    only = argv[argv.index("--only") + 1] if "--only" in argv else None
    if "--list" in argv:
        for e in entries():
            print(f"{e['name']:13} {e['tier']:10} {' '.join(e['files'])}")
        return 0
    tiers = {"pre-commit"} | ({"ci"} if ci or not check else set())
    failed = []
    for e in entries():
        if (only and e["name"] != only) or (not only and e["tier"] not in tiers):
            continue
        t0 = time.monotonic()
        if not check:
            code, out = run(e, e["write"])
        elif e["check"] == "diff":
            code, out = diff_check(e)
        else:
            code, out = run(e, e["check"])
        status = "skip" if code is None else "FAIL" if code else "ok"
        if code:
            failed.append(e["name"])
            sys.stdout.write(out if out.endswith("\n") else out + "\n")
        elif code is None:
            print(f"gen: {e['name']}: {out}")
        print(f"gen: {e['name']:13} {time.monotonic() - t0:4.1f}s {status}")
    if failed:
        print(f"gen: {', '.join(failed)} stale or failing; run: just gen" if check else "gen: failed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
