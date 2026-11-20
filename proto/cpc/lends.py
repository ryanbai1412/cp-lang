"""Unified Index, `lends` projections and lexical exclusivity (plan/19-index-and-lends.md).

The prototype checks the surface rules; it does not have traits or generic structs, so `[I]T`,
slices and rows stay built into compiler.py and only *behave* as the library code plan/19 describes:

- a projection (`fn at(self, i: I) lends mut T`) is checked as a fn with one inline callback, and
  emitted as a C++ function returning the lent place by reference;
- a lent view (`a[l..=r]`, `g.row(r)`) is second-class: it may be passed, bound by `let`, iterated,
  and nothing else;
- two lends of one receiver that are live at once must be provably disjoint, by the projection's
  `law disjoint` and the flow facts in scope.
"""
import copy

from .ast import N
from .types import *

MUTATORS = {"push", "pop", "sort", "rev", "clear", "insert", "remove", "swap", "set", "add"}
SCALARS = (TInt, TBool, TChar, TDom, TMod)
FLIP = {"<": ">", ">": "<", "<=": ">=", ">=": "<=", "==": "==", "!=": "!="}
NEGATE = {"<": ">=", ">": "<=", "<=": ">", ">=": "<", "==": "!=", "!=": "=="}


def walk(n):
    """Every AST node under `n`, including `n`."""
    if isinstance(n, N):
        yield n
        for v in n.__dict__.values():
            yield from walk(v)
    elif isinstance(n, (list, tuple)):
        for v in n:
            yield from walk(v)


def show(e):
    """Source text of an expression, exact enough to compare two expressions for equality."""
    k = e.kind
    if k == "num": return str(e.value)
    if k == "char": return repr(e.value)
    if k == "bool": return str(e.value).lower()
    if k == "name": return e.name
    if k == "paren": return show(e.a)
    if k == "binop": return f"({show(e.a)} {e.op} {show(e.b)})"
    if k == "unop": return f"({e.op} {show(e.a)})"
    if k == "index": return f"{show(e.obj)}[{', '.join(map(show, e.idx))}]"
    if k == "field": return f"{show(e.obj)}.{e.name}"
    if k == "unwrap": return f"{show(e.a)}!"
    if k == "call": return f"{show(e.fn)}({', '.join(map(show, e.args))})"
    if k == "kwarg": return f"{e.name}: {show(e.value)}"
    if k == "range": return f"{show(e.lo)}{'..=' if e.incl else '..'}{show(e.hi) if e.hi is not None else ''}"
    if k == "cmp":
        return "(" + show(e.first) + "".join(f" {op} {show(r)}" for op, r in zip(e.ops, e.rest)) + ")"
    if k == "tuple": return "(" + ", ".join(map(show, e.items)) + ")"
    return f"<{k}@{id(e)}>"


def names_in(e):
    return {n.name for n in walk(e) if n.kind == "name"}


def is_pure(e):
    """No calls: evaluating it twice gives the same value while no var in it is assigned."""
    return all(n.kind in ("num", "char", "bool", "name", "paren", "binop", "unop", "field", "index", "range", "cmp", "unwrap")
               for n in walk(e))


def subst(e, env):
    """`e` with each name in `env` replaced by env[name]; `s.lo` / `s.hi` of a range literal are reduced."""
    if isinstance(e, list):
        return [subst(x, env) for x in e]
    if not isinstance(e, N):
        return e
    if e.kind == "name" and e.name in env:
        return env[e.name]
    out = copy.copy(e)
    for k, v in e.__dict__.items():
        if isinstance(v, (N, list)):
            setattr(out, k, subst(v, env))
    if out.kind == "field" and out.name in ("lo", "hi"):
        r = out.obj
        while r.kind == "paren":
            r = r.a
        if r.kind == "range" and r.hi is not None:
            if out.name == "lo":
                return r.lo
            # `l..=r` ends at the boundary after r; `l..m` ends at m
            return N("field", r.line, obj=r.hi, name="r") if r.incl else r.hi
    return out


def root_name(e):
    while e.kind in ("index", "field", "paren", "unwrap") or (e.kind == "call" and e.fn.kind == "field"):
        e = e.fn.obj if e.kind == "call" else (e.a if e.kind in ("paren", "unwrap") else e.obj)
    return e.name if e.kind == "name" else None


def written_names(stmts):
    """Names a block may assign to (syntactic): assignment targets, `mut` arguments, mutating methods."""
    out = set()
    for n in walk(stmts):
        if n.kind == "assign":
            out.add(root_name(n.target))
        elif n.kind == "call":
            for a in n.args:
                if getattr(a, "mut", False):
                    out.add(root_name(a))
            if n.fn.kind == "field" and n.fn.name in MUTATORS:
                out.add(root_name(n.fn.obj))
        elif n.kind in ("for", "let") and getattr(n, "pat", None) is not None:
            out |= {p.name for p in walk(n.pat) if p.kind == "pname"}
    out.discard(None)
    return out


class Hold:
    """`let part = a[s]`: `a` stays lent until the last statement that mentions `part`."""
    def __init__(self, name, sym, root, steps, mut, line, until):
        self.name, self.sym, self.root, self.steps, self.mut, self.line, self.until = name, sym, root, steps, mut, line, until


class Proj:
    """A user projection: `fn name(self, ...) lends [mut] T`."""
    def __init__(self, name, cpp, pnames, ptypes, ret, mut, law, mut_self, owner):
        self.name, self.cpp, self.pnames, self.ptypes, self.ret = name, cpp, pnames, ptypes, ret
        self.mut, self.law, self.mut_self, self.owner = mut, law, mut_self, owner


def _law(src):
    from .parser import Parser
    return Parser(src).simple_stmt()

BUILTIN_LAWS = {
    "[]": ("Array.at", "i != j", None),                               # one element; component-wise for a[i, j]
    "slice": ("Array.at(Range)", "s.hi <= t.lo or t.hi <= s.lo", _law("law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo")),
    "row": ("Array.row", "r != q", _law("law disjoint(r, q) = r != q")),
}


class LendMixin:
    def lend_init(self):
        self.facts = []          # (lhs text, op, rhs text, names) known to hold here
        self.holds = []          # live let-bound lends
        self.covered = set()     # ids of nodes that are the receiver of an enclosing place expression
        self.idx_kind = {}       # id(index node) -> "at" | "slice"
        self.projs = {}          # struct cpp name -> {method name: Proj}
        self.proj_names = {"row"}
        self.index_impls = {}    # struct cpp name -> DomInfo of its `impl Index`
        self.cur_rest = []
        self.view_specs = {}
        self.escapes = {}        # id(node) -> why a lent value may not be the result of this node
        self.last_type = {}      # id(node) -> type of the last evaluation (to see a lent view without re-evaluating)

    # ------------------------------------------------------------------ facts
    def add_facts(self, cond, negate=False):
        terms = []
        def split(x):
            while x.kind == "paren":
                x = x.a
            if x.kind == "binop" and x.op == "and" and not negate:
                split(x.a); split(x.b)
            else:
                terms.append(x)
        split(cond)
        for t in terms:
            if t.kind != "cmp" or (negate and len(t.ops) != 1):
                continue
            prev = t.first
            for op, nxt in zip(t.ops, t.rest):
                if op in FLIP and is_pure(prev) and is_pure(nxt):
                    op2 = NEGATE[op] if negate else op
                    self.facts.append((show(prev), op2, show(nxt), names_in(prev) | names_in(nxt)))
                prev = nxt

    def drop_facts(self, names):
        if self.facts and names:
            self.facts = [f for f in self.facts if not (f[3] & names)]

    def has_fact(self, a, ops, b):
        for fa, op, fb, _ in self.facts:
            if fa == a and fb == b and op in ops:
                return True
            if fa == b and fb == a and FLIP[op] in ops:
                return True
        return False

    def lit(self, e):
        while e.kind == "paren":
            e = e.a
        if e.kind in ("num", "char"):
            return e.value
        if e.kind == "unop" and e.op == "-" and e.a.kind == "num":
            return -e.a.value
        return None

    def interval(self, e):
        try:
            saved = (list(self.facts), list(self.holds), set(self.covered), len(self.checks) if hasattr(self, "checks") else 0)
            _, (c, t) = self.capture(lambda: self.expr(e))
        except Exception:
            return None
        finally:
            self.facts, self.holds, self.covered = saved[0], saved[1], saved[2]
            if hasattr(self, "checks"):
                del self.checks[saved[3]:]
        if isinstance(t, TInt) and t.lo is not None and t.hi is not None:
            return t.lo, t.hi
        return None

    def prove(self, e):
        """Can the flow facts decide this law instance? Never a guess: False means "write the if or the assert"."""
        while e.kind == "paren":
            e = e.a
        if e.kind == "binop" and e.op == "or":
            return self.prove(e.a) or self.prove(e.b)
        if e.kind == "binop" and e.op == "and":
            return self.prove(e.a) and self.prove(e.b)
        if e.kind != "cmp" or len(e.ops) != 1 or e.ops[0] not in FLIP:
            return False
        op, a, b = e.ops[0], e.first, e.rest[0]
        la, lb = self.lit(a), self.lit(b)
        if la is not None and lb is not None:
            return {"!=": la != lb, "==": la == lb, "<": la < lb, "<=": la <= lb, ">": la > lb, ">=": la >= lb}[op]
        ta, tb = show(a), show(b)
        same = ta == tb and is_pure(a)
        if op in ("<=", ">=", "==") and same:
            return True
        implied = {"!=": {"!=", "<", ">"}, "<": {"<"}, ">": {">"}, "<=": {"<=", "<", "=="}, ">=": {">=", ">", "=="}, "==": {"=="}}[op]
        if self.has_fact(ta, implied, tb):
            return True
        if op in ("!=", "<", ">", "<=", ">=") and not same:
            ia, ib = self.interval(a), self.interval(b)
            if ia and ib:
                if op == "!=": return ia[1] < ib[0] or ib[1] < ia[0]
                if op == "<": return ia[1] < ib[0]
                if op == "<=": return ia[1] <= ib[0]
                if op == ">": return ia[0] > ib[1]
                if op == ">=": return ia[0] >= ib[1]
        return False

    # ------------------------------------------------------------------ places
    def is_proj_call(self, e):
        return e.kind == "call" and e.fn.kind == "field" and e.fn.name in self.proj_names

    def place_path(self, e):
        """(root symbol, steps) for a place expression, or None for a plain value."""
        steps = []
        while True:
            if e.kind in ("paren", "unwrap"):
                e = e.a
            elif e.kind == "field":
                steps.append(("field", e.name))
                e = e.obj
            elif e.kind == "index":
                steps.append(("proj", self.idx_kind.get(id(e), "[]"), e.idx))
                e = e.obj
            elif self.is_proj_call(e):
                steps.append(("proj", e.fn.name, [a for a in e.args]))
                e = e.fn.obj
            else:
                break
        if e.kind != "name":
            return None
        sym = self.lookup(e.name)
        if sym is None or sym.kind not in ("let", "var"):
            return None
        steps.reverse()
        via = None
        lent = getattr(sym, "lent", None)
        if lent is not None:                    # a let-bound lend stands for the place it was lent from
            via, (sym, base) = sym, lent
            steps = base + steps
        return sym, steps, via

    def law_text(self, step):
        name = step[1]
        if name in BUILTIN_LAWS:
            return BUILTIN_LAWS[name][0], BUILTIN_LAWS[name][1]
        for tbl in self.projs.values():
            if name in tbl:
                p = tbl[name]
                return f"{p.owner}.{name}", (show(p.law.expr) if p.law is not None else None)
        return name, None

    def step_disjoint(self, a, b):
        name = a[1]
        if name == "[]":
            return len(a[2]) == len(b[2]) and any(
                self.prove(N("cmp", x.line, first=x, ops=["!="], rest=[y])) for x, y in zip(a[2], b[2]))
        law = BUILTIN_LAWS[name][2] if name in BUILTIN_LAWS else None
        if law is None:
            for tbl in self.projs.values():
                if name in tbl:
                    law = tbl[name].law
        if law is None or len(law.params) != len(a[2]) + len(b[2]):
            return False
        env = dict(zip(law.params, list(a[2]) + list(b[2])))
        return self.prove(subst(law.expr, env))

    def overlap(self, p, q):
        """None when the two places can't share storage, else the reason they might."""
        for k, (a, b) in enumerate(zip(p, q)):
            if a[0] == "field" and b[0] == "field":
                if a[1] != b[1]:
                    return None                 # different fields are always disjoint
                continue
            if a[0] == "proj" and b[0] == "proj" and a[1] == b[1]:
                if self.step_disjoint(a, b):
                    return None
                who, law = self.law_text(a)
                if [show(x) for x in a[2]] == [show(x) for x in b[2]] and all(is_pure(x) for x in a[2]):
                    if k + 1 < len(p) or k + 1 < len(q):
                        continue                # the same element: look one step deeper
                    return (f"both are the same part of it" +
                            (f"; {who} lends disjoint parts only when `{law}`" if law else ""))
                if law is None:
                    return f"{who} declares no `law disjoint`, so two of its lends always conflict"
                return f"{who} lends disjoint parts only when `{law}`, and that is not known here"
            if a[0] == "proj" and b[0] == "proj":
                return (f"`{self.law_text(a)[0]}` and `{self.law_text(b)[0]}` are different projections of the same "
                        f"value, and those always conflict")
            return "one is a field and the other a projection of the same value"
        return "one contains the other"

    def check_args(self, e, items):
        """Arguments of one call are lent together: `f(mut a[i], mut a[j])` needs `i != j`."""
        paths = [(a, m, self.place_path(a)) for a, m in items]
        for x in range(len(paths)):
            for y in range(x + 1, len(paths)):
                (a, am, ap), (b, bm, bp) = paths[x], paths[y]
                if not (am or bm) or ap is None or bp is None or ap[0] is not bp[0]:
                    continue
                why = self.overlap(ap[1], bp[1])
                if why is not None:
                    self.error(b, f"`{show(a)}` and `{show(b)}` may be the same storage, and "
                                  f"{'both are' if am and bm else 'one is'} lent `mut` to this call: {why}\n"
                                  f"  help: guard it (`if i != j: ...`), state it (`assert i != j`), "
                                  f"or copy the read-only one first")

    # ------------------------------------------------------------------ let-bound lends
    def begin_hold(self, s, name, sym, value):
        path = self.place_path(value)
        if path is None:
            return
        root, steps, via = path
        sym.lent = (root, steps)
        sym.lent_mut = root.mut
        last = None
        for st in self.cur_rest:
            if any(n.kind == "name" and n.name == name for n in walk(st)):
                last = st
        if last is None:
            return
        mut = any(name in written_names([st]) for st in self.cur_rest)
        if mut and not root.mut:
            self.error(s, f"`{name}` is changed below, but `{root_name(value)}` is not a var, so it can only be lent for reading")
        self.holds.append(Hold(name, sym, root, steps, mut, s.line, last))

    def check_access(self, e, write):
        if not self.holds:
            return
        path = self.place_path(e)
        if path is None:
            return
        root, steps, via = path
        for h in self.holds:
            if h.root is not root or via is h.sym:
                continue
            if not (write or h.mut):
                continue
            why = self.overlap(steps, h.steps)
            if why is None:
                continue
            what = "changed" if write else "used"
            kind = "mutably " if h.mut else ""
            self.error(e, f"`{show(e)}` can't be {what} here: `{root_name(e)}` is {kind}lent to `{h.name}` (line {h.line}) "
                          f"until its last use on line {h.until.line} ({why})\n"
                          f"  help: finish with `{h.name}` first, go through `{h.name}`, or keep the position and "
                          f"re-project later; `.copy()` keeps an owned copy")

    def no_escape(self, t, node, what):
        """Lent values are second-class (plan/19 §3.5)."""
        if getattr(t, "view", None):
            src = show(node) if isinstance(node, N) else "it"
            self.error(node, f"{what} would keep the lent {t.view} `{src}` after this block; a lent value lives only in the "
                             f"block that made it\n  help: keep the range or the row index (a plain value) and re-project "
                             f"where it is used, or keep a copy: `{src}.copy()`")
