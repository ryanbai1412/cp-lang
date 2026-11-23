"""Regenerate every committed generated file, or check that none is stale.

    uv run python scripts/gen.py           rewrite them all
    uv run python scripts/gen.py --check   fail if any is stale (commit tier, about 1.5 s)

One entry per generator: what it writes and how to check it. A new committed
generated file gets a line here, so the commit tier catches it going stale.
"""
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

GENERATORS = [
    # script, write args, check args, files it owns
    ("skill-gen.py", [], ["--drift"], "skills/cp-lang/**"),
    ("conformance-coverage.py", [], ["--check"], "spec/conformance/COVERAGE.md, rule-gaps.txt"),
    ("conformance-matrix.py", [], ["--check"], "spec/conformance/generated cases from matrix.toml"),
    ("conformance-traits.py", [], ["--check"], "spec/conformance/trait-gaps.txt"),
    ("conformance-sigs.py", [], ["--check"], "spec/conformance/sig-gaps.txt"),
    ("conformance-prelude.py", [], ["--check"], "spec/conformance/prelude-gaps.txt"),
]


def main():
    check = "--check" in sys.argv
    failed = []
    for script, write, chk, owns in GENERATORS:
        t0 = time.monotonic()
        r = subprocess.run([sys.executable, str(ROOT / "scripts" / script), *(chk if check else write)],
                           cwd=ROOT, capture_output=True, text=True)
        took = time.monotonic() - t0
        if r.returncode:
            failed.append(script)
            sys.stdout.write(r.stdout + r.stderr)
        print(f"gen: {script:26} {took:4.1f}s {'FAIL' if r.returncode else 'ok'}  ({owns})")
    if failed:
        print(f"gen: {len(failed)} generated file set(s) stale or failing; run: just gen" if check else "gen: failed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
