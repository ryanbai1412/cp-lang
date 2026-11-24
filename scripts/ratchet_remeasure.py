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
    for name, (_, _, gated) in mods["tip"].COUNTERS.items():
        sides = [s for s in SIDES if name in counts[s] and name in mods[s].COUNTERS]
        moved = [s for s in sides if rule(mods[s], name) != rule(mods["tip"], name)]
        if not gated or name not in current or not moved:
            continue
        try:
            measured = {side: len(mods[side].COUNTERS[name][1](root)) for side in ["tip", *moved]}
        except Exception as err:  # A counter that can't measure here keeps the merged count; CI's check measures it.
            print(f"land: WARNING: could not re-measure {name} by both rules: {err}", file=sys.stderr)
            continue
        value = max(0, min(int(counts[s][name]) + (measured["tip"] - measured[s] if s in moved else 0) for s in sides))
        if value != current[name]:
            restated.append(f"{name} {current[name]} -> {value}")
            current[name] = value
    if restated:
        Path(RATCHET).write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        print(f"land: restated by the rebased tip's measuring rule: {', '.join(restated)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
