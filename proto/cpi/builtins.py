"""The prelude and the L0/L1 core the interpreter provides natively.

List, Str, Opt, ranges, domains and their boundaries are library structs in
the design (plan/19); here they are native values with the same surface, so
stdlib/ can start at the algebra layer.
"""
import math
import functools
import itertools

from .values import *


def copyv(v):
    t = type(v)
    if t is CList:
        return CList([copyv(x) for x in v.items], v.is_str)
    if t is CArray:
        return CArray(v.dims, [copyv(x) for x in v.items])
    if t is Tup:
        return Tup([copyv(x) for x in v.items], v.names)
    if t is Obj:
        return Obj(v.stype, {k: copyv(x) for k, x in v.f.items()})
    if t is CSet:
        return CSet(set(v.items))
    if t is CMap:
        return CMap({k: copyv(x) for k, x in v.items.items()})
    return v


def keyfn(it, by, node):
    """`by:` is a key fn or an Order value (a struct with an `lt` field)."""
    if by is None:
        return None
    if type(by) is Obj and "lt" in by.f:
        lt = by.f["lt"]
        return functools.cmp_to_key(lambda a, b: -1 if it.call(lt, [a, b], None, node) else (1 if it.call(lt, [b, a], None, node) else 0))
    return lambda x: it.call(by, [x], None, node)


def seq(it, v, node):
    return list(it.values(v, node))


def f_print(it, args, kw, node):
    line = " ".join(it.show(a) for a in args)
    if it.capture:
        it.out.append(line)
    else:
        print(line)


def f_minmax(pick):
    def f(it, args, kw, node):
        xs = args if len(args) > 1 else seq(it, args[0], node)
        if not xs:
            it.fail("min/max of nothing", node)
        best = xs[0]
        for x in xs[1:]:
            if (x < best) if pick < 0 else (best < x):
                best = x
        return best
    return f


def f_abs(it, args, kw, node):
    return it.int(abs(args[0]), node) if type(args[0]) is int else abs(args[0])


def f_sum(it, args, kw, node):
    xs = seq(it, args[0], node)
    acc = 0
    for x in xs:
        acc = it.binop("+", acc, x, node)
    return acc


def f_sort(it, args, kw, node):
    xs = args[0]
    if type(xs) not in (CList, CArray):
        it.fail("sort needs a list or an array", node)
    xs.items.sort(key=keyfn(it, kw.get("by"), node))


def f_sorted(it, args, kw, node):
    return CList(sorted(seq(it, args[0], node), key=keyfn(it, kw.get("by"), node)))


def f_rand(it, args, kw, node):
    return it.rng.randint(args[0], args[1])


def f_shuffle(it, args, kw, node):
    it.rng.shuffle(args[0].items)


def f_laws_hold(it, args, kw, node):
    """Check every `law` of an operation value on all tuples of the samples."""
    o = args[0]
    if type(o) is not Obj:
        it.fail("laws_hold needs an operation value", node)
    default = seq(it, args[1], node) if len(args) > 1 else None
    for law in o.stype.decl.laws:
        pools = []
        for p in law.params:
            tn = p.ty.name if p.ty is not None and p.ty.kind == "tname" else None
            xs = seq(it, kw[tn], node) if tn in kw else default
            if xs is None:
                it.fail(f"laws_hold: no samples for `{p.name}: {tn}` of law `{law.name}`", node)
            pools.append(xs)
        total = 1
        for xs in pools:
            total *= len(xs)
        if total <= 4000:
            combos = itertools.product(*pools)
        else:
            combos = (tuple(it.rng.choice(xs) for xs in pools) for _ in range(4000))
        for c in combos:
            env = Env(Env(o.stype.env, o.f), {p.name: x for p, x in zip(law.params, c)})
            saved = it.mod
            it.mod = o.stype.decl.mod
            try:
                ok = it.ev(law.expr, env)
            finally:
                it.mod = saved
            if not ok:
                vals = ", ".join(f"{p.name} = {it.show(x)}" for p, x in zip(law.params, c))
                it.fail(f"{o.stype!r} breaks law `{law.name}` for {vals}", node)
    return True


def int_fn(f):
    def g(it, args, kw, node):
        x = args[0]
        if type(x) is not int:
            it.fail(f"expected an Int, got {it.show(x)}", node)
        return f(x)
    return g


def f_bit_ceil(x):
    return 1 if x <= 1 else 1 << (x - 1).bit_length()


def f_ctz(x):
    return 64 if x == 0 else (x & -x).bit_length() - 1


def type_call(it, tv, args, kw, node):
    n = tv.name
    if n == "List":
        return CList(seq(it, args[0], node) if args else [])
    if n == "Str":
        if not args:
            return CList([], True)
        a = args[0]
        if type(a) is CList and all(type(c) is str for c in a.items):
            return CList(list(a.items), True)
        return CList(list(it.show(a)), True)
    if n == "Set":
        return CSet(set(seq(it, args[0], node)) if args else set())
    if n == "Map":
        return CMap({})
    x = args[0]
    if n == "Int":
        if type(x) is float:
            return it.int(math.floor(x), node)
        if type(x) is str:
            return ord(x)
        if type(x) is bool:
            return int(x)
        if type(x) is CList and x.is_str:
            return int("".join(x.items))
        return x
    if n == "Float":
        return float(x)
    if n == "Char":
        if type(x) is int and 0 <= x < 128:
            return chr(x)
        it.fail(f"Char({it.show(x)}): a Char is one ASCII byte", node)
    if n == "Bool":
        return bool(x)
    it.fail(f"cannot call {tv!r}", node)


def dom_at(it, d, i, node, what="index"):
    if type(i) is not int or not 0 <= i < d.K:
        it.fail(f"{what} {it.show(i)} is outside {d!r} (0..{d.K})", node)
    return DomVal(d, i)


def dom_try(d, i):
    return DomVal(d, i) if 0 <= i < d.K else None


def install(it):
    it.capture = False
    it.type_call = type_call
    g = it.globals.vars
    for n in ("Int", "Float", "Bool", "Char", "Str", "List", "Set", "Map", "Iter", "Opt", "Range", "Bounds"):
        g[n] = TypeV(n)
    g.update({
        "print": f_print, "min": f_minmax(-1), "max": f_minmax(1), "abs": f_abs, "sum": f_sum,
        "sort": f_sort, "sorted": f_sorted, "rand": f_rand, "shuffle": f_shuffle,
        "laws_hold": f_laws_hold,
        "some": lambda it, a, k, n: a[0],
        "popcount": int_fn(lambda x: bin(x & ((1 << 64) - 1)).count("1")),
        "ctz": int_fn(f_ctz),
        "bit_width": int_fn(lambda x: x.bit_length()),
        "bit_ceil": int_fn(f_bit_ceil),
        "isqrt": int_fn(math.isqrt),
        "sqrt": lambda it, a, k, n: math.sqrt(a[0]),
        "atan2": lambda it, a, k, n: math.atan2(a[0], a[1]),
        "copy": lambda it, a, k, n: copyv(a[0]),
    })

    def fail_empty(it, o, node):
        it.fail("the list is empty", node)

    def l_pop(it, o, a, k, n):
        if not o.items:
            it.fail("pop from an empty list", n)
        return o.items.pop()

    def l_swap(it, o, a, k, n):
        i, j = it.offset(o, [a[0]], n), it.offset(o, [a[1]], n)
        o.items[i], o.items[j] = o.items[j], o.items[i]

    def l_resize(it, o, a, k, n):
        del o.items[a[0]:]
        while len(o.items) < a[0]:
            o.items.append(copyv(a[1]))

    it.props = {
        CList: {
            "len": lambda it, o, n: len(o.items),
            "is_empty": lambda it, o, n: not o.items,
            "first": lambda it, o, n: o.items[0] if o.items else fail_empty(it, o, n),
            "last": lambda it, o, n: o.items[-1] if o.items else fail_empty(it, o, n),
            "rev": lambda it, o, n: CList(o.items[::-1], o.is_str),
        },
        CArray: {
            "len": lambda it, o, n: len(o.items),
        },
        RangeVal: {
            "len": lambda it, o, n: len(o),
            "is_empty": lambda it, o, n: len(o) == 0,
            "lo": lambda it, o, n: o.lo if o.dom is None else DomVal(o.dom.bounds, o.lo),
            "hi": lambda it, o, n: o.hi if o.dom is None else DomVal(o.dom.bounds, o.hi),
            "first": lambda it, o, n: None if len(o) == 0 else (o.lo if o.dom is None else DomVal(o.dom, o.lo)),
            "last": lambda it, o, n: None if len(o) == 0 else (o.hi - 1 if o.dom is None else DomVal(o.dom, o.hi - 1)),
            "rev": lambda it, o, n: CList(list(o.values())[::-1]),
        },
        Domain: {
            "K": lambda it, o, n: o.K,
            "bounds": lambda it, o, n: o.bounds,
            "first": lambda it, o, n: dom_at(it, o, 0, n, "the first position"),
            "last": lambda it, o, n: dom_at(it, o, o.K - 1, n, "the last position"),
            "all": lambda it, o, n: RangeVal(o, 0, o.K) if o.of is None else it.fail("`.all` is for positions", n),
            "rev": lambda it, o, n: CList([DomVal(o, i) for i in range(o.K - 1, -1, -1)]),
            "name": lambda it, o, n: CList(list(o.name), True),
        },
        DomVal: {
            "to_idx": lambda it, o, n: o.ord,
            "ord": lambda it, o, n: o.ord,
            "label": lambda it, o, n: o.label,
            "l": lambda it, o, n: DomVal(o.dom.bounds, o.ord),
            "r": lambda it, o, n: DomVal(o.dom.bounds, o.ord + 1),
            "before": lambda it, o, n: DomVal(o.dom.bounds, o.ord),
            "after": lambda it, o, n: DomVal(o.dom.bounds, o.ord + 1),
            "next": lambda it, o, n: dom_try(o.dom, o.ord + 1),
            "prev": lambda it, o, n: dom_try(o.dom, o.ord - 1),
            "left": lambda it, o, n: dom_try(o.dom.of, o.ord - 1) if o.dom.of else it.fail("`.left` is for a boundary", n),
            "right": lambda it, o, n: dom_try(o.dom.of, o.ord) if o.dom.of else it.fail("`.right` is for a boundary", n),
        },
        str: {
            "ord": lambda it, o, n: ord(o),
            "is_digit": lambda it, o, n: o.isdigit(),
            "is_lower": lambda it, o, n: "a" <= o <= "z",
            "is_upper": lambda it, o, n: "A" <= o <= "Z",
        },
        TypeV: {
            "MAX": lambda it, o, n: I64_MAX if o == INT else math.inf,
            "MIN": lambda it, o, n: I64_MIN if o == INT else -math.inf,
            "zero": lambda it, o, n: 0 if o == INT else (0.0 if o == FLOAT else it.fail(f"{o!r} has no zero", n)),
            "one": lambda it, o, n: 1 if o == INT else (1.0 if o == FLOAT else it.fail(f"{o!r} has no one", n)),
        },
        CSet: {"len": lambda it, o, n: len(o.items)},
        CMap: {"len": lambda it, o, n: len(o.items)},
    }

    def bound_at(it, o, i, node, k):
        b = o.dom.bounds
        if not 0 <= i <= o.dom.K:
            it.fail(f"a window of {k} at {it.show(o)} leaves {o.dom!r}", node)
        return DomVal(b, i)

    it.vmethods = {
        CList: {
            "push": lambda it, o, a, k, n: o.items.append(a[0]),
            "pop": l_pop,
            "clear": lambda it, o, a, k, n: o.items.clear(),
            "copy": lambda it, o, a, k, n: copyv(o),
            "contains": lambda it, o, a, k, n: a[0] in o.items,
            "reverse": lambda it, o, a, k, n: o.items.reverse(),
            "sort": lambda it, o, a, k, n: o.items.sort(key=keyfn(it, k.get("by") if k else None, n)),
            "swap": l_swap,
            "extend": lambda it, o, a, k, n: o.items.extend(seq(it, a[0], n)),
            "resize": l_resize,
            "truncate": lambda it, o, a, k, n: o.items.__delitem__(slice(a[0], None)),
        },
        CArray: {
            "copy": lambda it, o, a, k, n: copyv(o),
            "fill": lambda it, o, a, k, n: o.items.__setitem__(slice(None), [copyv(a[0]) for _ in o.items]),
        },
        RangeVal: {
            "contains": lambda it, o, a, k, n: it.contains(o, a[0], n),
        },
        Domain: {
            "nth": lambda it, o, a, k, n: dom_at(it, o, a[0], n),
            "from_idx": lambda it, o, a, k, n: dom_at(it, o, a[0], n),
            "try_at": lambda it, o, a, k, n: dom_try(o, a[0]),
            "try": lambda it, o, a, k, n: dom_try(o, (ord(a[0]) - ord(o.lo)) if o.char else a[0] - o.lo),
        },
        DomVal: {
            "offset": lambda it, o, a, k, n: dom_try(o.dom, o.ord + a[0]),
            "cyc": lambda it, o, a, k, n: DomVal(o.dom, (o.ord + a[0]) % o.dom.K),
            "l": lambda it, o, a, k, n: bound_at(it, o, o.ord + 1 - a[0], n, a[0]),
            "r": lambda it, o, a, k, n: bound_at(it, o, o.ord + a[0], n, a[0]),
        },
        TypeV: {
            "filled": lambda it, o, a, k, n: CList([copyv(a[1]) for _ in range(a[0])]),
        },
        CSet: {
            "add": lambda it, o, a, k, n: o.items.add(a[0]),
            "remove": lambda it, o, a, k, n: o.items.discard(a[0]),
            "contains": lambda it, o, a, k, n: a[0] in o.items,
        },
        CMap: {
            "get": lambda it, o, a, k, n: o.items.get(a[0]),
            "contains": lambda it, o, a, k, n: a[0] in o.items,
            "remove": lambda it, o, a, k, n: o.items.pop(a[0], None),
        },
    }
