"""After a rebase, restates a ratchet count that two rules measured.

A rebase can meet a counter whose measuring code one side changed: the
commit's count is by its rule, main's by main's, and the merge's lower of the
two compares numbers of different rules (593f03665 kept 961, an
editor_logic_lines count by the old rule, where the tip measured 1033 by the
new one). For each counter whose rule at the rebased tip differs from the
commit's or main's, that side's count moves by what the tip's rule measures
over the side's rule on the tip's tree, and the lower of the sides stands.

    uv run --quiet python scripts/ratchet_remeasure.py BEFORE UPSTREAM   # at the rebased tip
"""

import importlib.util
import json
import os
import subprocess
import sys
import types
from pathlib import Path

SCRIPT, RATCHET = "scripts/debt-ratchet.py", "scripts/debt-ratchet.json"
SIDES = ("before", "upstream")


def show(rev, path):
    r = subprocess.run(["git", "show", f"{rev}:{path}"], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def load(text, name):
    # Beside the real script, so a rule that reads its sibling scripts finds them.
    path = Path(SCRIPT).parent / f".{name}.py"
    path.write_text(text, encoding="utf-8")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
    finally:
        path.unlink()
    return module


def rule(module, name):
    """The counter's code and every function and constant of `module` it reaches."""
    parts, seen = [], set()

    def walk(code):
        parts.append((code.co_code, code.co_names, tuple(c for c in code.co_consts if not isinstance(c, types.CodeType))))
        for const in code.co_consts:
            if isinstance(const, types.CodeType):
                walk(const)
        for ref in code.co_names:
            if ref in seen:
                continue
            seen.add(ref)
            value = getattr(vars(module).get(ref), "__wrapped__", vars(module).get(ref))
            if isinstance(value, types.FunctionType):
                walk(value.__code__)
            elif value is not None and not callable(value) and not isinstance(value, types.ModuleType):
                parts.append((ref, repr(value)))

    walk(module.COUNTERS[name][1].__code__)
    return parts


class Refused(RuntimeError):
    pass


def build_cpl():
    """The debug cpl at the rebased tip, as $CPL: the hook's build predates the rebase."""
    if subprocess.run(["cargo", "build", "-q", "-p", "cpl-cli"]).returncode:
        return False
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
    os.environ["CPL"] = str((target if target.is_absolute() else Path.cwd() / target) / "debug" / "cpl")
    return True


def measure(mods, name, sides, root, build=build_cpl):
    """Each side's count by its rule, or None when a counter can't measure here and
    keeps the merged count. A hard-zero counter never passes unmeasured (99bc5b2b0
    landed past corpus_fmt_dirty on a stale release cpl): it builds the debug cpl
    and measures again, or refuses."""
    def run():
        return {side: len(mods[side].COUNTERS[name][1](root)) for side in sides}
    try:
        return run()
    except Exception as err:
        first = err
    if type(first).__name__ == "MeasureError" and str(first).startswith("cpl:") and build():
        try:
            return run()
        except Exception as err:
            first = err
    if name in getattr(mods["tip"], "HARD_ZERO", ()):
        raise Refused(f"{name} is hard-zero and could not be re-measured by both rules, so it can't pass"
                      f" unmeasured: {first}; fix that, then land again")
    print(f"land: WARNING: could not re-measure {name} by both rules: {first}", file=sys.stderr)
    return None


def restate(baseline, old, new):
    """`baseline` by the new rule, from what the old rule (`old`) and the new (`new`) measure.
    A side already over its baseline by the old rule holds debt the rule change can't place:
    9887838f1's rule dropped 8 lines main had over its 147, and baseline + new - old made it 139."""
    return min(baseline, new) if old > baseline else baseline + new - old


def merged(counts, name, sides, moved, measured):
    """The rebased tip's baseline. When the commit landing brings the tip's rule, its own number
    was measured on its old base, so the tip's measure is the baseline: 0dbb0c6ba kept its 597
    though the rebased tip measured 603, and main sat 6 over. Otherwise main changed the rule:
    restate each side the rule moved under, and keep the lower."""
    if "before" not in moved:
        return measured["tip"]
    return max(0, min(restate(int(counts[s][name]), measured[s], measured["tip"]) if s in moved
                      else int(counts[s][name]) for s in sides))


def measure_all(mods, jobs, root):
    """`measure` for each job, in forked processes: serially a rule change reaching a shared
    helper re-measured dozens of counters one by one. A refusal comes back as its exception."""
    def one(job):
        name, _, moved = job
        try:
            return measure(mods, name, ["tip", *moved], root)
        except Refused as err:
            return err
    if len(jobs) < 2 or not hasattr(os, "fork"):
        return [one(j) for j in jobs]
    import multiprocessing
    global _ONE
    _ONE = one
    with multiprocessing.get_context("fork").Pool(min(len(jobs), os.cpu_count() or 1)) as pool:
        return pool.map(_run_one, jobs)


def _run_one(job):
    return _ONE(job)


def main(argv):
    sys.dont_write_bytecode = True
    revs = {"before": argv[1], "upstream": argv[2], "tip": "HEAD"}
    texts = {side: show(rev, SCRIPT) for side, rev in revs.items()}
    if None in texts.values() or len(set(texts.values())) == 1:
        return 0
    counts = {side: json.loads(show(revs[side], RATCHET) or "{}") for side in SIDES}
    current = json.loads(Path(RATCHET).read_text(encoding="utf-8"))
    root, restated = Path.cwd(), []
    mods = {side: load(text, f"ratchet_rule_{side}") for side, text in texts.items()}
    # Counters that run cpl over every case are CI's on each main tip (debt-ratchet's CI_ONLY):
    # restating one here cost a land 100 s.
    ci_only = () if os.environ.get("CI") else getattr(mods["tip"], "CI_ONLY", ())
    jobs = []
    for name, (_, _, gated) in mods["tip"].COUNTERS.items():
        sides = [s for s in SIDES if name in counts[s] and name in mods[s].COUNTERS]
        moved = [s for s in sides if rule(mods[s], name) != rule(mods["tip"], name)]
        if gated and name in current and moved and name not in ci_only:
            jobs.append((name, sides, moved))
    for (name, sides, moved), measured in zip(jobs, measure_all(mods, jobs, root)):
        if isinstance(measured, Refused):
            print(f"land: {measured}", file=sys.stderr)
            return 1
        if measured is None:
            continue
        value = merged(counts, name, sides, moved, measured)
        if value != current[name]:
            restated.append(f"{name} {current[name]} -> {value}")
            current[name] = value
    if restated:
        Path(RATCHET).write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        print(f"land: restated by the rebased tip's measuring rule: {', '.join(restated)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
