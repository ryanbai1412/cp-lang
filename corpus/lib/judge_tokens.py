"""The token comparison `cpl test` and `cpl stress` judge by, for Python scripts.

Mirrors `same_token` in crates/cpl-stress/src/judge.rs (spec/tools/cli.md, `cpl test d`):
equal tokens match, yes/no match in any case, and two numbers, one written with a `.` or an
exponent, match within 1e-6 absolute or relative error. crates/cpl-cli/tests/judge_agreement.rs
checks the two give the same verdicts."""
import math

_DIGITS = "0123456789"


def _number(t):
    return any(c in _DIGITS for c in t) and all(c in _DIGITS or c in "+-.eE" for c in t)


def _real(t):
    return any(c in ".eE" for c in t)


def same_token(expected, got):
    if expected == got:
        return True
    if expected.lower() == got.lower() and expected.lower() in ("yes", "no"):
        return True
    if not (_number(expected) and _number(got) and (_real(expected) or _real(got))):
        return False
    try:
        a, b = float(expected), float(got)
    except ValueError:
        return False
    if not (math.isfinite(a) and math.isfinite(b)):
        return False
    diff = abs(a - b)
    return diff <= 1e-6 or diff <= 1e-6 * abs(a)


def same_tokens(expected, got):
    """Whether two outputs hold the same answer, token by token."""
    e, g = expected.split(), got.split()
    return len(e) == len(g) and all(map(same_token, e, g))
