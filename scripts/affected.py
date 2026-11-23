#!/usr/bin/env python3
"""Pick what the commit tier of the gate runs for a set of changed paths.

    affected.py [--json] [path ...]     default: the staged paths
    affected.py [--json] --base REV     the paths changed between REV and HEAD

A path under a crate selects that crate and, through nextest's `rdeps()`,
every crate that depends on it. Prose selects nothing. Any path this file
does not know selects the whole fast gate, so a new kind of input can only
make the commit tier slower, never let it miss a test. The land tier
(`scripts/check.sh --land`) selects the same way over a train's diff; CI's
`--fast` on every main tip always runs everything.
"""

import fnmatch
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Read by no crate, test or script check (scripts/check_cache.py's excludes).
PROSE = ["AGENTS.md", "REPO.md", "decisions.md", "worklog.md", "research/*",
         "plan/*", "corpus/submissions/*", "corpus/statements/*", "archive/*",
         ".github/*", "LICENSE*", "*.png"]
# Inputs of every crate: build settings and the std library cpl-check embeds.
EVERY_CRATE = ["Cargo.toml", "Cargo.lock", "rust-toolchain*", ".cargo/*",
               ".config/nextest.toml", "std/*", "clippy.toml", "rustfmt.toml"]
SITE = ["web/*", "docs/*"]
# Data files and the nextest filter of the tests that read them.
DATA_TESTS = [(f"corpus/problems/*/{leaf}", "package(cpl-lint) & test(/corpus_warnings/)")
              for leaf in ("spec.cp", "port.cp")]
# Editor files a crate embeds or its tests read; the rest are CI's editor jobs'.
EDITOR_CRATES = {"editors/neovim/": "cpl-cli", "editors/vscode/": "cpl-cli",
                 "editors/jetbrains/": "cpl-cli", "editors/tree-sitter-cp/": "cpl-cli",
                 "editors/lsp-matrix.json": "cpl-lsp"}


def matches(path, patterns):
    return any(fnmatch.fnmatch(path, p) for p in patterns)


def crate_dirs():
    """Map each workspace crate's directory to its package name."""
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    dirs = {}
    for pkg in json.loads(out)["packages"]:
        rel = os.path.relpath(os.path.dirname(pkg["manifest_path"]), ROOT)
        dirs[rel.replace(os.sep, "/") + "/"] = pkg["name"]
    return dirs


def script_test(path):
    """The unit test file that covers a script, or None."""
    name = os.path.basename(path)
    if name in ("land.sh", "check.sh", "check_cache.py", "gate-env.sh"):
        return "scripts/test_check_cache.py"
    if path.startswith("scripts/release/"):
        return "scripts/test_release.py"
    stem = os.path.splitext(name)[0].replace("-", "_")
    test = f"scripts/test_{stem.removeprefix('test_')}.py"
    return test if os.path.exists(os.path.join(ROOT, test)) else None


def select(paths, dirs):
    sel = {"all": False, "reason": "", "packages": [], "rust_files": [],
           "script_tests": [], "site": False, "conform_cases": [], "data_filter": ""}
    packages, tests, data = set(), set(), []
    for path in paths:
        crate = next((n for d, n in dirs.items() if path.startswith(d)), None)
        if path.endswith(".rs"):
            sel["rust_files"].append(path)
        editor = next((n for d, n in EDITOR_CRATES.items() if path.startswith(d)), None)
        if path.startswith("spec/conformance/") and path.endswith(".cp"):
            sel["conform_cases"].append(path)
        elif crate:
            packages.add(crate)
        elif editor:
            packages.add(editor)
        elif expr := next((e for p, e in DATA_TESTS if fnmatch.fnmatch(path, p)), None):
            if expr not in data:
                data.append(expr)
        elif matches(path, PROSE):
            continue
        elif matches(path, EVERY_CRATE):
            packages.update(dirs.values())
        elif matches(path, SITE):
            sel["site"] = True
        elif path.startswith("scripts/") and (t := script_test(path)):
            tests.add(t)
        elif path.startswith(".githooks/"):
            tests.add("scripts/test_check_cache.py")
        else:
            sel.update(all=True, reason=path)
            break
    sel["packages"] = sorted(packages)
    sel["script_tests"] = sorted(tests)
    sel["data_filter"] = " | ".join(f"({e})" for e in data)
    return sel


def staged():
    out = subprocess.run(["git", "diff", "--cached", "--name-only", "-z"],
                         cwd=ROOT, capture_output=True, check=True).stdout
    return [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]


def changed_since(base):
    out = subprocess.run(["git", "diff", "--name-only", "-z", base, "HEAD"],
                         cwd=ROOT, capture_output=True, check=True).stdout
    return [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]


def main(argv):
    as_json = "--json" in argv
    argv = [a for a in argv if a != "--json"]
    if argv[:1] == ["--base"]:
        paths = changed_since(argv[1])
    else:
        paths = argv or staged()
    sel = select(paths, crate_dirs())
    if as_json:
        print(json.dumps(sel))
        return 0
    # Shell form for scripts/check.sh: one `key=value` per line.
    print(f"all={int(sel['all'])}")
    print(f"reason={sel['reason']}")
    print("filter=" + " | ".join(f"rdeps({p})" for p in sel["packages"]))
    print("packages=" + " ".join(sel["packages"]))
    print("rust_files=" + " ".join(sel["rust_files"]))
    print("script_tests=" + " ".join(sel["script_tests"]))
    print(f"site={int(sel['site'])}")
    print("conform_cases=" + " ".join(sel["conform_cases"]))
    print("data_filter=" + sel["data_filter"])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
