#!/usr/bin/env python3
"""Fast documentation gate for scripts/check.sh --fast.

Checks every documented `cp` code fence (`cpl check`), runs every `site/examples/*.cp`
fixture and write-code exercise reference through `cpl run`, verifies the one-page
cheat-sheet line budget, and checks all local links/fragments in `site/_build/`.
"""

from html.parser import HTMLParser
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from urllib.parse import unquote, urlsplit

import mistune

import build
import cookbook
import errors

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
CPL = REPO / "target/debug/cpl"

EXAMPLE_FIXTURES = {
    "bounds": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
    "sum": [("5\n3 1 4 1 5\n", "14"), ("1\n-9\n", "-9")],
    "knapsack": [("3 7\n3 4\n4 5\n2 3\n", "9"), ("2 3\n4 10\n5 20\n", "0")],
    "range_array": [("6 4\n0 2 4 2 1 2\n", "1 1 3 0 1")],
    "accumulation": [("4\n0 1 2 3\n", "6"), ("1\n0\n", "0")],
    "optional": [("5\n3 7 4 1 8\n", "8"), ("3\n1 3 5\n", "-1")],
    "ascii": [("5\nabaca\n", "3"), ("1\nz\n", "0")],
    "memo": [("4\n", "45"), ("1\n", "1")],
}


def documentation_paths():
    return [*sorted((ROOT / "content").glob("*.md")), REPO / "docs/one-page-cheat-sheet.md"]


def documented_programs():
    markdown = mistune.create_markdown(renderer="ast")

    def blocks(tokens):
        for token in tokens:
            if token["type"] == "block_code":
                info = token.get("attrs", {}).get("info", "").split()
                if info and info[0].lower() == "cp":
                    yield token["raw"]
            yield from blocks(token.get("children", []))

    for path in documentation_paths():
        for number, source in enumerate(blocks(markdown(path.read_text())), 1):
            yield path, number, source


class LinkPage(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.links, self.ids, self.duplicates = [], set(), []
        self.feed(source)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if "id" in attrs:
            if attrs["id"] in self.ids:
                self.duplicates.append(attrs["id"])
            self.ids.add(attrs["id"])
        if tag in ("a", "link", "script"):
            self.links.append(attrs.get("href", attrs.get("src", "")))


def check_cheat_sheet_budget():
    lines = len((REPO / "docs/one-page-cheat-sheet.md").read_text().splitlines())
    if lines > 250:
        sys.exit(f"check_docs: docs/one-page-cheat-sheet.md has {lines} lines (cap 250)")
    return lines


def check_doc_fences():
    programs = list(documented_programs())
    if not programs:
        sys.exit("check_docs: found 0 documented cp blocks")
    with tempfile.TemporaryDirectory(prefix="cpl-docs-fast-") as temporary:
        tmp = Path(temporary)
        library = REPO / "examples/local-lib"
        (tmp / "cp.toml").write_text(f'[libs]\nlib = "{library.as_posix()}"\n')
        files = []
        mapping = {}
        for path, number, source in programs:
            cp = tmp / f"{path.stem}-{number:03d}.cp"
            cp.write_text(source)
            files.append(str(cp))
            mapping[str(cp)] = f"{path.relative_to(REPO)} block #{number}"
        res = subprocess.run([str(CPL), "check", *files], cwd=REPO, capture_output=True, text=True, timeout=60)
        if res.returncode != 0:
            out = res.stdout + res.stderr
            for k, v in mapping.items():
                out = out.replace(k, v)
            sys.exit(f"check_docs: documented cp block failed `cpl check`:\n{out}")
    return len(programs)


def run_examples():
    fixtures = {k: list(v) for k, v in EXAMPLE_FIXTURES.items()}
    for lesson in build.LESSONS:
        for exercise in lesson["exercises"]:
            if exercise["kind"] == "output":
                fixtures.setdefault(exercise["fixture"], []).extend(
                    (case["input"], case["expected"]) for case in exercise["tests"]
                )
    on_disk = {p.stem for p in (ROOT / "examples").glob("*.cp")}
    if on_disk != set(fixtures):
        sys.exit(f"check_docs: site/examples mismatch: extra={on_disk - set(fixtures)} missing={set(fixtures) - on_disk}")
    cases_run = 0
    for name, cases in sorted(fixtures.items()):
        source_path = ROOT / "examples" / f"{name}.cp"
        for data, expected in cases:
            res = subprocess.run(
                [str(CPL), "run", str(source_path)],
                cwd=REPO,
                input=data,
                capture_output=True,
                text=True,
                timeout=15,
            )
            if res.returncode != 0:
                sys.exit(f"check_docs: example {name}.cp failed:\n{res.stderr}")
            if res.stdout.split() != expected.split():
                sys.exit(
                    f"check_docs: example {name}.cp output mismatch: "
                    f"got {res.stdout.split()!r}, expected {expected.split()!r}"
                )
            cases_run += 1
    return len(fixtures), cases_run


def run_course_references():
    refs = [ex for ex in build.EXERCISES.values() if ex["kind"] == "code"]
    cases_run = 0
    with tempfile.TemporaryDirectory(prefix="cpl-refs-fast-") as temporary:
        tmp = Path(temporary)
        for ex in refs:
            cp = tmp / f'{ex["id"]}.cp'
            cp.write_text(ex["reference"])
            for case in ex["tests"]:
                res = subprocess.run(
                    [str(CPL), "run", str(cp)],
                    cwd=REPO,
                    input=case["input"],
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                if res.returncode != 0:
                    sys.exit(f'check_docs: exercise reference {ex["id"]} failed:\n{res.stderr}')
                if res.stdout.split() != case["expected"].split():
                    sys.exit(
                        f'check_docs: exercise reference {ex["id"]} output mismatch: '
                        f'got {res.stdout.split()!r}, expected {case["expected"].split()!r}'
                    )
                cases_run += 1
    return len(refs), cases_run


def check_links():
    target = ROOT / "_build"
    if not target.is_dir():
        sys.exit("check_docs: site/_build missing; run site/build.py or site/bundle.py first")
    pages = {p.name: LinkPage(p.read_text()) for p in sorted(target.glob("*.html"))}
    for name, page in pages.items():
        if page.duplicates:
            sys.exit(f"check_docs: duplicate ids in {name}: {page.duplicates}")
        for link in page.links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            path = unquote(url.path) or name
            if not (target / path).is_file():
                sys.exit(f"check_docs: broken link in {name}: {link}")
            if url.fragment and unquote(url.fragment) not in pages[path].ids:
                sys.exit(f"check_docs: broken fragment in {name}: {link}")
    return len(pages)


def check_error_index():
    rust_codes = set(errors.discover_rust_codes(REPO))
    catalog_codes = [e["code"] for e in errors.ERROR_CATALOG]
    if len(catalog_codes) != len(set(catalog_codes)):
        sys.exit("check_docs: duplicate codes in site/errors.py")
    missing = sorted(rust_codes - set(catalog_codes))
    extra = sorted(set(catalog_codes) - rust_codes)
    if missing or extra:
        sys.exit(f"check_docs: error index drift: missing={missing}, extra={extra}")

    target = ROOT / "_build"
    for entry in errors.ERROR_CATALOG:
        page = target / f'{errors.error_slug(entry["code"])}.html'
        if not page.is_file():
            sys.exit(f"check_docs: missing generated error page {page.name}")

    verified = 0
    with tempfile.TemporaryDirectory(prefix="cpl-errors-fast-") as temporary:
        tmp = Path(temporary)
        # Batch-check all fixes first.
        fix_files = []
        for entry in errors.ERROR_CATALOG:
            if not entry["fix"]:
                continue
            fix_path = tmp / f'{entry["code"]}_fix.cp'
            fix_path.write_text(entry["fix"])
            fix_files.append((entry, fix_path))
        res = subprocess.run(
            [str(CPL), "check", *[str(p) for _, p in fix_files]],
            cwd=REPO,
            capture_output=True,
            text=True,
            timeout=60,
        )
        if res.returncode != 0:
            sys.exit(f"check_docs: error-index fix failed `cpl check`:\n{res.stderr}\n{res.stdout}")

        # Verify each trigger produces its exact diagnostic/lint/trap.
        for entry in errors.ERROR_CATALOG:
            code = entry["code"]
            mode = entry["mode"]
            if mode in ("internal", "reserved"):
                verified += 1
                continue
            trig = tmp / f"{code}_trigger.cp"
            trig.write_text(entry["trigger"])
            fix = tmp / f"{code}_fix.cp"
            if mode == "check":
                r = subprocess.run(
                    [str(CPL), "check", str(trig)],
                    cwd=REPO,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                combined = r.stdout + "\n" + r.stderr
                if f"[{code}]" not in combined:
                    sys.exit(
                        f"check_docs: error-index trigger for {code} did not emit [{code}]:\n{combined}"
                    )
                if code.startswith("W"):
                    rf = subprocess.run(
                        [str(CPL), "check", str(fix)],
                        cwd=REPO,
                        capture_output=True,
                        text=True,
                        timeout=15,
                    )
                    if f"[{code}]" in (rf.stdout + "\n" + rf.stderr):
                        sys.exit(f"check_docs: error-index fix for {code} still emits [{code}]")
            elif mode == "build":
                r = subprocess.run(
                    [str(CPL), "build", str(trig)],
                    cwd=REPO,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                combined = r.stdout + "\n" + r.stderr
                if f"[{code}]" not in combined:
                    sys.exit(
                        f"check_docs: error-index build trigger for {code} did not emit [{code}]:\n{combined}"
                    )
            elif mode == "lint":
                r = subprocess.run(
                    [str(CPL), "lint", str(trig)],
                    cwd=REPO,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                combined = r.stdout + "\n" + r.stderr
                if f"[{code}]" not in combined:
                    sys.exit(
                        f"check_docs: error-index lint trigger for {code} did not emit [{code}]:\n{combined}"
                    )
                rf = subprocess.run(
                    [str(CPL), "lint", str(fix)],
                    cwd=REPO,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                if f"[{code}]" in (rf.stdout + "\n" + rf.stderr):
                    sys.exit(
                        f"check_docs: error-index lint fix for {code} still emits [{code}]:\n{rf.stderr}"
                    )
            elif mode == "run":
                r = subprocess.run(
                    [str(CPL), "run", str(trig)],
                    cwd=REPO,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                if r.returncode == 0:
                    sys.exit(f"check_docs: error-index runtime trigger for {code} did not trap")
                rf = subprocess.run(
                    [str(CPL), "run", str(fix)],
                    cwd=REPO,
                    capture_output=True,
                    text=True,
                    timeout=15,
                )
                if rf.returncode != 0:
                    sys.exit(f"check_docs: error-index runtime fix for {code} failed:\n{rf.stderr}")
            else:
                sys.exit(f"check_docs: unknown error-index mode {mode!r} for {code}")
            verified += 1
    return verified


def check_cookbook() -> tuple[int, int]:
    """Verify every cookbook recipe port exists, passes `cpl check`, and matches samples."""
    recipes = cookbook.all_recipes()
    if not recipes:
        sys.exit("check_docs: cookbook has 0 recipes")
    seen: set[str] = set()
    port_paths: list[str] = []
    for _, port_name, _, _ in recipes:
        if port_name in seen:
            sys.exit(f"check_docs: duplicate cookbook recipe {port_name}")
        seen.add(port_name)
        p = cookbook.port_path(REPO, port_name)
        if not p.is_file():
            sys.exit(f"check_docs: cookbook port missing: {p.relative_to(REPO)}")
        port_paths.append(str(p))

    res = subprocess.run(
        [str(CPL), "check", *port_paths],
        cwd=REPO,
        capture_output=True,
        text=True,
        timeout=60,
    )
    if res.returncode != 0:
        sys.exit(f"check_docs: cookbook `cpl check` failed:\n{res.stderr}\n{res.stdout}")

    samples_dir = REPO / "proto/ports/ecnerwala/samples"
    sample_runs = 0
    for _, port_name, _, _ in recipes:
        p = cookbook.port_path(REPO, port_name)
        if "@!interactive" in p.read_text():
            continue
        for inp in sorted(samples_dir.glob(f"{port_name}.*.in")):
            out = inp.with_suffix(".out")
            if not out.is_file():
                continue
            r = subprocess.run(
                [str(CPL), "run", str(p)],
                input=inp.read_text(),
                cwd=REPO,
                capture_output=True,
                text=True,
                timeout=15,
            )
            if r.returncode != 0:
                sys.exit(f"check_docs: cookbook sample {inp.name} failed:\n{r.stderr}")
            got = "\n".join(line.rstrip() for line in r.stdout.strip().splitlines())
            exp = "\n".join(line.rstrip() for line in out.read_text().strip().splitlines())
            if got != exp:
                sys.exit(
                    f"check_docs: cookbook sample {inp.name} mismatch\n"
                    f"  expected: {exp!r}\n  got:      {got!r}"
                )
            sample_runs += 1
    return len(recipes), sample_runs


def check_stdlib_module_coverage() -> int:
    """Every non-prelude stdlib/*.cp module must be documented in site/content/stdlib.md."""
    stdlib_md = (ROOT / "content/stdlib.md").read_text()
    modules = sorted(p.stem for p in (REPO / "stdlib").glob("*.cp") if p.stem != "prelude")
    missing = [m for m in modules if f"std.{m}" not in stdlib_md]
    if missing:
        sys.exit(
            f"check_docs: stdlib modules missing from site/content/stdlib.md: {', '.join(missing)}"
        )
    return len(modules)


def check_ports_gallery() -> tuple[int, int]:
    """Verify ports gallery parses research/PORTS.md and renders CP + dream + C++ panels."""
    import ports_gallery

    _, rows = ports_gallery.parse_ports_md(REPO)
    if not rows:
        sys.exit("check_docs: ports gallery parsed 0 rows from research/PORTS.md")
    dreams = 0
    for row in rows:
        if row["cp_rel"] and not (REPO / row["cp_rel"]).is_file():
            sys.exit(f"check_docs: ports gallery missing .cp file {row['cp_rel']}")
        if row["dream_rel"]:
            if not (REPO / row["dream_rel"]).is_file():
                sys.exit(f"check_docs: ports gallery missing dream file {row['dream_rel']}")
            dreams += 1
    page = ROOT / "_build/ports.html"
    if not page.is_file():
        sys.exit("check_docs: missing generated site/_build/ports.html")
    html_text = page.read_text()
    if "ports-card" not in html_text or "Accepted C++" not in html_text:
        sys.exit("check_docs: ports.html missing expected side-by-side cards")
    if dreams > 0 and "ports-pane-dream" not in html_text:
        sys.exit("check_docs: ports.html missing dream comparison panes")
    return len(rows), dreams


def main():
    t0 = time.monotonic()
    if not CPL.is_file():
        sys.exit(f"check_docs: {CPL} not built; run cargo build -p cpl-cli")
    cheat_lines = check_cheat_sheet_budget()
    fence_count = check_doc_fences()
    ex_count, ex_cases = run_examples()
    ref_count, ref_cases = run_course_references()
    err_count = check_error_index()
    cb_count, cb_runs = check_cookbook()
    stdlib_mods = check_stdlib_module_coverage()
    ports_total, ports_dreams = check_ports_gallery()
    pg_res = subprocess.run(
        [sys.executable, str(ROOT / "test_playground_parity.py")],
        cwd=REPO,
        capture_output=True,
        text=True,
        timeout=60,
    )
    if pg_res.returncode != 0:
        sys.exit(f"check_docs: playground parity tests failed:\n{pg_res.stderr}\n{pg_res.stdout}")
    page_count = check_links()
    ms = int((time.monotonic() - t0) * 1000)
    print(
        f"docs gate: {fence_count} cp fences, {ex_count} examples ({ex_cases} runs), "
        f"{ref_count} course refs ({ref_cases} runs), {err_count} error codes, "
        f"{cb_count} cookbook recipes ({cb_runs} sample runs), "
        f"{ports_total} gallery ports ({ports_dreams} with dream), "
        f"{stdlib_mods} stdlib modules, {page_count} HTML pages, "
        f"cheat-sheet {cheat_lines}/250 lines — ok in {ms} ms"
    )


if __name__ == "__main__":
    main()
