import copy
import re
import json
"""Type checker + C++ emitter (one pass). See PLAN.md for representation decisions."""
from .ast import N
from .types import *

class CompileError(Exception):
    pass

class TPerm(TArr):
    pass

class TMap(T):
    def __init__(self, key, val, hashed=False): self.key, self.val, self.hashed = key, val, hashed
    def __eq__(self, o): return isinstance(o, TMap) and (o.key, o.val, o.hashed) == (self.key, self.val, self.hashed)
    def __hash__(self): return hash(("map", self.key, self.val, self.hashed))
    def __repr__(self): return f"{'HashMap' if self.hashed else 'Map'}<{self.key!r}, {self.val!r}>"

class TSet(T):
    def __init__(self, elem): self.elem = elem
    def __eq__(self, o): return isinstance(o, TSet) and o.elem == self.elem
    def __hash__(self): return hash(("set", self.elem))
    def __repr__(self): return f"Set<{self.elem!r}>"

class TGen(T):
    def __init__(self, elem): self.elem = elem
    def __eq__(self, o): return isinstance(o, TGen) and o.elem == self.elem
    def __hash__(self): return hash(("gen", self.elem))
    def __repr__(self): return f"yields {self.elem!r}"

class TEnum(T):
    tags = None
    def __init__(self, name, cpp):
        self.name, self.cpp, self.variants = name, cpp, []   # (vname, [fname], [ftype])
    def __eq__(self, o): return self is o
    def __hash__(self): return id(self)
    def __repr__(self): return self.name
    def variant(self, vn):
        for k, v in enumerate(self.variants):
            if v[0] == vn:
                return k, v
        return None, None

class THeap(T):
    def __init__(self, elem): self.elem = elem
    def __eq__(self, o): return isinstance(o, THeap) and o.elem == self.elem
    def __hash__(self): return hash(("heap", self.elem))
    def __repr__(self): return f"MinHeap<{self.elem!r}>"

CPP_RESERVED = {"new", "delete", "int", "long", "auto", "class", "default", "register", "this", "template",
                "typename", "union", "unsigned", "signed", "char", "double", "float", "short", "switch", "case",
                "do", "goto", "static", "extern", "friend", "virtual", "public", "private", "protected", "operator",
                "sizeof", "typedef", "volatile", "mutable", "namespace", "using", "try", "catch", "throw",
                "explicit", "inline", "export", "bool", "void", "main", "std", "vector", "string", "ll", "max",
                "min", "abs", "swap", "sort", "data", "end", "begin", "count", "rank", "index", "left", "right",
                "y0", "y1", "j0", "j1", "next", "prev", "hash", "array", "list", "set", "map", "pair", "time"}

CPP_RESERVED = set(CPP_RESERVED) | {
    "size", "data", "begin", "end", "count", "find", "gcd", "lcm", "swap", "min", "max",
    "move", "distance", "next", "prev", "left", "right", "hash", "sort", "reverse", "fill",
    "copy", "rank", "abs", "pow", "exp", "log", "sqrt", "y0", "y1", "j0", "j1", "jn", "yn",
    "index", "time", "clock", "rand", "array", "list", "map", "set", "queue", "stack",
    "vector", "string", "pair", "tuple", "greater", "less", "plus", "minus", "function",
    "accumulate", "iota", "unique", "partition", "merge", "remove", "replace", "search",
    "empty", "ws", "dec", "hex", "fixed", "bitset", "deque", "complex", "ref", "apply",
    "visit", "any", "all", "signal", "div", "ldiv", "exit", "free", "link", "read", "write",
    "select", "stdin", "stdout", "floor", "ceil", "round", "lower_bound", "upper_bound",
}

class Sym:
    def __init__(self, kind, ty, cpp, mut=False, node=None):
        self.kind, self.ty, self.cpp, self.mut, self.node = kind, ty, cpp, mut, node
        self.fact = None

def unparse(e):
    k = e.kind
    if k == "num": return str(e.value)
    if k == "name": return e.name
    if k == "binop": return f"{unparse(e.a)} {e.op} {unparse(e.b)}"
    if k == "unop": return f"{e.op}{' ' if e.op == 'not' else ''}{unparse(e.a)}"
    if k == "index": return f"{unparse(e.obj)}[{', '.join(map(unparse, e.idx))}]"
    if k == "field": return f"{unparse(e.obj)}.{e.name}"
    if k == "call": return f"{unparse(e.fn)}(...)"
    if k == "paren": return f"({unparse(e.a)})"
    return k

class Compiler:
    def __init__(self, prog, var_iv=None):
        self.prog = prog
        self.scopes = [{}]
        self.structs = {}          # key -> TTuple
        self.struct_defs = []
        self.globals = []          # C++ global declarations
        self.funcs = []            # C++ function definitions
        self.init_lines, self.solve_lines = [], []
        self.solutions = {}        # name -> lines
        self.gen_lines = []        # generator mode body
        self.checks = []           # (line, depth, desc)
        self.errors = []
        self.var_iv = var_iv or {}  # decl id -> (lo, hi)
        self.var_iv_next = {}
        self.loop_trips = []       # stack of max trip counts (None unknown)
        self.decl_depth = {}       # decl id -> len(loop_trips) at declaration
        self.contrib = {}          # decl id -> list of (lo, hi) contributions
        self.contrib_nodes = {}
        self.last_read = 0
        self.reads_in_fns = False
        self.out = None
        self.ind = 1
        self.fn_ret = []
        self.used_names = [set()]
        self.at_top = True
        self.tests_ty = None
        self.tmp = 0
        self.dom_count = 0

    # ---------- diagnostics
    def error(self, node, msg):
        raise CompileError(f"line {node.line}: {msg}")

    def note_check(self, node, desc):
        self.checks.append((node.line, len(self.loop_trips), desc))

    # ---------- scopes
    def lookup(self, name):
        for s in reversed(self.scopes):
            if name in s:
                return s[name]
        return None

    def fresh(self, base):
        base = base if base not in CPP_RESERVED else base + "_"
        used = self.used_names[-1]
        nm, k = base, 0
        while nm in used:
            k += 1
            nm = f"{base}_{k}"
        used.add(nm)
        return nm

    def tmpname(self, base="t"):
        self.tmp += 1
        return f"_{base}{self.tmp}"

    def emit(self, line):
        self.out.append("    " * self.ind + line)

    # ---------- C++ types
    def cpp_type(self, t):
        if isinstance(t, TInt): return "ll"
        if isinstance(t, TBool): return "bool"
        if isinstance(t, TChar): return "char"
        if isinstance(t, TStr): return "string"
        if isinstance(t, TUnit): return "void"
        if isinstance(t, TDom): return "int"
        if isinstance(t, TMod): return f"Mod<{t.m}>"
        if isinstance(t, TOpt):
            if self.narrow(t.inner) and t.inner.lo > -2**31:
                return "OptI32"
            return f"Opt<{self.cpp_type(t.inner)}>"
        if isinstance(t, TRange): return "Rng"
        if isinstance(t, TTuple): return t.cpp
        if isinstance(t, TList): return f"vector<{'int' if self.narrow(t.elem) else self.cpp_type(t.elem)}>"
        if isinstance(t, TMap):
            if isinstance(t.key, TDom):
                return f"DenseMap<{self.cpp_type(t.val)}>"
            if t.hashed:
                return f"unordered_map<{self.cpp_type(t.key)}, {self.cpp_type(t.val)}, CpHash>"
            return f"map<{self.cpp_type(t.key)}, {self.cpp_type(t.val)}>"
        if isinstance(t, TSet):
            if self.word_set(t):
                return "Mask64"
            return "DenseSet" if self.dense_set(t) else f"set<{self.cpp_type(t.elem)}>"
        if isinstance(t, TArr): return f"vector<{'int' if self.narrow(t.elem) else self.cpp_type(t.elem)}>"
        if isinstance(t, TEnum): return t.cpp
        if isinstance(t, THeap):
            e = self.cpp_type(t.elem)
            return f"priority_queue<{e}, vector<{e}>, greater<{e}>>"
        raise CompileError(f"no C++ type for {t!r}")

    def tuple_type(self, names, types):
        key = (tuple(names), tuple(types))
        if key in self.structs:
            return self.structs[key]
        cpp = f"Tup{len(self.structs)}"
        names = [n if n else f"f{i}" for i, n in enumerate(names)]
        t = TTuple(names, list(types), cpp)
        self.structs[key] = t
        fields = " ".join(f"{'I32' if self.narrow(ft) else self.cpp_type(ft)} {self.field_cpp(fn)};" for fn, ft in zip(names, types))
        tie = ", ".join(self.field_cpp(fn) for fn in names)
        self.struct_defs.append(
            f"struct {cpp} {{ {fields}\n"
            f"    auto tie() const {{ return std::tie({tie}); }}\n"
            + (f"    bool operator<(const {cpp}& o) const {{ return tie() < o.tie(); }}\n"
               f"    bool operator>(const {cpp}& o) const {{ return o.tie() < tie(); }}\n"
               if not any(isinstance(x, TOpt) for x in types) else "") +
            f"    bool operator==(const {cpp}& o) const {{ return tie() == o.tie(); }}\n"
            f"    bool operator!=(const {cpp}& o) const {{ return !(*this == o); }} }};")
        return t

    def field_cpp(self, name):
        return name + "_" if name in CPP_RESERVED else name

    # ---------- domains
    def new_domain(self, name, lo_c, len_c, lo_iv, len_iv, anon=False, labels="int"):
        self.dom_count += 1
        cpp = f"D{self.dom_count}_{name}" if not anon else f"D{self.dom_count}"
        info = DomInfo(name, cpp, f"{cpp}_lo", f"{cpp}_len", lo_iv, len_iv, anon=anon, labels=labels)
        self.globals.append(f"ll {cpp}_lo, {cpp}_len;")
        return info, lo_c, len_c

    def domain_from_range(self, rng, name, node, anon=False):
        """rng: a `range` node. Emits lo/len assignments; returns DomInfo."""
        lo_c, lo_t = self.expr(rng.lo)
        if rng.hi is None:
            self.error(node, "domain needs an upper bound")
        hi_c, hi_t = self.expr(rng.hi)
        labels = "char" if isinstance(lo_t, TChar) else "int"
        if not (is_int(lo_t) or labels == "char") or not (is_int(hi_t) or isinstance(hi_t, TChar)):
            self.error(node, f"domain bounds must be Int, got {lo_t!r}..{hi_t!r}")
        len_c = f"({hi_c}) - ({lo_c})" + (" + 1" if rng.incl else "")
        lo_iv = (getattr(lo_t, "lo", None), getattr(lo_t, "hi", None))
        hl, hh = getattr(hi_t, "lo", None), getattr(hi_t, "hi", None)
        d = 1 if rng.incl else 0
        len_iv = (None if hl is None or lo_iv[1] is None else max(0, hl - lo_iv[1] + d),
                  None if hh is None or lo_iv[0] is None else hh - lo_iv[0] + d)
        info, _, _ = self.new_domain(name, lo_c, len_c, lo_iv, len_iv, anon=anon, labels=labels)
        info.lo_src, info.hi_src = lo_c, f"({hi_c})" + (" + 1" if rng.incl else "")
        self.emit(f"{info.cpp}_lo = {lo_c}; {info.cpp}_len = max(0LL, (ll)({len_c}));")
        return info

    def bounds_of(self, info):
        if info.bounds is None:
            b = DomInfo(info.name + ".bounds", info.cpp + "b", info.lo_cpp, f"({info.len_cpp} + 1)",
                        info.lo_iv, (None if info.len_iv[0] is None else info.len_iv[0] + 1,
                                     None if info.len_iv[1] is None else info.len_iv[1] + 1))
            b.of = info
            info.bounds = b
        return info.bounds

    def dom_label(self, info, c):
        return f"((ll)({c}) + {info.lo_cpp})"

    # ---------- type expressions
    def resolve_type(self, tn):
        k = tn.kind
        if k == "tyields":
            return TGen(self.resolve_type(tn.inner))
        if k == "tname":
            nm = tn.name
            prim = {"Int": INT, "Bool": BOOL, "Char": CHAR, "Str": STR}
            if nm in prim:
                return prim[nm]
            if nm == "List":
                return TList(self.resolve_type(tn.args[0]))
            if nm in ("Map", "HashMap"):
                return TMap(self.resolve_type(tn.args[0]), self.resolve_type(tn.args[1]), nm == "HashMap")
            if nm == "Set":
                return TSet(self.resolve_type(tn.args[0]))
            if nm == "Perm":
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom):
                    self.error(tn, "Perm<D> needs a domain")
                pos, _, _ = self.new_domain(f"Pos<{d.info.name}>", "0", d.info.len_cpp, (0, 0), d.info.len_iv, anon=True)
                pos.lo_src, pos.hi_src = "0", d.info.len_cpp
                self.emit(f"{pos.cpp}_lo = 0; {pos.cpp}_len = {d.info.len_cpp};")
                return TPerm([TDom(pos)], d)
            if nm == "MinHeap":
                return THeap(self.resolve_type(tn.args[0]))
            if nm == "WithMin":
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom):
                    self.error(tn, "WithMin<D> needs a domain")
                if getattr(d.info, "withmin", None) is None:
                    b = d.info
                    lo_iv = tuple(None if x is None else x - 1 for x in b.lo_iv)
                    len_iv = tuple(None if x is None else x + 1 for x in b.len_iv)
                    w, _, _ = self.new_domain(f"WithMin<{b.name}>", "", "", lo_iv, len_iv, anon=True)
                    w.name, w.base, w.anon = f"WithMin<{b.name}>", b, False
                    w.lo_src, w.hi_src = f"({b.lo_cpp} - 1)", getattr(b, "hi_src", None)
                    self.emit(f"{w.cpp}_lo = {b.lo_cpp} - 1; {w.cpp}_len = {b.len_cpp} + 1;")
                    b.withmin = TDom(w)
                return d.info.withmin
            if nm == "Mod":
                a = tn.args[0]
                v = a.value if a.kind == "num" else self.const_value(a)
                return TMod(v)
            if nm == "Range":
                return TRange(self.resolve_type(tn.args[0]))
            s = self.lookup(nm)
            if s and s.kind == "type":
                return s.ty
            if s and s.kind in ("var", "let") and is_int(s.ty):
                return self.anon_count_domain(f"0..{nm}", "0", s.cpp, (0, 0), (s.ty.lo, s.ty.hi), tn)
            self.error(tn, f"unknown type {nm}")
        if k == "trange":
            r = tn.range
            lo_c, lo_t = self.expr(r.lo)
            hi_c, hi_t = self.expr(r.hi) if r.hi is not None else (None, TInt())
            hi_v = getattr(hi_t, "hi", None)
            if hi_v is not None and not r.incl:
                hi_v -= 1
            return TInt(getattr(lo_t, "lo", None), hi_v)
        if k == "topt":
            return TOpt(self.resolve_type(tn.inner))
        if k == "tlist":
            return TList(self.resolve_type(tn.elem))
        if k == "ttuple":
            names = [n for n, _ in tn.fields]
            types = [self.resolve_type(t) for _, t in tn.fields]
            return self.tuple_type(names, types)
        if k == "tarray":
            dims = [self.dim_type(d) for d in tn.dims]
            return TArr(dims, self.resolve_type(tn.elem))
        if k == "texpr":
            c, t = self.expr(tn.expr)
            if isinstance(t, TType):
                return t.t
            self.error(tn, "expected a type")
        self.error(tn, f"unsupported type form {k}")

    def anon_count_domain(self, name, lo_c, hi_c_excl, lo_iv, hi_iv, node):
        info, _, _ = self.new_domain(name, lo_c, f"({hi_c_excl}) - ({lo_c})", lo_iv,
                                     (None if hi_iv[0] is None else hi_iv[0] - (lo_iv[1] or 0),
                                      None if hi_iv[1] is None else hi_iv[1] - (lo_iv[0] or 0)), anon=True)
        info.lo_src, info.hi_src = lo_c, hi_c_excl
        self.emit(f"{info.cpp}_lo = {lo_c}; {info.cpp}_len = max(0LL, (ll)(({hi_c_excl}) - ({lo_c})));")
        return TDom(info)

    def dim_type(self, d):
        if d.kind == "trange":
            r = d.range
            lo_c, lo_t = self.expr(r.lo)
            hi_c, hi_t = self.expr(r.hi)
            hi_excl = f"({hi_c}) + 1" if r.incl else hi_c
            hiv = (getattr(hi_t, "lo", None), getattr(hi_t, "hi", None))
            if r.incl:
                hiv = tuple(None if x is None else x + 1 for x in hiv)
            return self.anon_count_domain(unparse(r), lo_c, hi_excl,
                                          (getattr(lo_t, "lo", None), getattr(lo_t, "hi", None)), hiv, d)
        t = self.resolve_type(d)
        if not isinstance(t, TDom):
            self.error(d, f"array dimension must be a domain or a range, got {t!r}")
        return t

    def const_value(self, e):
        if e.kind == "num":
            return e.value
        c, t = self.expr(e)
        if is_int(t) and t.lo is not None and t.lo == t.hi:
            return t.lo
        self.error(e, "expected a constant")

    # ---------- expressions: return (cpp, type)
    def int_result(self, node, lo, hi):
        if not fits(lo, hi):
            self.error(node, f"`{unparse(node)}` may reach {lo if lo is not None and lo < INT_MIN else hi:.3g}, "
                             f"beyond Int; widen with I128 or tighten the input bounds")
        return TInt(lo, hi)

    def coerce(self, c, t, want, node):
        """Convert expression (c, t) to type `want`; returns cpp or raises."""
        if want is None or t == want:
            return c
        if isinstance(want, TInt) and isinstance(t, TInt):
            return c
        if isinstance(want, TMod) and isinstance(t, TInt):
            return f"Mod<{want.m}>({c})"
        if isinstance(want, TOpt):
            if isinstance(t, TOpt) and t.inner == want.inner:
                return c
            if isinstance(t, TOpt) and isinstance(t.inner, TInt) and isinstance(want.inner, TInt):
                return c
            inner = self.coerce(c, t, want.inner, node)
            return f"{self.cpp_type(want)}({inner})"
        if isinstance(want, TDom) and isinstance(t, TDom) and getattr(want.info, "base", None) is t.info:
            return f"(int)(({c}) + 1)"
        if isinstance(want, TDom) and isinstance(t, TInt):
            info = want.info
            if t.lo is not None and t.hi is not None and info.lo_iv[0] is not None and \
               info.lo_iv[0] == info.lo_iv[1] and info.len_iv[0] is not None and \
               info.lo_iv[0] <= t.lo and t.hi < info.lo_iv[0] + info.len_iv[0]:
                return f"(int)(({c}) - {info.lo_cpp})"
            self.error(node, f"Int is not a {info.name}; convert with {info.name}(x) or test `x in {info.name}`")
        if isinstance(want, TArr) and isinstance(t, TArr) and len(want.dims) == len(t.dims) and \
           want.dims == t.dims and (want.elem == t.elem or (is_int(want.elem) and is_int(t.elem))):
            return c
        if isinstance(want, (TMap, TSet)) and isinstance(t, TList) and t.elem is None:
            return self.set_ctor(want)
        if isinstance(want, TList) and isinstance(t, TList) and (t.elem == want.elem or t.elem is None):
            return c if t.elem is not None else f"{self.cpp_type(want)}()"
        self.error(node, f"expected {want!r}, got {t!r}")

    def expr(self, e, want=None):
        m = getattr(self, "x_" + e.kind, None)
        if m is None:
            self.error(e, f"unsupported expression {e.kind}")
        return m(e, want)

    def x_num(self, e, want):
        v = e.value
        if isinstance(v, float):
            return repr(v), TInt()
        if isinstance(want, TMod):
            return f"Mod<{want.m}>({v}LL)", want
        return (f"{v}LL" if abs(v) > 2**31 - 1 else str(v)), TInt(v, v)

    def x_bool(self, e, want):
        return ("true" if e.value else "false"), BOOL

    def x_char(self, e, want):
        return repr(e.value).replace('"', '\\"') if e.value != "'" else "'\\''", CHAR

    def x_str(self, e, want):
        return f'string("{e.value}")', STR

    def x_paren(self, e, want):
        c, t = self.expr(e.a, want)
        return f"({c})", t

    def x_none(self, e, want):
        if not isinstance(want, TOpt):
            self.error(e, "`none` needs a known ?T type here")
        return f"{self.cpp_type(want)}()", want

    def x_name(self, e, want):
        if e.name == "inf" and not self.lookup("inf"):
            return "CP_INF", TInt(4e18, 4e18)
        s = self.lookup(e.name)
        if s is None:
            prim = {"Int": INT, "Bool": BOOL, "Char": CHAR}
            if e.name in prim:
                return "", TType(prim[e.name])
            self.error(e, f"unknown name {e.name}")
        if s.kind == "type":
            return "", TType(s.ty)
        if s.kind == "ctor":
            return self.construct(s, [], e)
        if s.kind == "fn":
            return s.cpp, s.ty
        t = s.ty
        if isinstance(t, TInt) and s.node is not None and id(s.node) in self.var_iv:
            lo, hi = self.var_iv[id(s.node)]
            t = TInt(lo, hi)
        return s.cpp, t

    def x_doblock(self, e, want):
        if want is None:
            want = self.infer_block_type(e)
        self.fn_ret.append(want)
        self.scopes.append({})
        saved_ctx, self.loop_ctx = getattr(self, "loop_ctx", []), []
        self.in_do = getattr(self, "in_do", 0) + 1
        try:
            lines, _ = self.capture(lambda: self.fn_body(e.body, want))
        finally:
            self.in_do -= 1
            self.loop_ctx = saved_ctx
            self.scopes.pop()
            self.fn_ret.pop()
        body = "\n".join("    " * self.ind + l for l in lines)
        return f"[&]() -> {self.cpp_type(want)} {{\n{body}\n{'    ' * self.ind}}}()", want

    def infer_block_type(self, e):
        last = e.body[-1] if e.body else None
        if last is None or last.kind != "exprstmt":
            self.error(e, "this block needs a declared type, e.g. `let x: ?Int = do: ...` (prototype)")
        def probe():
            self.scopes.append({})
            try:
                for st in e.body[:-1]:
                    self.stmt(st)
                return self.expr(last.expr)[1]
            finally:
                self.scopes.pop()
        saved = (list(self.funcs), list(self.struct_defs), dict(self.var_iv))
        _, t = self.capture(probe)
        self.funcs[:], self.struct_defs[:] = saved[0], saved[1]
        self.var_iv = saved[2]
        return TInt() if is_int(t) else t

    def x_try(self, e, want):
        self.error(e, "`?` must be the whole right-hand side of a let (prototype)")

    def x_tuple(self, e, want):
        if isinstance(want, TOpt):
            want = want.inner
        parts, types = [], []
        wt = want.types if isinstance(want, TTuple) and len(want.types) == len(e.items) else [None] * len(e.items)
        for it, w in zip(e.items, wt):
            c, t = self.expr(it, w)
            if w is not None:
                c = self.coerce(c, t, w, it)
                t = w
            parts.append(c)
            types.append(TInt() if is_int(t) else t)
        names = e.names if any(e.names) else (want.names if isinstance(want, TTuple) else [None] * len(types))
        tt = want if isinstance(want, TTuple) and len(want.types) == len(types) else self.tuple_type(names, types)
        return f"{tt.cpp}{{{', '.join(parts)}}}", tt

    def x_list(self, e, want):
        if (isinstance(want, TArr) and len(e.items) == 1 and e.items[0].kind == "gen"
                and len(e.items[0].clauses) == 1 and e.items[0].clauses[0][0] == "for"):
            g = e.items[0]
            _, pat, it = g.clauses[0]
            _, (c, t) = self.capture(lambda: self.expr(it))
            if len(want.dims) == 1 and isinstance(t, TType) and t.t == want.dims[0]:
                return self.x_table(N("table", e.line, dims=[it], pat=pat, body=g.elem), want)
        if len(e.items) == 1 and e.items[0].kind == "gen":
            return self.reduction("List", e.items[0], e, want)
        et = want.elem if isinstance(want, TList) else None
        parts = []
        for it in e.items:
            c, t = self.expr(it, et)
            if et is None:
                et = TInt() if is_int(t) else t
            parts.append(self.coerce(c, t, et, it))
        lt = TList(et)
        if et is None:
            return "{}", TList(None)
        return f"{self.cpp_type(lt)}{{{', '.join(parts)}}}", lt

    def x_unop(self, e, want):
        c, t = self.expr(e.a, want)
        if e.op == "not":
            if not isinstance(t, TBool):
                self.error(e, f"`not` needs Bool, got {t!r}")
            return f"!({c})", BOOL
        if isinstance(t, TInt):
            lo, hi = iv_neg(t)
            return f"-({c})", self.int_result(e, lo, hi)
        if isinstance(t, TMod):
            return f"-({c})", t
        self.error(e, f"cannot negate {t!r}")

    def x_binop(self, e, want):
        op = e.op
        if op in ("|", "&", "-") and e.a.kind == "name" and isinstance(getattr(self.lookup(e.a.name), "ty", None), TSet):
            a, at = self.expr(e.a)
            if True:
                b, bt = self.expr(e.b, at)
                if bt != at:
                    self.error(e, f"`{op}` needs two {at!r}, got {bt!r}")
                if not self.word_set(at):
                    self.error(e, f"set `{op}` is only supported for sets over domains of at most 64 elements (prototype)")
                m = {"|": "a.m | b.m", "&": "a.m & b.m", "-": "a.m & ~b.m"}[op]
                return f"([](Mask64 a, Mask64 b) {{ Mask64 r; r.m = {m}; return r; }}({a}, {b}))", at
        if op in ("and", "or"):
            a, at = self.expr(e.a)
            b, bt = self.expr(e.b)
            if not (isinstance(at, TBool) and isinstance(bt, TBool)):
                self.error(e, f"`{op}` needs Bool operands, got {at!r} and {bt!r}")
            return f"({a} {'&&' if op == 'and' else '||'} {b})", BOOL
        if op == "??":
            a, at = self.expr(e.a)
            if not isinstance(at, TOpt):
                self.error(e, f"`??` needs a ?T on the left, got {at!r}")
            b, bt = self.expr(e.b, at.inner)
            b = self.coerce(b, bt, at.inner, e.b)
            tmp = self.tmpname("o")
            return f"([&]{{ auto {tmp} = {a}; return {tmp}.has() ? {tmp}.get() : {self.cpp_type(at.inner)}({b}); }}())", at.inner
        a, at = self.expr(e.a, want if isinstance(want, TMod) else None)
        b, bt = self.expr(e.b, at if isinstance(at, TMod) else None)
        if isinstance(at, TOpt) or isinstance(bt, TOpt):
            self.error(e, f"`{op}` on an optional ({at!r}, {bt!r}); unwrap with `!`, `??` or `if let` first")
        if isinstance(at, TInt) and isinstance(bt, TMod):
            a, at = f"Mod<{bt.m}>({a})", bt
        if isinstance(at, TMod):
            if isinstance(bt, TInt) and op != "**":
                b = f"Mod<{at.m}>({b})"
            elif op == "**":
                return f"({a}).pow({b})", at
            if op not in "+-*/":
                self.error(e, f"`{op}` is not defined on Mod")
            return f"({a} {op} {b})", at
        if isinstance(at, TDom):
            info = at.info
            if op in "+-" and isinstance(bt, TInt):
                c = f"chk_dom((ll)({a}) {op} ({b}), {info.len_cpp}, {e.line})"
                self.note_check(e, f"`{unparse(e)}` stays in {info.name}")
                return c, at
            if op == "-" and isinstance(bt, TDom) and bt.info is info:
                return f"((ll)({a}) - ({b}))", TInt(None if info.len_iv[1] is None else -info.len_iv[1],
                                                     None if info.len_iv[1] is None else info.len_iv[1])
            self.error(e, f"`{op}` between {at!r} and {bt!r}; domain values support `i + k`, `i - k` and `i - j`")
        if isinstance(bt, TDom):
            self.error(e, f"`{op}` between {at!r} and {bt!r}; use `{unparse(e.b)}.ord` for its position")
        if isinstance(at, TChar) and isinstance(bt, TInt) and op in "+-":
            return f"(char)({a} {op} {b})", CHAR
        if isinstance(at, TChar) and isinstance(bt, TChar) and op == "-":
            return f"(ll)({a} - {b})", TInt(-255, 255)
        if not (isinstance(at, TInt) and isinstance(bt, TInt)):
            self.error(e, f"`{op}` needs Int operands, got {at!r} and {bt!r}")
        if op in "+-*":
            # Operands that fit in 32 bits can both be C++ `int` (literals are); when the
            # result may not fit, compute it in 64 bits.
            small = lambda t: None not in (t.lo, t.hi) and -2**31 <= t.lo and t.hi < 2**31
            lo, hi = {"+": iv_add, "*": iv_mul}[op](at, bt) if op != "-" else iv_add(at, TInt(*iv_neg(bt)))
            if small(at) and small(bt) and not small(TInt(lo, hi)):
                a = f"(ll){a}"
        if op == "+":
            return f"({a} + {b})", self.int_result(e, *iv_add(at, bt))
        if op == "-":
            nb = TInt(*iv_neg(bt))
            return f"({a} - {b})", self.int_result(e, *iv_add(at, nb))
        if op == "*":
            return f"({a} * {b})", self.int_result(e, *iv_mul(at, bt))
        if op in ("/", "%"):
            nz = e.b.kind == "name" and getattr(self.lookup(e.b.name), "nonzero", False)
            if not nz and bt.lo is not None and bt.hi is not None and bt.lo <= 0 <= bt.hi:
                self.error(e, f"`{unparse(e.b)}` may be 0 in `{unparse(e)}`")
            plain = at.lo is not None and at.lo >= 0 and bt.lo is not None and bt.lo > 0
            pos_b = bt.lo is not None and bt.lo > 0
            if op == "/":
                c = f"({a} / {b})" if plain else f"cp_ediv({a}, {b})"
                if pos_b and None not in (at.lo, at.hi, bt.hi):
                    qs = [at.lo // bt.lo, at.lo // bt.hi, at.hi // bt.lo, at.hi // bt.hi]
                    return c, TInt(min(qs), max(qs))
                return c, TInt()
            c = f"({a} % {b})" if plain else f"cp_emod({a}, {b})"
            if None not in (bt.lo, bt.hi):
                m = max(abs(bt.lo), abs(bt.hi))
                return c, TInt(0, m - 1)
            return c, TInt(0, None)
        if op == "**":
            if None not in (at.lo, at.hi, bt.lo, bt.hi) and bt.lo >= 0:
                hi = max(abs(at.lo), abs(at.hi)) ** bt.hi
                return f"cp_ipow({a}, {b})", self.int_result(e, -hi if at.lo < 0 else 0, hi)
            self.error(e, f"cannot bound `{unparse(e)}`; use Mod or I128")
        if op in ("&", "|", "^"):
            lo = 0 if (at.lo or 0) >= 0 and (bt.lo or 0) >= 0 else None
            hi = None
            if lo == 0 and at.hi is not None and bt.hi is not None:
                hi = (1 << max(at.hi, bt.hi).bit_length()) - 1
            return f"({a} {op} {b})", TInt(lo, hi)
        if op == "<<":
            if None not in (at.lo, at.hi, bt.hi) and at.lo >= 0:
                return f"({a} << {b})", self.int_result(e, 0, at.hi << bt.hi)
            return f"({a} << {b})", TInt()
        if op == ">>":
            return f"({a} >> {b})", TInt(0 if (at.lo or 0) >= 0 else None, at.hi)
        self.error(e, f"unknown operator {op}")

    def x_cmp(self, e, want):
        parts = []
        left_c, left_t = self.expr(e.first)
        for op, rn in zip(e.ops, e.rest):
            if op in ("in", "not in"):
                c = self.membership(left_c, left_t, rn, e)
                parts.append(c if op == "in" else f"!({c})")
                rc, rt = None, None
            else:
                want_r = left_t.inner if isinstance(left_t, TOpt) and rn.kind == "none" else None
                if rn.kind == "none":
                    if not isinstance(left_t, TOpt) or op not in ("==", "!="):
                        self.error(e, f"`{op} none` needs a ?T and == / !=")
                    parts.append(f"{'!' if op == '==' else ''}({left_c}).has()")
                    continue
                rc, rt = self.expr(rn)
                lt = left_t
                if (isinstance(lt, TOpt) or isinstance(rt, TOpt)) and op not in ("==", "!="):
                    self.error(e, f"`{op}` on an optional ({lt!r} vs {rt!r}): ?T has no ordering; "
                                  f"handle `none` explicitly")
                if isinstance(lt, TDom) != isinstance(rt, TDom) or (
                        isinstance(lt, TDom) and lt.info is not rt.info):
                    self.error(e, f"comparing {lt!r} with {rt!r}")
                if isinstance(lt, TOpt) and not isinstance(rt, TOpt):
                    rc = self.coerce(rc, rt, lt, rn)
                elif isinstance(rt, TOpt) and not isinstance(lt, TOpt):
                    left_c = self.coerce(left_c, lt, rt, e.first)
                parts.append(f"({left_c} {op} {rc})")
            left_c, left_t = rc, rt
        return "(" + " && ".join(parts) + ")", BOOL

    def membership(self, xc, xt, rn, e):
        rc, rt = self.expr(rn)
        if isinstance(rt, TType) and isinstance(rt.t, TDom):
            info = rt.t.info
            if isinstance(xt, TDom):
                return "true"
            if not isinstance(xt, TInt):
                self.error(e, f"`in {info.name}` needs an Int, got {xt!r}")
            return f"((ll)({xc}) - {info.lo_cpp} >= 0 && (ll)({xc}) - {info.lo_cpp} < {info.len_cpp})"
        if isinstance(rt, TRange):
            return f"([&](ll _x, Rng _r){{ return _r.lo <= _x && _x < _r.hi; }}({xc}, {rc}))"
        if isinstance(rt, (TMap, TSet)):
            kt = rt.key if isinstance(rt, TMap) else rt.elem
            return f"({rc}.count({self.coerce(xc, xt, kt, e)}) > 0)"
        if isinstance(rt, (TList, TArr)):
            return f"(find({rc}.begin(), {rc}.end(), {xc}) != {rc}.end())"
        self.error(e, f"cannot test membership in {rt!r}")

    def x_range(self, e, want):
        lo, lt = self.expr(e.lo)
        if e.hi is None:
            self.error(e, "open range needs an upper bound here")
        hi, ht = self.expr(e.hi)
        if isinstance(lt, TDom) and isinstance(ht, TDom) and lt.info is ht.info:
            return f"Rng{{{lo}, (ll)({hi}){' + 1' if e.incl else ''}}}", TRange(lt)
        if isinstance(lt, TInt) and isinstance(ht, TInt):
            return f"Rng{{{lo}, (ll)({hi}){' + 1' if e.incl else ''}}}", TRange(INT)
        self.error(e, f"range between {lt!r} and {ht!r}")

    def x_unwrap(self, e, want):
        c, t = self.expr(e.a)
        if not isinstance(t, TOpt):
            self.error(e, f"`!` on a non-optional {t!r}")
        self.note_check(e, f"`{unparse(e.a)}!` is not none")
        return f"opt_unwrap({c}, {e.line})", t.inner

    def x_ifexpr(self, e, want):
        cc, ct = self.expr(e.cond)
        if not isinstance(ct, TBool):
            self.error(e, f"condition must be Bool, got {ct!r}")
        a, at = self.expr(e.a, want)
        b, bt = self.expr(e.b, want or at)
        t = want or at
        if is_int(at) and is_int(bt) and want is None:
            t = TInt(*iv_join(at, bt))
        return f"({cc} ? {self.coerce(a, at, t, e.a)} : {self.coerce(b, bt, t, e.b)})", t

    def x_index(self, e, want):
        oc, ot = self.expr(e.obj)
        if isinstance(ot, TMap):
            kc, kt = self.expr(e.idx[0], ot.key)
            kc = self.coerce(kc, kt, ot.key, e.idx[0])
            lv = getattr(self, "lvalue", None)
            if lv is not None and lv[0] is e and lv[1] == "=":
                return f"{oc}[{kc}]", ot.val
            self.note_check(e, f"key `{unparse(e.idx[0])}` is in the map")
            return f"cp_mget({oc}, {kc}, {e.line})", ot.val
        if isinstance(ot, TList):
            if len(e.idx) != 1:
                self.error(e, "lists take one index")
            ic, it = self.expr(e.idx[0])
            if not is_int(it):
                self.error(e, f"list index must be Int, got {it!r}")
            self.note_check(e, f"`{unparse(e)}` in bounds")
            lv = getattr(self, "lvalue", None)
            if self.narrow(ot.elem) and not (lv is not None and lv[0] is e):
                return f"(ll){oc}[cp_lidx({ic}, {oc}.size(), {e.line})]", ot.elem
            return f"{oc}[cp_lidx({ic}, {oc}.size(), {e.line})]", ot.elem
        if not isinstance(ot, TArr):
            self.error(e, f"cannot index {ot!r}")
        if len(e.idx) == 1 and len(ot.dims) > 1:
            self.error(e, f"{ot!r} is indexed as g[{', '.join(d.info.name.lower()[:1] or 'i' for d in ot.dims)}]; for a whole row use .row(...)")
        if len(e.idx) != len(ot.dims):
            self.error(e, f"{ot!r} needs {len(ot.dims)} indices, got {len(e.idx)}")
        flat = None
        for ix, d in zip(e.idx, ot.dims):
            ic = self.index_into(ix, d.info, e)
            flat = ic if flat is None else f"({flat}) * {d.info.len_cpp} + ({ic})"
        lv = getattr(self, "lvalue", None)
        if self.narrow(ot.elem) and not (lv is not None and lv[0] is e):
            return f"(ll){oc}[{flat}]", ot.elem
        return f"{oc}[{flat}]", ot.elem

    def index_into(self, ix, info, e):
        ic, it = self.expr(ix)
        if isinstance(it, TDom):
            if it.info is info:
                return ic
            if getattr(info, "base", None) is it.info:
                return f"(({ic}) + 1)"
            if info.of is it.info or it.info.of is info:
                self.error(ix, f"index of type {it!r} into array over {info.name}; use .before/.after for cuts")
            self.error(ix, f"index of type {it!r} into array over {info.name}")
        if isinstance(it, TInt):
            if not info.anon:
                self.error(ix, f"Int index `{unparse(ix)}` into array over {info.name}; "
                               f"convert with {info.name}(x) or test `x in {info.name}`")
            off = f"((ll)({ic}) - {info.lo_cpp})" if getattr(info, "lo_src", "0") != "0" else f"(ll)({ic})"
            if ix.kind == "name":
                s = self.lookup(ix.name)
                if s and s.fact and s.fact == (info.lo_src, info.hi_src):
                    return f"(int)({off})"
            if it.lo is not None and it.hi is not None and info.lo_iv[0] is not None and \
               info.lo_iv[0] == info.lo_iv[1] and info.len_iv[0] is not None and \
               it.lo >= info.lo_iv[0] and it.hi < info.lo_iv[0] + info.len_iv[0]:
                return f"(int)({off})"
            self.note_check(e, f"`{unparse(ix)}` in {info.name}")
            return f"chk_dom({off}, {info.len_cpp}, {e.line})"
        self.error(ix, f"cannot index with {it!r}")

    def x_field(self, e, want):
        oc, ot = self.expr(e.obj)
        f = e.name
        if isinstance(ot, TDom) and f == "elem" and getattr(ot.info, "base", None) is not None:
            return f"([&](int _w) {{ return _w == 0 ? Opt<int>() : Opt<int>(_w - 1); }}({oc}))", TOpt(TDom(ot.info.base))
        if isinstance(ot, TType) and isinstance(ot.t, TDom):
            info = ot.t.info
            if f == "len":
                return f"(ll){info.len_cpp}", TInt(*info.len_iv)
            if f in ("first", "last"):
                self.note_check(e, f"{info.name} is non-empty")
                return (f"chk_dom(0, {info.len_cpp}, {e.line})" if f == "first"
                        else f"chk_dom({info.len_cpp} - 1, {info.len_cpp}, {e.line})"), ot.t
            if f == "bounds":
                return "", TType(TDom(self.bounds_of(info)))
            if f == "bottom" and getattr(info, "base", None) is not None:
                return "0", ot.t
            if f == "all":
                return f"Rng{{0, (ll){info.len_cpp}}}", TRange(ot.t)
            self.error(e, f"{info.name} has no `.{f}`")
        if isinstance(ot, TDom):
            info = ot.info
            if f == "mirror":
                return f"((int)({info.len_cpp}) - 1 - ({oc}))", ot
            if f in ("ord", "idx"):
                return f"(ll)({oc})", TInt(0, None if info.len_iv[1] is None else info.len_iv[1] - 1)
            if f == "rank":
                return f"((ll)({oc}) + 1)", TInt(1, info.len_iv[1])
            if f == "label":
                return self.dom_label(info, oc), TInt(info.lo_iv[0], None if None in (info.lo_iv[1], info.len_iv[1]) else info.lo_iv[1] + info.len_iv[1] - 1)
            if f in ("before", "after"):
                b = self.bounds_of(info)
                return (oc if f == "before" else f"({oc} + 1)"), TDom(b)
            if f in ("next", "prev"):
                d = "+" if f == "next" else "-"
                tmp = self.tmpname("i")
                return (f"([&]{{ int {tmp} = {oc} {d} 1; return {tmp} >= 0 && {tmp} < {info.len_cpp} ? "
                        f"Opt<int>({tmp}) : Opt<int>(); }}())"), TOpt(ot)
            self.error(e, f"{info.name} values have no `.{f}`")
        if isinstance(ot, TTuple):
            i = ot.field(f)
            if i is None:
                self.error(e, f"{ot!r} has no field `{f}`")
            lv = getattr(self, "lvalue", None)
            if self.narrow(ot.types[i]) and not (lv is not None and lv[0] is e):
                return f"(ll){oc}.{self.field_cpp(ot.names[i])}", ot.types[i]
            return f"{oc}.{self.field_cpp(ot.names[i])}", ot.types[i]
        if isinstance(ot, TRange):
            if f in ("start", "end"):
                c = f"{oc}.lo" if f == "start" else f"{oc}.hi"
                el = ot.elem
                return (f"(int)({c})" if isinstance(el, TDom) else c), (TDom(self.bounds_of(el.info)) if isinstance(el, TDom) else INT)
            if f == "len":
                return f"max(0LL, {oc}.hi - {oc}.lo)", TInt(0, None)
            if f in ("first", "last"):
                self.note_check(e, "range is non-empty")
                c = f"cp_rfirst({oc}, {e.line})" if f == "first" else f"cp_rlast({oc}, {e.line})"
                return (f"(int){c}" if isinstance(ot.elem, TDom) else c), ot.elem
        if isinstance(ot, (TList, TStr, TMap, TSet)):
            if f == "len":
                return f"(ll){oc}.size()", TInt(0, None)
        if isinstance(ot, TMod) and f == "value":
            return f"{oc}.v", TInt(0, ot.m - 1)
        self.error(e, f"{ot!r} has no `.{f}`")

    # ---------- calls
    REDUCERS = {"sum", "count", "min", "max", "any", "all", "product"}

    def x_call(self, e, want):
        fn = e.fn
        if fn.kind == "name" and fn.name in self.REDUCERS and not self.lookup(fn.name):
            if len(e.args) == 1:
                return self.reduction(fn.name, e.args[0], e, want)
            if fn.name in ("min", "max") and len(e.args) >= 2:
                cs = [self.expr(a) for a in e.args]
                ts = [t for _, t in cs]
                if all(is_int(t) for t in ts):
                    lo = [t.lo for t in ts]; hi = [t.hi for t in ts]
                    f = min if fn.name == "min" else max
                    rlo = None if None in lo else f(lo)
                    rhi = None if None in hi else f(hi)
                    c = cs[0][0]
                    for x, _ in cs[1:]:
                        c = f"{fn.name}<ll>({c}, {x})"
                    return c, TInt(rlo, rhi)
                if all(t == ts[0] for t in ts) and not isinstance(ts[0], TOpt):
                    c = cs[0][0]
                    for x, _ in cs[1:]:
                        c = f"{fn.name}({c}, {x})"
                    return c, ts[0]
                self.error(e, f"{fn.name} of {ts!r}")
        if fn.kind == "name" and fn.name == "rand" and not self.lookup("rand"):
            (lc, lt), (hc, ht) = self.expr(e.args[0]), self.expr(e.args[1])
            if not (is_int(lt) and is_int(ht)):
                self.error(e, "rand(lo, hi) needs Ints")
            return f"gen_int({lc}, {hc})", TInt(lt.lo, ht.hi)
        if fn.kind == "name" and fn.name == "abs" and not self.lookup("abs"):
            c, t = self.expr(e.args[0])
            if not is_int(t):
                self.error(e, "abs needs Int")
            m = None if None in (t.lo, t.hi) else max(abs(t.lo), abs(t.hi))
            return f"llabs({c})", TInt(0 if m is not None else None, m)
        if fn.kind == "name" and fn.name == "sorted" and not self.lookup("sorted"):
            c, t = self.expr(e.args[0])
            if not isinstance(t, (TList, TArr)) or (isinstance(t, TArr) and len(t.dims) != 1):
                self.error(e, "sorted needs a list or 1-D array")
            et = t.elem
            tmp = self.tmpname("s")
            return f"([&]{{ vector<{self.cpp_type(et)}> {tmp}({c}.begin(), {c}.end()); sort({tmp}.begin(), {tmp}.end()); return {tmp}; }}())", (t if isinstance(t, TArr) else TList(et))
        if fn.kind == "name" and fn.name == "dijkstra" and not self.lookup("dijkstra"):
            gc, gt = self.expr(e.args[0])
            ok = (isinstance(gt, TArr) and len(gt.dims) == 1 and isinstance(gt.elem, TList)
                  and isinstance(gt.elem.elem, TTuple) and len(gt.elem.elem.types) == 2
                  and gt.elem.elem.types[0] == gt.dims[0] and is_int(gt.elem.elem.types[1]))
            if not ok:
                self.error(e, f"dijkstra needs [D]List<(D, Int)>, got {gt!r}")
            sc, st = self.expr(e.args[1], gt.dims[0])
            sc = self.coerce(sc, st, gt.dims[0], e.args[1])
            return f"cp_dijkstra({gc}, {sc})", TArr([gt.dims[0]], TOpt(TInt(0, None)))
        if fn.kind == "generic" and fn.name in ("Map", "Set", "HashMap"):
            ts = [self.resolve_type(a) if a.kind.startswith("t") else self.resolve_type(N("texpr", a.line, expr=a)) for a in fn.args]
            t = TSet(ts[0]) if fn.name == "Set" else TMap(*ts, hashed=fn.name == "HashMap")
            return self.set_ctor(t), t
        if fn.kind == "generic" and fn.name in ("List", "MinHeap"):
            a = fn.args[0]
            et = self.resolve_type(a) if a.kind.startswith("t") else self.resolve_type(N("texpr", a.line, expr=a))
            t = TList(et) if fn.name == "List" else THeap(et)
            return f"{self.cpp_type(t)}()", t
        if fn.kind == "name" and fn.name in ("Int", "Bool"):
            c, t = self.expr(e.args[0])
            if fn.name == "Int":
                if isinstance(t, TBool): return f"(ll)({c})", TInt(0, 1)
                if isinstance(t, TChar): return f"(ll)({c})", TInt(0, 255)
                if isinstance(t, TMod): return f"({c}).v", TInt(0, t.m - 1)
                if is_int(t): return c, t
            self.error(e, f"cannot convert {t!r} to {fn.name}")
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "type" \
                and getattr(getattr(self.lookup(fn.name), "node", None), "kind", None) == "struct":
            sym = self.lookup(fn.name)
            fields = sym.ty.names
            items, names = [], []
            for i, a in enumerate(e.args):
                if a.kind == "kwarg":
                    items.append(a.value); names.append(a.name)
                else:
                    if names and names[-1] not in fields[:i]:
                        pass
                    if i >= len(fields):
                        self.error(a, f"{fn.name} has {len(fields)} fields")
                    items.append(a); names.append(fields[i])
            if sorted(names) != sorted(fields):
                missing = [f for f in fields if f not in names]
                self.error(e, f"{fn.name}(...) is missing {', '.join(missing) or 'nothing'} / has unknown fields")
            order = [names.index(f) for f in fields]
            lit = N("tuple", e.line, items=[items[k] for k in order], names=list(fields))
            return self.x_tuple(lit, sym.ty)
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "ctor":
            return self.construct(self.lookup(fn.name), e.args, e)
        if fn.kind == "name":
            fsym = self.lookup(fn.name)
            fnode = getattr(fsym, "node", None)
            if fsym is not None and fsym.kind in ("fn", "genfn") and getattr(fnode, "params", None) is not None:
                for a, p in zip(e.args, fnode.params):
                    is_mut = getattr(p, "mode", None) == "inout"
                    if is_mut and not getattr(a, "mut", False):
                        self.error(a, f"`{p.name}` is a mut parameter of {fn.name}; write `mut {unparse(a)}` at the call")
                    if getattr(a, "mut", False) and not is_mut:
                        self.error(a, f"`{p.name}` is not a mut parameter of {fn.name}")
                    if is_mut:
                        self.note_write(a, e.line)
        if any(getattr(a, "mut", False) for a in e.args) and not (
                fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) in ("fn", "genfn")):
            self.error(e, "`mut` arguments are only for user fns with mut parameters")
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "genfn":
            return self.call_generic(self.lookup(fn.name).node, e)
        fc, ft = self.expr(fn) if fn.kind != "field" else (None, None)
        if fn.kind == "field":
            if fn.name in ("push", "pop", "sort", "rev", "clear", "insert", "remove", "swap"):
                self.note_write(fn.obj, e.line)
            return self.method_call(e, want)
        if isinstance(ft, TType):
            t = ft.t
            if isinstance(t, TDom) and len(e.args) == 1:
                info = t.info
                c, at = self.expr(e.args[0])
                if isinstance(at, TDom) and at.info is info:
                    return c, t
                if not is_int(at):
                    self.error(e, f"{info.name}(x) needs an Int, got {at!r}")
                self.note_check(e, f"`{unparse(e.args[0])}` in {info.name}")
                return f"chk_dom((ll)({c}) - {info.lo_cpp}, {info.len_cpp}, {e.line})", t
            if isinstance(t, TMod) and len(e.args) == 1:
                c, at = self.expr(e.args[0])
                return f"Mod<{t.m}>({c})", t
            self.error(e, f"cannot call type {t!r}")
        if isinstance(ft, TFn):
            if len(e.args) != len(ft.params):
                self.error(e, f"expected {len(ft.params)} arguments, got {len(e.args)}")
            args = []
            for a, pt in zip(e.args, ft.params):
                c, at = self.expr(a, pt)
                args.append(self.coerce(c, at, pt, a))
            sym = self.lookup(fn.name)
            if getattr(sym, "local", False):
                args = [fc] + args
            return f"{fc}({', '.join(args)})", ft.ret
        self.error(e, f"cannot call {ft!r}")

    def method_call(self, e, want):
        obj, m = e.fn.obj, e.fn.name
        oc, ot = self.expr(obj)
        args = e.args
        tbl = getattr(self, "struct_methods", {})
        key = ot.t.cpp if isinstance(ot, TType) and hasattr(ot.t, "cpp") else getattr(ot, "cpp", None)
        if key in tbl and m in tbl[key]:
            fname, has_self, mut_self = tbl[key][m]
            call_args = list(args)
            if not isinstance(ot, TType):
                if not has_self:
                    self.error(e, f"{m} is a static fn; call it on the type")
                recv = copy.copy(obj)
                if mut_self:
                    recv.mut = True
                call_args = [recv] + call_args
            return self.expr(N("call", e.line, fn=N("name", e.line, name=fname), args=call_args), want)
        if isinstance(ot, TType) and isinstance(ot.t, TDom) and m in ("at", "try_at", "nth", "try_nth"):
            info = ot.t.info
            c, t = self.expr(args[0])
            if m in ("at", "try_at"):
                lo = getattr(info, "lo_cpp", None)
                if lo is None:
                    self.error(e, f"{info.name} is not an integer range; use {info.name}.nth(position)")
                c = f"(({c}) - ({lo}))"
            if m in ("at", "nth"):
                what = "a value" if m == "at" else "a 0-based position"
                self.note_check(e, f"`{unparse(args[0])}` is {what} of {info.name}")
                return f"chk_dom({c}, {info.len_cpp}, {e.line})", ot.t
            return f"([&](ll _k){{ return _k >= 0 && _k < {info.len_cpp} ? Opt<int>((int)_k) : Opt<int>(); }}({c}))", TOpt(ot.t)
        if isinstance(ot, (TMap, TSet)):
            kt = ot.key if isinstance(ot, TMap) else ot.elem
            def key(a):
                c, t = self.expr(a, kt)
                return self.coerce(c, t, kt, a)
            if isinstance(ot, TMap) and m == "get":
                ov = self.cpp_type(TOpt(ot.val))
                return f"([&](auto _it){{ return _it == {oc}.end() ? {ov}() : {ov}(_it->second); }}({oc}.find({key(args[0])})))", TOpt(ot.val)
            if isinstance(ot, TSet) and m == "insert":
                return f"{oc}.insert({key(args[0])}).second", BOOL
            if m == "remove":
                return f"({oc}.erase({key(args[0])}) > 0)", BOOL
            if m == "clear":
                return f"{oc}.clear()", UNIT
            if isinstance(ot, TSet) and m in ("min", "max"):
                ov = self.cpp_type(TOpt(ot.elem))
                end = "begin()" if m == "min" else "rbegin()"
                return f"({oc}.empty() ? {ov}() : {ov}(*{oc}.{end}))", TOpt(ot.elem)
            self.error(e, f"no method {m} on {ot!r}")
        if isinstance(ot, TList):
            if m == "push":
                c, t = self.expr(args[0], ot.elem)
                return f"{oc}.push_back({self.coerce(c, t, ot.elem, args[0])})", UNIT
            if m == "pop":
                self.note_check(e, "pop from a non-empty list")
                return f"cp_pop({oc}, {e.line})", ot.elem
            if m == "last":
                return f"({oc}.empty() ? {self.cpp_type(TOpt(ot.elem))}() : {self.cpp_type(TOpt(ot.elem))}({oc}.back()))", TOpt(ot.elem)
            if m == "sort":
                return f"sort({oc}.begin(), {oc}.end())", UNIT
            if m == "rev":
                return f"reverse({oc}.begin(), {oc}.end())", UNIT
            if m == "clear":
                return f"{oc}.clear()", UNIT
        if isinstance(ot, (TArr, TList)) and m == "sorted_by" and (isinstance(ot, TList) or len(ot.dims) == 1):
            et = ot.elem
            ec = self.cpp_type(et)
            pn, kc, kt = self.lambda_key(args[0], et)
            tmp = self.tmpname("s")
            key = f"[&](const {ec}& {pn}) {{ return {kc}; }}"
            return (f"([&]{{ auto _key = {key}; vector<{ec}> {tmp}({oc}.begin(), {oc}.end()); "
                    f"stable_sort({tmp}.begin(), {tmp}.end(), [&](const {ec}& A, const {ec}& B) {{ return _key(A) < _key(B); }}); "
                    f"return {tmp}; }}())"), TList(et)
        if isinstance(ot, TArr) and len(ot.dims) == 1 and m in ("last_le", "first_ge"):
            c, t = self.expr(args[0])
            if m == "last_le":
                return (f"([&](auto _x){{ ll _k = upper_bound({oc}.begin(), {oc}.end(), _x) - {oc}.begin() - 1; "
                        f"return _k >= 0 ? Opt<int>((int)_k) : Opt<int>(); }}({c}))"), TOpt(ot.dims[0])
            return (f"([&](auto _x){{ ll _k = lower_bound({oc}.begin(), {oc}.end(), _x) - {oc}.begin(); "
                    f"return _k < (ll){oc}.size() ? Opt<int>((int)_k) : Opt<int>(); }}({c}))"), TOpt(ot.dims[0])
        if isinstance(ot, THeap):
            if m == "push":
                c, t = self.expr(args[0], ot.elem)
                return f"{oc}.push({self.coerce(c, t, ot.elem, args[0])})", UNIT
            if m == "pop":
                ot_c = self.cpp_type(TOpt(ot.elem))
                return f"([&]{{ if ({oc}.empty()) return {ot_c}(); auto _v = {oc}.top(); {oc}.pop(); return {ot_c}(_v); }}())", TOpt(ot.elem)
            if m == "len":
                return f"(ll){oc}.size()", TInt(0, None)
        if isinstance(ot, TDom) and m == "offset":
            c, t = self.expr(args[0])
            info = ot.info
            return (f"([&](ll _k){{ return _k >= 0 && _k < {info.len_cpp} ? Opt<int>((int)_k) : Opt<int>(); }}"
                    f"((ll)({oc}) + ({c})))"), TOpt(ot)
        if isinstance(ot, TOpt) and m == "map":
            self.error(e, ".map on ?T is not in the prototype yet")
        self.error(e, f"{ot!r} has no method `{m}`")

    # ---------- iteration
    def bind_pattern(self, pat, c, t, node, mut=False):
        """Declare pattern names for value c of type t (emits C++ declarations)."""
        if pat.kind == "pname":
            if pat.name == "_":
                return
            nm = self.fresh(pat.name)
            self.emit(f"{'auto' if mut else 'const auto&'} {nm} = {c};")
            self.scopes[-1][pat.name] = Sym("var" if mut else "let", t, nm, mut=mut, node=pat)
            return
        if not isinstance(t, TTuple) or len(t.types) != len(pat.items):
            self.error(node, f"cannot destructure {t!r} into {len(pat.items)} names")
        tmp = self.tmpname("d")
        self.emit(f"const auto& {tmp} = {c};")
        for sub, fn, ft in zip(pat.items, t.names, t.types):
            self.bind_pattern(sub, f"{tmp}.{self.field_cpp(fn)}", ft, node)

    def loop(self, pat, it, node, body):
        """Emit a for-loop over iterable node `it`, binding `pat`; body() emits the loop body."""
        if not hasattr(self, "loop_ctx"):
            self.loop_ctx = []
        if it.kind == "call" and it.fn.kind == "name":
            sym = self.lookup(it.fn.name)
            if sym is not None and sym.kind == "fn" and isinstance(sym.ty.ret, TGen):
                self.gen_loop(sym, it, pat, node, body)
                return
        body0 = body
        def body():
            self.loop_ctx.append(("c",))
            body0()
            self.loop_ctx.pop()
        rev = False
        if it.kind == "field" and it.name == "rev":
            rev, it = True, it.obj
        if it.kind == "call" and it.fn.kind == "field" and it.fn.name == "pairs" and not it.args:
            c, t = self.expr(it.fn.obj)
            if not (isinstance(t, TArr) and len(t.dims) == 1) or pat.kind != "ptuple" or len(pat.items) != 2:
                self.error(it, "`for (x, y) in a.pairs()` needs a 1-D array")
            a, i, j = self.tmpname("a"), self.tmpname("i"), self.tmpname("j")
            self.emit(f"{{ const auto& {a} = {c};")
            self.emit(f"for (size_t {i} = 0; {i} < {a}.size(); ++{i}) for (size_t {j} = {i} + 1; {j} < {a}.size(); ++{j}) {{")
            self.scopes.append({})
            self.loop_trips.append(None)
            self.ind += 1
            self.bind_pattern(pat.items[0], f"{a}[{i}]", t.elem, node)
            self.bind_pattern(pat.items[1], f"{a}[{j}]", t.elem, node)
            body()
            self.ind -= 1
            self.loop_trips.pop()
            self.scopes.pop()
            self.emit("} }")
            return
        self.scopes.append({})
        trips = None
        if it.kind == "range":
            lo, lt = self.expr(it.lo)
            hi, ht = self.expr(it.hi)
            hi_x = f"(ll)({hi}) + 1" if it.incl else f"(ll)({hi})"
            v = self.fresh(pat.name if pat.kind == "pname" else "i")
            if isinstance(lt, TDom) and isinstance(ht, TDom) and lt.info is ht.info:
                et, cty = lt, "int"
                trips = lt.info.len_iv[1]
            elif is_int(lt) and is_int(ht):
                hiv = None if ht.hi is None else ht.hi + (1 if it.incl else 0)
                et, cty = TInt(lt.lo, None if hiv is None else hiv - 1), "ll"
                if hiv is not None and lt.lo is not None:
                    trips = max(0, hiv - lt.lo)
            else:
                self.error(it, f"cannot iterate a range of {lt!r}..{ht!r}")
            if rev:
                self.emit(f"for ({cty} {v} = {hi_x} - 1; {v} >= (ll)({lo}); --{v}) {{")
            else:
                self.emit(f"for ({cty} {v} = {lo}; {v} < {hi_x}; ++{v}) {{")
            s = Sym("let", et, v, node=pat)
            if is_int(et):
                s.fact = (lo, hi_x.replace("(ll)", "", 1) if False else (f"({hi}) + 1" if it.incl else hi))
            if pat.kind != "pname":
                self.error(it, "range loops bind a single name")
            self.scopes[-1][pat.name] = s
        else:
            c, t = self.expr(it)
            if isinstance(t, TType) and isinstance(t.t, TDom):
                info = t.t.info
                trips = info.len_iv[1]
                v = self.fresh(pat.name if pat.kind == "pname" else "i")
                if rev:
                    self.emit(f"for (int {v} = {info.len_cpp} - 1; {v} >= 0; --{v}) {{")
                else:
                    self.emit(f"for (int {v} = 0; {v} < {info.len_cpp}; ++{v}) {{")
                self.ind += 1
                self.bind_pattern(pat, v, t.t, node) if pat.kind != "pname" else \
                    self.scopes[-1].__setitem__(pat.name, Sym("let", t.t, v, node=pat))
                self.ind -= 1
            elif isinstance(t, TRange):
                trips = None
                v = self.fresh(pat.name if pat.kind == "pname" else "i")
                cty = "int" if isinstance(t.elem, TDom) else "ll"
                r = self.tmpname("r")
                self.emit(f"{{ Rng {r} = {c};")
                if rev:
                    self.emit(f"for ({cty} {v} = {r}.hi - 1; {v} >= {r}.lo; --{v}) {{")
                else:
                    self.emit(f"for ({cty} {v} = {r}.lo; {v} < {r}.hi; ++{v}) {{")
                self.scopes[-1][pat.name] = Sym("let", t.elem, v, node=pat)
                self.ind += 1
                body()
                self.ind -= 1
                self.emit("} }")
                self.scopes.pop()
                return
            elif isinstance(t, (TMap, TSet)):
                v = self.tmpname("x")
                self.emit(f"for (const auto& {v} : {c}) {{")
                self.ind += 1
                if isinstance(t, TMap):
                    if pat.kind != "ptuple" or len(pat.items) != 2:
                        self.error(it, "iterate a map as `for (k, v) in m`")
                    self.bind_pattern(pat.items[0], f"{v}.first", t.key, node)
                    self.bind_pattern(pat.items[1], f"{v}.second", t.val, node)
                else:
                    self.bind_pattern(pat, v, t.elem, node)
                self.ind -= 1
            elif isinstance(t, (TArr, TList)):
                if isinstance(t, TArr) and len(t.dims) == 1:
                    trips = t.dims[0].info.len_iv[1]
                el = t.elem
                if rev:
                    self.error(it, ".rev on arrays: iterate the domain in reverse instead")
                v = self.tmpname("x")
                self.emit(f"for (const auto& {v} : {c}) {{")
                self.ind += 1
                self.bind_pattern(pat, v, el, node)
                self.ind -= 1
            else:
                self.error(it, f"cannot iterate {t!r}")
        self.loop_trips.append(trips)
        self.ind += 1
        body()
        self.ind -= 1
        self.loop_trips.pop()
        self.emit("}")
        self.scopes.pop()

    def capture(self, f):
        """Run f() with output redirected; return the emitted lines."""
        saved, saved_ind = self.out, self.ind
        self.out, self.ind = [], 1
        try:
            r = f()
        finally:
            lines, self.out, self.ind = self.out, saved, saved_ind
        return lines, r

    def gen_loop(self, sym, it, pat, node, body):
        et = sym.ty.ret.elem
        args = self.call_args(it, sym.ty)
        outer = self.gen_ctx()
        rt = self.fn_ret[-1] if self.fn_ret and len(self.fn_ret) != getattr(self, "sol_depth", -1) else None
        if outer is None:
            hr, rv = self.tmpname("hr"), self.tmpname("rv")
            self.emit(f"bool {hr} = false;")
            if rt is not None and not isinstance(rt, TUnit):
                self.emit(f"{'bool' if isinstance(rt, TGen) else self.cpp_type(rt)} {rv}{{}};")
            ctx = ("g", hr, rv)
        else:
            ctx = outer
        v = self.tmpname("y")
        self.emit(f"{sym.cpp}({', '.join(args + [f'[&]({self.cpp_type(et)} {v}) -> bool {{'])}")
        self.loop_ctx.append(ctx)
        self.scopes.append({})
        self.ind += 1
        self.bind_pattern(pat, v, et, node)
        body()
        self.emit("return true;")
        self.ind -= 1
        self.scopes.pop()
        self.loop_ctx.pop()
        self.emit("});")
        if outer is not None:
            self.emit(f"if ({ctx[1]}) return false;")
        elif rt is not None:
            if isinstance(rt, TUnit):
                self.emit(f"if ({ctx[1]}) return;")
            else:
                self.emit(f"if ({ctx[1]}) return {ctx[2]};")

    def gen_loops(self, clauses, k, inner, node):
        if k == len(clauses):
            inner()
            return
        cl = clauses[k]
        if cl[0] == "for":
            self.loop(cl[1], cl[2], node, lambda: self.gen_loops(clauses, k + 1, inner, node))
        elif cl[0] == "if":
            c, t = self.expr(cl[1])
            if not isinstance(t, TBool):
                self.error(node, f"filter must be Bool, got {t!r}")
            self.emit(f"if ({c}) {{")
            self.ind += 1
            self.gen_loops(clauses, k + 1, inner, node)
            self.ind -= 1
            self.emit("}")
        else:
            c, t = self.expr(cl[2])
            if not isinstance(t, TOpt):
                self.error(node, "`if let` filter needs a ?T")
            o = self.tmpname("o")
            self.emit(f"{{ auto {o} = {c}; if ({o}.has()) {{")
            self.scopes.append({})
            self.ind += 1
            self.bind_pattern(cl[1], f"{o}.get()", t.inner, node)
            self.gen_loops(clauses, k + 1, inner, node)
            self.ind -= 1
            self.scopes.pop()
            self.emit("} }")

    def as_gen(self, arg):
        if arg.kind == "gen":
            return arg.elem, arg.clauses
        x = N("name", arg.line, name="_it")
        return x, [("for", N("pname", arg.line, name="_it"), arg)]

    def trips_of(self, clauses):
        p = 1
        for cl in clauses:
            if cl[0] != "for":
                continue
            it = cl[2]
            t = None
            if it.kind == "field" and it.name == "rev":
                it = it.obj
            if it.kind == "range":
                lo, lt = self.capture(lambda: self.expr(it.lo))[1]
                hi, ht = self.capture(lambda: self.expr(it.hi))[1]
                if isinstance(lt, TDom):
                    t = lt.info.len_iv[1]
                elif lt.lo is not None and ht.hi is not None:
                    t = max(0, ht.hi - lt.lo + (1 if it.incl else 0))
            else:
                _, (c, ty) = self.capture(lambda: self.expr(it))
                if isinstance(ty, TType) and isinstance(ty.t, TDom):
                    t = ty.t.info.len_iv[1]
                elif isinstance(ty, TArr) and len(ty.dims) == 1:
                    t = ty.dims[0].info.len_iv[1]
            if t is None:
                return None
            p *= t
        return p

    def reduction(self, name, arg, e, want):
        elem, clauses = self.as_gen(arg)
        has_filter = any(c[0] != "for" for c in clauses)
        trips = self.trips_of(clauses)
        if trips is not None:
            self.check_cost(e.line, self.__dict__.get("trip_stack", [1])[-1] * trips)
        acc = self.tmpname("a")
        box = {}
        def inner():
            if name == "List":
                et = want.elem if isinstance(want, TList) else None
                c, t = self.expr(elem, et)
                et = et or (TInt() if is_int(t) else t)
                box["t"], box["et"] = t, et
                self.emit(f"{acc}.push_back({self.coerce(c, t, et, elem)});")
                return
            c, t = self.expr(elem, want if name in ("sum", "product") else None)
            box["t"] = t
            if name == "sum":
                self.emit(f"{acc} += {c};")
            elif name == "product":
                self.emit(f"{acc} *= {c};")
            elif name == "count":
                if not isinstance(t, TBool):
                    self.error(e, "count needs Bool elements")
                self.emit(f"{acc} += ({c}) ? 1 : 0;")
            elif name in ("any", "all"):
                if not isinstance(t, TBool):
                    self.error(e, f"{name} needs Bool elements")
                cond, val = (c, "true") if name == "any" else (f"!({c})", "false")
                g = self.gen_ctx()
                if g is not None:
                    self.emit(f"if ({cond}) {{ {g[1]} = true; {g[2]} = {val}; return false; }}")
                else:
                    self.emit(f"if ({cond}) return {val};")
            else:
                if isinstance(t, TOpt):
                    self.error(e, f"{name} over ?T values; filter with `if let`")
                cmp = "<" if name == "min" else ">"
                self.emit(f"{{ auto _v = {c}; if (!{acc}.has() || _v {cmp} {acc}.get()) {acc} = decltype({acc})(_v); }}")
        early = name in ("any", "all")
        if early:
            self.fn_ret.append(BOOL)
            saved_ctx, self.loop_ctx = getattr(self, "loop_ctx", []), []
        try:
            lines, _ = self.capture(lambda: (self.ind.__class__, self.gen_loops(clauses, 0, inner, e)))
        finally:
            if early:
                self.fn_ret.pop()
                self.loop_ctx = saved_ctx
        t = box.get("t")
        if t is None:
            self.error(e, "empty reduction")
        body = "\n".join(lines)
        if name == "List":
            lt = TList(box["et"])
            return f"([&]() {{ {self.cpp_type(lt)} {acc};\n{body}\n return {acc}; }}())", lt
        if name in ("any", "all"):
            return f"([&]() -> bool {{\n{body}\n return {'false' if name == 'any' else 'true'}; }}())", BOOL
        if name == "count":
            return f"([&]() -> ll {{ ll {acc} = 0;\n{body}\n return {acc}; }}())", TInt(0, trips)
        if name in ("sum", "product"):
            rt = t if not is_int(t) else TInt()
            if is_int(t):
                if name == "sum":
                    lo = None if trips is None or t.lo is None else min(0, trips * t.lo)
                    hi = None if trips is None or t.hi is None else max(0, trips * t.hi)
                    rt = self.int_result(e, lo, hi)
                else:
                    rt = TInt()
            init = "0" if name == "sum" else "1"
            return f"([&]() {{ {self.cpp_type(rt)} {acc} = {init};\n{body}\n return {acc}; }}())", rt
        rt = t if not is_int(t) else TInt(t.lo, t.hi)
        nonempty = not has_filter and self.nonempty(clauses)
        ot = TOpt(rt)
        expr = f"([&]() {{ {self.cpp_type(ot)} {acc};\n{body}\n return {acc}; }}())"
        if nonempty:
            return f"{expr}.get()", rt
        return expr, ot

    def nonempty(self, clauses):
        for cl in clauses:
            it = cl[2]
            if it.kind == "field" and it.name == "rev":
                it = it.obj
            if it.kind == "range":
                _, (lc, lt) = self.capture(lambda: self.expr(it.lo))
                _, (hc, ht) = self.capture(lambda: self.expr(it.hi))
                if isinstance(lt, TInt) and lt.hi is not None and ht.lo is not None and \
                   (ht.lo >= lt.hi if it.incl else ht.lo > lt.hi):
                    continue
                return False
            _, (c, t) = self.capture(lambda: self.expr(it))
            if isinstance(t, TType) and isinstance(t.t, TDom) and (t.t.info.len_iv[0] or 0) >= 1:
                continue
            if isinstance(t, TArr) and len(t.dims) == 1 and (t.dims[0].info.len_iv[0] or 0) >= 1:
                continue
            return False
        return True

    def x_gen(self, e, want):
        self.error(e, "a bare generator expression needs a reduction (sum, min, ...) or List(...)")

    def x_table(self, e, want):
        dims = []
        for d in e.dims:
            if d.kind == "range":
                dims.append(self.dim_type(N("trange", d.line, range=d)))
            else:
                c, t = self.expr(d)
                if not (isinstance(t, TType) and isinstance(t.t, TDom)):
                    self.error(d, f"table dimension must be a domain or range, got {t!r}")
                dims.append(t.t)
        pats = [e.pat] if len(dims) == 1 else (e.pat.items if e.pat.kind == "ptuple" else None)
        if pats is None or len(pats) != len(dims):
            self.error(e, "table pattern must bind one name per dimension")
        want_el = want.elem if isinstance(want, TArr) else None
        tab = self.tmpname("tab")
        box = {}
        def body():
            self.scopes.append({})
            vs = []
            for p, d in zip(pats, dims):
                v = self.fresh(p.name if p.name != "_" else "i")
                vs.append(v)
                self.scopes[-1][p.name] = Sym("let", d, v, node=p)
            loops_open = 0
            for v, d in zip(vs, dims):
                self.emit(f"for (int {v} = 0; {v} < {d.info.len_cpp}; ++{v}) {{")
                self.ind += 1
                loops_open += 1
            c, t = self.expr(e.body, want_el)
            if want_el is not None:
                c = self.coerce(c, t, want_el, e.body)
                t = want_el
            box["t"] = TInt() if is_int(t) else t
            flat = vs[0]
            for v, d in zip(vs[1:], dims[1:]):
                flat = f"({flat}) * {d.info.len_cpp} + {v}"
            self.emit(f"{tab}[{flat}] = {c};")
            for _ in range(loops_open):
                self.ind -= 1
                self.emit("}")
            self.scopes.pop()
        lines, _ = self.capture(body)
        et = box["t"]
        size = " * ".join(f"(size_t){d.info.len_cpp}" for d in dims)
        code = (f"([&]() {{ vector<{self.cpp_type(et)}> {tab}({size});\n" + "\n".join(lines) + f"\n return {tab}; }}())")
        return code, TArr(dims, et)

    # ---------- statements
    def block(self, stmts, new_scope=True):
        if new_scope:
            self.scopes.append({})
        for s in stmts:
            self.stmt(s)
        if new_scope:
            self.scopes.pop()

    def stmt(self, s):
        m = getattr(self, "s_" + s.kind, None)
        if m is None:
            self.error(s, f"unsupported statement {s.kind}")
        m(s)

    def declare(self, name, ty, cpp, mut, node, kind=None):
        if isinstance(ty, (TSet, TMap, TArr, TOpt, TList)) and node is not None:
            try:
                self.__dict__.setdefault("reprs", []).append((getattr(node, "line", 0), name, repr(ty), self.cpp_type(ty)))
            except Exception:
                pass
        sym = Sym(kind or ("var" if mut else "let"), ty, cpp, mut=mut, node=node)
        self.scopes[-1][name] = sym
        self.decl_depth[id(node)] = len(self.loop_trips)
        return sym

    def is_read(self, v):
        return v is not None and v.kind == "call" and v.fn.kind == "name" and v.fn.name == "read" \
            and all(a.kind == "kwarg" for a in v.args) and not self.lookup("read")

    def s_let(self, s):
        v = s.value
        if v is not None and v.kind == "doblock" and s.pat.kind == "pname" and s.ty is not None:
            ty = self.resolve_type(s.ty)
            cpp = self.fresh(s.pat.name)
            self.decl_global_or_local(cpp, ty)
            label = self.tmpname("do_end")
            if not hasattr(self, "do_stack"):
                self.do_stack = []
            self.do_stack.append((cpp, ty, label, len(self.fn_ret)))
            self.emit("{")
            self.ind += 1
            self.scopes.append({})
            try:
                self.block(self.do_tail(v.body), new_scope=False)
            finally:
                self.scopes.pop()
                self.ind -= 1
                self.do_stack.pop()
            self.emit("}")
            self.emit(f"{label}:;")
            self.declare(s.pat.name, ty, cpp, s.mut, s)
            return
        if v is not None and v.kind == "call" and v.fn.kind == "name" and v.fn.name == "compress" \
                and s.pat.kind == "ptuple" and len(s.pat.items) == 2:
            c, t = self.expr(v.args[0])
            if not isinstance(t, (TList, TArr)) or not is_int(t.elem):
                self.error(s, "compress needs a list of Int")
            dn, vn = s.pat.items[0].name, s.pat.items[1].name
            vals = self.fresh(vn)
            self.decl_global_or_local(vals, TList(INT))
            self.emit(f"{vals}.assign({c}.begin(), {c}.end()); sort({vals}.begin(), {vals}.end()); "
                      f"{vals}.erase(unique({vals}.begin(), {vals}.end()), {vals}.end());")
            info, _, _ = self.new_domain(dn, "0", f"{vals}.size()", (0, 0), (0, None))
            info.lo_src, info.hi_src = "0", f"{vals}.size()"
            self.emit(f"{info.cpp}_lo = 0; {info.cpp}_len = (ll){vals}.size();")
            self.scopes[-1][dn] = Sym("type", TDom(info), info.cpp, node=s)
            self.declare(vn, TArr([TDom(info)], t.elem), vals, False, s)
            return
        if v is not None and (v.kind == "try" or getattr(s, "orelse", None)):
            inner = v.a if v.kind == "try" else v
            c, t = self.expr(inner)
            if not isinstance(t, TOpt):
                self.error(s, f"`?` / let-else needs a ?T, got {t!r}")
            o = self.tmpname("o")
            self.emit(f"auto {o} = {c};")
            ds = getattr(self, "do_stack", [])
            if v.kind == "try" and ds and ds[-1][3] == len(self.fn_ret):
                dcpp, dty, dlabel, _ = ds[-1]
                if not isinstance(dty, TOpt):
                    self.error(s, "`?` inside `do:` needs the block to have type ?T")
                self.emit(f"if (!{o}.has()) {{ {dcpp} = {{}}; goto {dlabel}; }}")
            elif v.kind == "try":
                if not self.fn_ret or not isinstance(self.fn_ret[-1], TOpt):
                    self.error(s, "`?` is only allowed in a fn returning ?T")
                self.emit(f"if (!{o}.has()) return {{}};")
            else:
                self.emit(f"if (!{o}.has()) {{")
                self.ind += 1
                self.block(s.orelse)
                self.ind -= 1
                self.emit("}")
            self.bind_pattern(s.pat, f"{o}.get()", t.inner, s, mut=s.mut)
            return
        if s.pat.kind != "pname":
            c, t = self.expr(s.value)
            self.bind_pattern(s.pat, c, t, s, mut=s.mut)
            return
        name = s.pat.name
        ty = self.resolve_type(s.ty) if s.ty is not None else None
        if self.is_read(s.value) and len(self.fn_ret) > getattr(self, "sol_depth", 0) and len(self.fn_ret) > (1 if getattr(self, "in_multitest", False) else 0):
            self.reads_in_fns = True
        if self.is_read(s.value):
            if ty is None:
                self.error(s, "read() needs a declared type: `let x: T = read()`")
            cpp = self.fresh(name)
            self.decl_global_or_local(cpp, ty)
            gen = [a for a in getattr(s.value, "args", []) if a.kind == "kwarg" and a.name == "gen"]
            if gen:
                gc, gt = self.expr(N("call", s.line, fn=gen[0].value, args=[]), ty)
                self.emit("if (CP_GEN) {")
                self.ind += 1
                self.emit(f"{cpp} = {self.coerce(gc, gt, ty, s)};")
                self.emit_value(cpp, ty, s)
                self.ind -= 1
                self.emit("} else {")
                self.ind += 1
                self.emit_read(cpp, ty, s)
                self.ind -= 1
                self.emit("}")
            else:
                self.emit_read(cpp, ty, s)
            self.emit("if (CP_GEN) out_char('\\n');")
            if self.out is self.init_lines:
                self.last_read = len(self.init_lines)
            store = TInt() if isinstance(ty, TInt) else ty
            sym = self.declare(name, ty, cpp, s.mut, s)
            if isinstance(ty, TInt) and not s.mut:
                self.var_iv[id(s)] = (ty.lo, ty.hi)
            if getattr(s, "where", None) is not None:
                self.emit_where(s.where, s)
            if self.out is self.init_lines:
                self.last_read = len(self.init_lines)
            self.read_pos = (id(self.out), len(self.out))
            return
        if s.value is None:
            if ty is None:
                self.error(s, "var without a value needs a type")
            c, vt = self.default_value(ty, s), ty
        elif isinstance(ty, TArr) and s.value.kind == "list" and len(s.value.items) == 1 and s.value.items[0].kind == "gen":
            c, vt = self.expr(s.value, ty)
            c, vt = self.coerce(c, vt, ty, s.value), ty
        elif isinstance(ty, TArr) and s.value.kind != "table":
            fc, ft = self.expr(s.value, ty.elem)
            if isinstance(ft, TArr):
                c, vt = self.coerce(fc, ft, ty, s.value), ty
            else:
                fc = self.coerce(fc, ft, ty.elem, s.value)
                size = " * ".join(f"(size_t){d.info.len_cpp}" for d in ty.dims)
                c, vt = f"{self.cpp_type(ty)}({size}, {fc})", ty
        else:
            c, vt = self.expr(s.value, ty)
            if ty is not None:
                c = self.coerce(c, vt, ty, s.value)
        decl_t = ty if ty is not None else vt
        if isinstance(decl_t, TInt) and ty is None and s.mut:
            decl_t = TInt()
        if isinstance(decl_t, TUnit):
            self.error(s, "cannot bind a value of type ()")
        cpp = self.fresh(name)
        if self.at_top and len(self.fn_ret) <= (1 if getattr(self, "in_multitest", False) else 0):
            self.globals.append(f"{self.cpp_type(decl_t)} {cpp};")
            self.emit(f"{cpp} = {c};")
        else:
            self.emit(f"{'' if s.mut else 'const '}{self.cpp_type(decl_t)} {cpp} = {c};")
        self.declare(name, decl_t, cpp, s.mut, s)
        if isinstance(decl_t, TInt):
            if s.mut:
                # A declared type bounds the variable; without one the initializer starts the range.
                src = vt if ty is None else decl_t
                self.contrib.setdefault(id(s), []).append((src.lo, src.hi))
            elif ty is None:
                self.var_iv[id(s)] = (vt.lo, vt.hi)
        if getattr(s, "where", None) is not None:
            self.emit_where(s.where, s)

    def decl_global_or_local(self, cpp, ty):
        if self.at_top and len(self.fn_ret) <= (1 if getattr(self, "in_multitest", False) else 0):
            self.globals.append(f"{self.cpp_type(ty)} {cpp};")
        else:
            self.emit(f"{self.cpp_type(ty)} {cpp};")

    def emit_where(self, cond, s):
        c, t = self.expr(cond)
        if not isinstance(t, BOOL.__class__):
            self.error(s, "`where` needs a Bool")
        self.emit(f"if (CP_GEN && !({c})) exit(4);")
        self.emit(f"CP_ASSERT({c}, \"where-clause failed\", {s.line});")

    def default_value(self, ty, s):
        if isinstance(ty, TArr):
            size = " * ".join(f"(size_t){d.info.len_cpp}" for d in ty.dims)
            return f"{self.cpp_type(ty)}({size})"
        return f"{self.cpp_type(ty)}()"

    def s_domain(self, s):
        if s.range.kind != "range":
            self.error(s, "domain needs a range: `domain V = 1..=n`")
        info = self.domain_from_range(s.range, s.name, s)
        self.scopes[-1][s.name] = Sym("type", TDom(info), info.cpp, node=s)

    def s_typedef(self, s):
        if s.name in self.scopes[-1]:
            self.error(s, f"`{s.name}` is already defined in this scope")
        self.scopes[-1][s.name] = Sym("type", self.resolve_type(s.ty), "", node=s)

    def s_exprstmt(self, s):
        e = s.expr
        if e.kind == "call" and e.fn.kind == "name" and e.fn.name in ("print", "write") and not self.lookup(e.fn.name):
            self.emit_print(e.args, newline=e.fn.name == "print", node=s)
            return
        c, t = self.expr(e)
        self.emit(f"{c};")

    def s_assert(self, s):
        c, t = self.expr(s.cond)
        if not isinstance(t, TBool):
            self.error(s, "assert needs a Bool")
        self.emit(f"CP_ASSERT({c}, \"assertion failed\", {s.line});")

    def in_gen_body(self):
        ctx = getattr(self, "loop_ctx", [])
        return ctx and ctx[-1][0] == "g"

    def gen_ctx(self):
        for c in reversed(getattr(self, "loop_ctx", [])):
            if c[0] == "g":
                return c
        return None

    def s_break(self, s): self.emit("return false;" if self.in_gen_body() else "break;")
    def s_continue(self, s): self.emit("return true;" if self.in_gen_body() else "continue;")

    def gen_stop(self):
        g = self.gen_ctx()
        if g is None:
            return "return false;"
        return f"{{ {g[1]} = true; {g[2]} = false; return false; }}"

    def s_yield(self, s):
        if not self.fn_ret or not isinstance(self.fn_ret[-1], TGen):
            self.error(s, "yield outside a generator (`-> yields T`)")
        et = self.fn_ret[-1].elem
        c, t = self.expr(s.value, et)
        self.emit(f"if (!_cb({self.coerce(c, t, et, s.value)})) {self.gen_stop()}")

    def s_yieldfrom(self, s):
        if not self.fn_ret or not isinstance(self.fn_ret[-1], TGen):
            self.error(s, "yield from outside a generator")
        e = s.value
        sym = self.lookup(e.fn.name) if e.kind == "call" and e.fn.kind == "name" else None
        if sym is None or sym.kind != "fn" or not isinstance(sym.ty.ret, TGen):
            self.error(s, "yield from needs a generator call")
        args = self.call_args(e, sym.ty)
        self.emit(f"if (!{sym.cpp}({', '.join(args + ['_cb'])})) {self.gen_stop()}")

    def call_args(self, e, fty):
        if len(e.args) != len(fty.params):
            self.error(e, f"expected {len(fty.params)} arguments, got {len(e.args)}")
        out = []
        for a, pt in zip(e.args, fty.params):
            c, at = self.expr(a, pt)
            out.append(self.coerce(c, at, pt, a))
        return out

    def s_return(self, s):
        if not self.fn_ret:
            self.error(s, "return outside a function")
        if getattr(self, "in_do", 0) and not getattr(s, "synthetic", False):
            self.error(s, "`return` inside `do:` (the block's value is its last expression)")
        rt = self.fn_ret[-1]
        g = self.gen_ctx()
        if g is not None:
            if isinstance(rt, TGen):
                self.emit(f"{{ {g[1]} = true; {g[2]} = true; return false; }}")
            elif s.value is None:
                self.emit(f"{{ {g[1]} = true; return false; }}")
            else:
                c, t = self.expr(s.value, rt)
                self.emit(f"{{ {g[2]} = {self.coerce(c, t, rt, s.value)}; {g[1]} = true; return false; }}")
            return
        if isinstance(rt, TGen):
            self.emit("return true;")
            return
        if s.value is None:
            self.emit("return;")
            return
        c, t = self.expr(s.value, rt)
        if rt is not None:
            c = self.coerce(c, t, rt, s.value)
        self.emit(f"return {c};")

    def cond(self, e):
        c, t = self.expr(e)
        if not isinstance(t, TBool):
            self.error(e, f"condition must be Bool, got {t!r}")
        return c

    def refine_facts(self, e):
        """`x in V` / `lo <= x < hi` conditions: return list of (name, new Sym)."""
        out = []
        terms = []
        def split(x):
            if x.kind == "binop" and x.op == "and":
                split(x.a); split(x.b)
            else:
                terms.append(x)
        split(e)
        for t in terms:
            if t.kind == "cmp" and t.ops == ["in"] and t.first.kind == "name":
                s = self.lookup(t.first.name)
                _, rt = self.capture(lambda: self.expr(t.rest[0]))[1]
                if s and isinstance(s.ty, TInt) and isinstance(rt, TType) and isinstance(rt.t, TDom):
                    info = rt.t.info
                    out.append((t.first.name, Sym("let", rt.t, f"(int)((ll)({s.cpp}) - {info.lo_cpp})", node=None)))
        return out

    def s_if(self, s):
        if not s.orelse and s.body and s.body[-1].kind in ("return", "break", "continue"):
            cnd = s.cond
            if cnd.kind == "cmp" and cnd.ops == ["=="] and cnd.first.kind == "name" and \
               cnd.rest[0].kind == "num" and cnd.rest[0].value == 0:
                sym = self.lookup(cnd.first.name)
                if sym is not None and not sym.mut:
                    self.after_if_nonzero = sym
        c = self.cond(s.cond)
        self.emit(f"if ({c}) {{")
        self.ind += 1
        self.scopes.append(dict(self.refine_facts(s.cond)))
        self.block(s.body, new_scope=False)
        self.scopes.pop()
        self.ind -= 1
        if s.orelse:
            self.emit("} else {")
            self.ind += 1
            self.block(s.orelse)
            self.ind -= 1
        self.emit("}")
        sym = getattr(self, "after_if_nonzero", None)
        if sym is not None:
            sym.nonzero = True
            self.after_if_nonzero = None

    def s_iflet(self, s):
        c, t = self.expr(s.value)
        if not isinstance(t, TOpt):
            self.error(s, f"`if let` needs a ?T, got {t!r}")
        o = self.tmpname("o")
        self.emit(f"{{ auto {o} = {c};")
        self.emit(f"if ({o}.has()) {{")
        self.ind += 1
        self.scopes.append({})
        self.bind_pattern(s.pat, f"{o}.get()", t.inner, s)
        if s.extra is not None:
            ec = self.cond(s.extra)
            self.emit(f"if ({ec}) {{")
            self.ind += 1
            self.block(s.body)
            self.ind -= 1
            self.emit("} else goto " + o + "_else;" if s.orelse else "}")
        else:
            self.block(s.body)
        self.scopes.pop()
        self.ind -= 1
        if s.orelse:
            self.emit("} else {")
            if s.extra is not None:
                self.emit(f"{o}_else:;")
            self.ind += 1
            self.block(s.orelse)
            self.ind -= 1
        self.emit("} }")

    def s_for(self, s):
        stack = self.__dict__.setdefault("trip_stack", [1])
        try:
            t = self.trips_of([("for", s.pat, s.iter)])
        except CompileError:
            t = None
        stack.append(stack[-1] * (t if t is not None else 1))
        self.check_cost(s.line, stack[-1])
        try:
            self.loop(s.pat, s.iter, s, lambda: self.block(s.body))
        finally:
            stack.pop()

    def check_cost(self, line, n):
        """Warn when a loop nest can run more than ~1e9 times at the largest declared input."""
        warned = self.__dict__.setdefault("cost_warnings", {})
        if n > 10**9 and line not in warned:
            warned[line] = n

    def construct(self, sym, args, e):
        et, k = sym.ty, sym.cpp
        vn, fns, fts = et.variants[k]
        if len(args) != len(fns):
            self.error(e, f"{vn} takes {len(fns)} fields")
        tmp = self.tmpname("e")
        parts = [f"{et.cpp} {tmp}{{}}; {tmp}.tag = {k};"]
        for a, fnm, ft in zip(args, fns, fts):
            c, t = self.expr(a, ft)
            parts.append(f"{tmp}.{vn}_{fnm} = {self.coerce(c, t, ft, a)};")
        return f"([&]{{ {' '.join(parts)} return {tmp}; }}())", et

    def s_struct(self, s):
        names = [f for f, _ in s.fields]
        types = [self.resolve_type(t) for _, t in s.fields]
        t = self.tuple_type(names, types)
        self.scopes[-1][s.name] = Sym("type", t, t.cpp, node=s)
        if not s.methods:
            return
        from .parser import Parser
        table = self.__dict__.setdefault("struct_methods", {}).setdefault(t.cpp, {})
        for m in s.methods:
            ps = list(m.params)
            has_self = bool(ps) and ps[0].name == "self"
            if has_self:
                ps[0] = N("param", m.line, name="self", ty=Parser(s.name).type_(), mode=ps[0].mode)
            fn = N("fn", m.line, name=f"{s.name}__{m.name}", tparams=m.tparams, params=ps, ret=m.ret,
                   body=m.body, decos=m.decos, pure=m.pure)
            if has_self:
                fn.self_struct = t
            table[m.name] = (fn.name, has_self, has_self and ps[0].mode == "inout")
            self.s_fn(fn)

    def lambda_key(self, lam, et):
        if lam.kind != "lambda" or len(lam.params) != 1:
            self.error(lam, "expected a one-parameter lambda `|x| ...`")
        self.scopes.append({})
        pn = self.fresh("x")
        self.bind_lambda_param(lam.params[0], pn, et, lam)
        c, t = self.expr(lam.body)
        self.scopes.pop()
        return pn, c, t

    def bind_lambda_param(self, pat, cpp, et, node):
        if pat.kind == "pname":
            self.scopes[-1][pat.name] = Sym("let", et, cpp, node=node)
        else:
            self.error(node, "lambda parameter must be a name (prototype)")

    def s_enum(self, s):
        cpp = f"E_{s.name}"
        et = TEnum(s.name, cpp)
        for d in getattr(s, "decos", []):
            if d.name == "tags":
                if len(d.args) != len(s.variants) or any(a.kind != "str" for a in d.args):
                    self.error(s, "@tags needs one string per variant")
                et.tags = [a.value for a in d.args]
        self.scopes[-1][s.name] = Sym("type", et, cpp, node=s)
        fields = []
        for k, (vn, fl) in enumerate(s.variants):
            fns = [f for f, _ in fl]
            fts = [self.resolve_type(t) for _, t in fl]
            et.variants.append((vn, fns, fts))
            for f, t in zip(fns, fts):
                fields.append(f"{self.cpp_type(t)} {vn}_{f}{{}};")
            self.scopes[-1][vn] = Sym("ctor", et, k, node=s)
        trivial = all(isinstance(t, (TInt, TDom, TBool, TChar)) for _, _, fts in et.variants for t in fts)
        if trivial and sum(1 for _, fns, _ in et.variants if fns) >= 2:
            # variants share storage; anonymous structs in a union are a GNU extension every judge's g++ accepts
            arms = " ".join(f"struct {{ {' '.join(f'{self.cpp_type(t)} {vn}_{f};' for f, t in zip(fns, fts))} }};"
                            for vn, fns, fts in et.variants if fns)
            cases = " : ".join(f"tag == {k} ? ({' && '.join(f'{vn}_{f} == o.{vn}_{f}' for f in fns)})"
                               for k, (vn, fns, _) in enumerate(et.variants) if fns)
            self.struct_defs.append(f"struct {cpp} {{ int tag = 0; union {{ {arms} }};\n"
                                    f"    {cpp}() {{ memset((void*)this, 0, sizeof *this); }}\n"
                                    f"    bool operator==(const {cpp}& o) const {{ return tag == o.tag && ({cases} : true); }}\n"
                                    f"    bool operator!=(const {cpp}& o) const {{ return !(*this == o); }} }};")
            self.struct_defs.append(f"template <> struct Opt<{cpp}> {{ {cpp} v; Opt() {{ v.tag = -1; }} Opt(const {cpp}& x) : v(x) {{}}\n"
                                    f"    bool has() const {{ return v.tag >= 0; }} const {cpp}& get() const {{ return v; }}\n"
                                    f"    bool operator==(const Opt& o) const {{ return has() == o.has() && (!has() || v == o.v); }}\n"
                                    f"    bool operator!=(const Opt& o) const {{ return !(*this == o); }} }};")
            return
        eqs = " && ".join(["tag == o.tag"] + [f"{vn}_{f} == o.{vn}_{f}" for vn, fns, _ in et.variants for f in fns])
        self.struct_defs.append(f"struct {cpp} {{ int tag = 0; {' '.join(fields)}\n"
                                f"    bool operator==(const {cpp}& o) const {{ return {eqs}; }}\n"
                                f"    bool operator!=(const {cpp}& o) const {{ return !(*this == o); }} }};")
        self.struct_defs.append(f"template <> struct Opt<{cpp}> {{ {cpp} v; Opt() {{ v.tag = -1; }} Opt(const {cpp}& x) : v(x) {{}}\n"
                                f"    bool has() const {{ return v.tag >= 0; }} const {cpp}& get() const {{ return v; }}\n"
                                f"    bool operator==(const Opt& o) const {{ return has() == o.has() && (!has() || v == o.v); }}\n"
                                f"    bool operator!=(const Opt& o) const {{ return !(*this == o); }} }};")

    def s_match(self, s):
        c, t = self.expr(s.subject)
        if not isinstance(t, TEnum):
            self.error(s, f"match needs an enum value, got {t!r}")
        m = self.tmpname("m")
        done = f"{m}_done"
        self.emit(f"{{ const auto& {m} = {c};")
        covered, wildcard = set(), False
        for arm in s.arms:
            self.scopes.append({})
            if arm.name == "_":
                wildcard = True
                self.emit("{")
            else:
                k, v = t.variant(arm.name)
                if k is None:
                    self.error(arm, f"{t.name} has no variant {arm.name}")
                vn, fns, fts = v
                if arm.binds and len(arm.binds) != len(fns):
                    self.error(arm, f"{vn} has {len(fns)} fields")
                if arm.guard is None:
                    covered.add(k)
                self.emit(f"if ({m}.tag == {k}) {{")
                self.ind += 1
                for b, fnm, ft in zip(arm.binds, fns, fts):
                    if b != "_":
                        bc = self.fresh(b)
                        self.emit(f"const auto& {bc} = {m}.{vn}_{fnm};")
                        self.scopes[-1][b] = Sym("let", ft, bc, node=arm)
                self.ind -= 1
            self.ind += 1
            if arm.guard is not None:
                self.emit(f"if ({self.cond(arm.guard)}) {{")
                self.ind += 1
            self.block(arm.body)
            self.emit(f"goto {done};")
            if arm.guard is not None:
                self.ind -= 1
                self.emit("}")
            self.ind -= 1
            self.emit("}")
            self.scopes.pop()
        missing = [v[0] for k, v in enumerate(t.variants) if k not in covered]
        if missing and not wildcard:
            self.error(s, f"match on {t.name} misses {', '.join(missing)}")
        self.emit(f"{done}:; }}")

    def s_whilelet(self, s):
        self.emit("while (true) {")
        self.loop_trips.append(None)
        self.ind += 1
        c, t = self.expr(s.value)
        if not isinstance(t, TOpt):
            self.error(s, f"`while let` needs a ?T, got {t!r}")
        o = self.tmpname("o")
        self.emit(f"auto {o} = {c};")
        self.emit(f"if (!{o}.has()) break;")
        self.scopes.append({})
        self.bind_pattern(s.pat, f"{o}.get()", t.inner, s)
        self.block(s.body)
        self.scopes.pop()
        self.ind -= 1
        self.loop_trips.pop()
        self.emit("}")

    def s_while(self, s):
        c = self.cond(s.cond)
        self.emit(f"while ({c}) {{")
        self.loop_trips.append(None)
        self.ind += 1
        if not hasattr(self, "loop_ctx"):
            self.loop_ctx = []
        self.loop_ctx.append(("c",))
        self.block(s.body)
        self.loop_ctx.pop()
        self.ind -= 1
        self.loop_trips.pop()
        self.emit("}")

    def s_assign(self, s):
        tgt, op = s.target, s.op
        self.lvalue = (tgt, op)
        tc, tt = self.expr(tgt)
        self.lvalue = None
        self.note_write(tgt, s.line)
        if tgt.kind == "name":
            sym = self.lookup(tgt.name)
            if sym is None or not sym.mut:
                self.error(s, f"`{tgt.name}` is not a var")
        else:
            root = tgt
            while root.kind in ("index", "field"):
                root = root.obj
            if root.kind == "name":
                sym = self.lookup(root.name)
                if sym is None or not sym.mut:
                    self.error(s, f"`{root.name}` is not a var")
            sym = None
        if op in ("min=", "max=") and isinstance(tt, TOpt):
            vc, vt = self.expr(s.value, tt.inner)
            self.emit(f"opt_{op[:3]}({tc}, {self.coerce(vc, vt, tt.inner, s.value)});")
            return
        want = tt.inner if isinstance(tt, TOpt) and op == "=" else tt
        vc, vt = self.expr(s.value, want if not isinstance(tt, TInt) else None)
        if op == "=":
            c = self.coerce(vc, vt, tt, s.value)
            self.emit(f"{tc} = {c};")
            self.track(sym, tgt, vt, "=", s)
            return
        if op in ("min=", "max="):
            if isinstance(tt, TInt) and isinstance(vt, TInt):
                self.emit(f"{tc} = {op[:3]}<ll>({tc}, {vc});")
                self.track(sym, tgt, vt, "=", s)
                return
            self.emit(f"{tc} = {op[:3]}({tc}, {self.coerce(vc, vt, tt, s.value)});")
            return
        bop = op[:-1]
        if isinstance(tt, TMod):
            self.emit(f"{tc} {op} {self.coerce(vc, vt, tt, s.value)};")
            return
        if isinstance(tt, TDom):
            self.error(s, f"`{op}` on a domain value; use `i = i + k` (checked)")
        if not (isinstance(tt, TInt) and isinstance(vt, TInt)):
            self.error(s, f"`{op}` between {tt!r} and {vt!r}")
        if bop in ("+", "-"):
            self.track(sym, tgt, vt if bop == "+" else TInt(*iv_neg(vt)), "+=", s)
        elif bop == "*":
            if tt.lo is not None:
                self.int_result(s, *iv_mul(tt, vt))
            self.track(sym, tgt, None, "*=", s)
        self.emit(f"{tc} {op} {vc};")

    def track(self, sym, tgt, vt, kind, s):
        """Record a value-range contribution for an Int var (scalar or array element)."""
        node = sym.node if sym is not None else None
        if node is None and tgt.kind == "index":
            root = tgt
            while root.kind == "index":
                root = root.obj
            if root.kind == "name":
                rs = self.lookup(root.name)
                node = rs.node if rs is not None else None
        if node is None:
            return
        lst = self.contrib.setdefault(id(node), [])
        self.contrib_nodes[id(node)] = (node, s)
        if not isinstance(vt, TInt) and vt is not None:
            return
        if vt is None or vt.lo is None or vt.hi is None:
            lst.append((None, None))
            return
        if kind == "=":
            lst.append((vt.lo, vt.hi))
            return
        depth = self.decl_depth.get(id(node), 0)
        trips = 1
        for t in self.loop_trips[depth:]:
            if t is None:
                lst.append((None, None))
                return
            trips *= t
        lst.append(("acc", min(0, vt.lo) * trips, max(0, vt.hi) * trips))

    def finish_ranges(self):
        """Turn contributions into next-pass var ranges."""
        nxt = {}
        for nid, lst in self.contrib.items():
            lo = hi = 0
            base_lo = base_hi = None
            acc_lo = acc_hi = 0
            ok = True
            for c in lst:
                if c[0] == "acc":
                    acc_lo += c[1]; acc_hi += c[2]
                elif c[0] is None or c[1] is None:
                    ok = False
                else:
                    base_lo = c[0] if base_lo is None else min(base_lo, c[0])
                    base_hi = c[1] if base_hi is None else max(base_hi, c[1])
            if not ok:
                nxt[nid] = (None, None)
                continue
            base_lo = base_lo or 0
            base_hi = base_hi or 0
            lo, hi = base_lo + acc_lo, base_hi + acc_hi
            if not fits(lo, hi) and nid in self.contrib_nodes:
                decl, at = self.contrib_nodes[nid]
                nm = decl.pat.name if decl.kind == "let" else getattr(decl, "name", "?")
                raise CompileError(f"line {at.line}: `{nm}` (declared line {decl.line}) may reach "
                                   f"{hi if abs(hi) >= abs(lo) else lo:.3g} by accumulating `{unparse(at.target)} {at.op} "
                                   f"{unparse(at.value)}` over loops; beyond Int — use I128, Mod or tighter bounds")
            nxt[nid] = (lo, hi)
        return nxt

    # ---------- functions
    def implicit_tparams(self, s):
        """An undeclared single capital letter in a parameter type is a type parameter."""
        found = []
        def walk(x):
            if isinstance(x, N):
                nm = getattr(x, "name", None)
                if (isinstance(nm, str) and len(nm) == 1 and nm.isupper()
                        and self.lookup(nm) is None and nm not in found):
                    found.append(nm)
                for v in x.__dict__.values():
                    walk(v)
            elif isinstance(x, list):
                for y in x:
                    walk(y)
        for p in s.params:
            walk(p.ty)
        return found

    def s_fn(self, s):
        decos = {d.name: d for d in s.decos}
        if not s.tparams:
            s.tparams = self.implicit_tparams(s)
        if s.tparams:
            if not self.at_top or self.fn_ret:
                self.error(s, "generic fns must be top-level (prototype)")
            self.scopes[-1][s.name] = Sym("genfn", None, s.name, node=s)
            return
        if "solution" in decos or ("brute" in decos and not s.params) or "multitest" in decos:
            self.solution_fn(s, decos)
            return
        params, ptypes, names = [], [], []
        self.scopes.append({})
        saved_top = self.at_top
        self.at_top = False
        for p in s.params:
            if p.ty is None:
                self.error(p, f"parameter `{p.name}` needs a type")
            pt = self.resolve_type(p.ty)
            ptypes.append(pt)
        rt = self.resolve_type(s.ret) if s.ret is not None else UNIT
        local = len(self.fn_ret) > (1 if getattr(self, "in_multitest", False) else 0) or not saved_top
        pre = getattr(self, "predeclared", {}).get(s.name)
        cpp = pre if pre and not local else self.fresh(s.name)
        if not local:
            self.predeclare_later(s, saved_top)
        fty = TFn(ptypes, rt)
        outer = self.scopes[-2]
        sym = Sym("fn", fty, cpp, node=s)
        if "brute" in decos:
            tgt = decos["brute"].args[0] if decos["brute"].args else None
            fast = self.lookup(tgt.name) if tgt is not None and tgt.kind == "name" else None
            if fast is None or fast.kind != "fn":
                self.error(s, "@brute(f) on a fn with parameters needs f to be a fn defined above")
            if fast.ty.params != ptypes or fast.ty.ret != rt:
                self.error(s, f"@brute({tgt.name}): `{s.name}` must have the same parameter and return types")
            if any(getattr(p, "mode", None) == "inout" for p in s.params):
                self.error(s, "@brute on fns with inout parameters is not supported (prototype)")
            self.fn_brutes.append((fast, sym, s))
        if "invariant" in decos:
            for a in decos["invariant"].args:
                kind = getattr(a, "name", None)
                if kind not in ("rev", "shuffle"):
                    self.error(s, "@invariant(...) takes `rev` or `shuffle` (prototype)")
                if not any(isinstance(t, (TArr, TList)) for t in ptypes):
                    self.error(s, f"@invariant({kind}) needs an array or list parameter")
                self.fn_props.append((sym, kind, s))
        sym.local = local
        outer[s.name] = sym
        if s.pure:
            self.check_pure(s)
        for p, pt in zip(s.params, ptypes):
            pc = self.fresh(p.name)
            mode = "&" if p.mode == "inout" else ("" if isinstance(pt, (TInt, TBool, TChar, TDom, TMod)) else "const &")
            params.append(f"{self.cpp_type(pt)} {mode}{pc}".replace("  ", " "))
            self.declare(p.name, pt, pc, p.mode == "inout", p)
        st = getattr(s, "self_struct", None)
        if st is not None:
            me = self.lookup("self")
            for fname, ftype in zip(st.names, st.types):
                self.declare(fname, ftype, f"{me.cpp}.{self.field_cpp(fname)}", me.mut, None)
        self.fn_ret.append(rt)
        saved_ctx, self.loop_ctx = getattr(self, "loop_ctx", []), []
        try:
            self.fn_rest(s, decos, params, ptypes, rt, cpp, local, saved_top)
        finally:
            self.loop_ctx = saved_ctx

    def fn_rest(self, s, decos, params, ptypes, rt, cpp, local, saved_top):
        if isinstance(rt, TGen):
            if local:
                self.error(s, "generators must be top-level fns (prototype)")
            lines, _ = self.capture(lambda: self.block(s.body, new_scope=False))
            self.fn_ret.pop()
            self.scopes.pop()
            self.at_top = saved_top
            ps = ", ".join(params + ["F&& _cb"])
            self.funcs.append(f"template <class F> bool {cpp}({ps}) {{\n" + "\n".join(lines) + "\n    return true;\n}")
            return
        memo = "memo" in decos
        if memo:
            hashed = any(a.kind == "name" and a.name == "hash" for a in decos["memo"].args) or not all(isinstance(t, TDom) for t in ptypes)
            if local or not all(isinstance(t, (TDom, TInt, TBool)) for t in ptypes) or isinstance(rt, TUnit):
                self.error(s, "@memo (prototype) needs a fn at top level or directly in a @multitest fn, with domain parameters")
            self.check_memo_reads(s)
            tab = f"_memo_{cpp}"
            if hashed:
                if any(a.kind == "kwarg" and a.name in ("into", "order") for a in decos["memo"].args):
                    self.error(s, "@memo(hash) does not support into/order yet (prototype)")
                k = len(ptypes)
                self.globals.append(f"unordered_map<array<ll, {k}>, {self.cpp_type(rt)}, CpArrHash> {tab};")
                pnames = [self.lookup(p.name).cpp for p in s.params]
                inner = self.fresh(s.name + "_body")
                body_lines, _ = self.capture(lambda: self.fn_body(s.body, rt))
                self.fn_ret.pop()
                self.scopes.pop()
                self.at_top = saved_top
                rcpp = self.cpp_type(rt)
                self.funcs.append(f"{rcpp} {inner}({', '.join(params)}) {{\n" + "\n".join(body_lines) + "\n}")
                key = ", ".join(f"(ll){pn}" for pn in pnames)
                self.funcs.append(
                    f"{rcpp} {cpp}({', '.join(params)}) {{\n"
                    f"    array<ll, {k}> _key{{{key}}};\n"
                    f"    auto _it = {tab}.find(_key);\n"
                    f"    if (_it != {tab}.end()) return _it->second;\n"
                    f"    {rcpp} _r = {inner}({', '.join(pnames)});\n"
                    f"    {tab}.emplace(_key, _r);\n"
                    f"    return _r;\n}}")
                self.funcs.insert(0, f"{rcpp} {cpp}({', '.join(params)});")
                self.emit(f"{tab}.clear();")
                return
            self.globals.append(f"vector<{self.cpp_type(TOpt(rt))}> {tab};")
            size = " * ".join(f"(size_t){t.info.len_cpp}" for t in ptypes)
            pnames = [self.lookup(p.name).cpp for p in s.params]
            flat = pnames[0]
            for pn, t in zip(pnames[1:], ptypes[1:]):
                flat = f"({flat}) * {t.info.len_cpp} + {pn}"
            inner = self.fresh(s.name + "_body")
            body_lines, _ = self.capture(lambda: self.fn_body(s.body, rt))
            self.fn_ret.pop()
            self.scopes.pop()
            self.at_top = saved_top
            rcpp = self.cpp_type(rt)
            self.funcs.append(f"{rcpp} {inner}({', '.join(params)}) {{\n" + "\n".join(body_lines) + "\n}")
            order = next((a.value.name for a in decos["memo"].args if a.kind == "kwarg" and a.name == "order"), "recursive")
            if order not in ("recursive", "ascending"):
                self.error(s, "@memo(order: ...) is `recursive` or `ascending`")
            fill = ""
            if order == "ascending":
                self.globals.append(f"bool {tab}_filled = false;")
                idx = [f"_k{k}" for k in range(len(ptypes))]
                loops = "".join(f"for (int {v} = 0; {v} < {t.info.len_cpp}; ++{v}) " for v, t in zip(idx, ptypes))
                fill = (f"    if (!{tab}_filled) {{ {tab}_filled = true; if ({tab}.size() != {size}) {tab}.assign({size}, {{}}); "
                        f"{loops}{cpp}({', '.join(idx)}); }}\n")
            self.funcs.append(
                f"{rcpp} {cpp}({', '.join(params)}) {{\n" + fill +
                f"    if ({tab}.size() != {size}) {tab}.assign({size}, {{}});\n"
                f"    auto& _m = {tab}[{flat}];\n"
                f"    if (!_m.has()) _m = {inner}({', '.join(pnames)});\n"
                f"    return _m.get();\n}}")
            self.funcs.insert(0, f"{rcpp} {cpp}({', '.join(params)});")
            into = next((a.value for a in decos["memo"].args if a.kind == "kwarg" and a.name == "into"), None)
            if order == "ascending":
                self.emit(f"{tab}_filled = false;")
            if into is not None:
                if into.kind != "name":
                    self.error(s, "@memo(into: name) needs a plain name")
                self.emit(f"{tab}.assign({size}, {{}});")
                self.declare(into.name, TArr(list(ptypes), TOpt(rt)), tab, True, s)
            else:
                self.emit(f"{tab}.clear();")
            return
        lines, _ = self.capture(lambda: self.fn_body(s.body, rt))
        self.fn_ret.pop()
        self.scopes.pop()
        self.at_top = saved_top
        rcpp = self.cpp_type(rt)
        if local:
            self.emit(f"auto {cpp} = [&](auto&& {cpp}, {', '.join(params)}) -> {rcpp} {{".replace(", )", ")"))
            for l in lines:
                self.emit(l.strip() if False else l[4:] if l.startswith("    ") else l)
            self.emit("};")
        else:
            self.funcs.append(f"{rcpp} {cpp}({', '.join(params)}) {{\n" + "\n".join(lines) + "\n}")

    def predeclare_later(self, s, saved_top):
        """Declare later top-level fns whose types already resolve, so fns can recurse mutually."""
        if not hasattr(self, "predeclared"):
            self.predeclared = {}
        body = self.prog.body
        if s not in body:
            return
        for t in body[body.index(s) + 1:]:
            if t.kind != "fn" or t.name in self.predeclared or self.lookup(t.name):
                continue
            if ({d.name for d in t.decos} & {"solution", "multitest"}) or ("brute" in {d.name for d in t.decos} and not t.params):
                continue
            try:
                pts = [self.resolve_type(p.ty) for p in t.params]
                rt = self.resolve_type(t.ret) if t.ret is not None else UNIT
            except CompileError:
                continue
            cpp = self.fresh(t.name)
            self.predeclared[t.name] = cpp
            sym = Sym("fn", TFn(pts, rt), cpp, node=t)
            sym.local = False
            self.scopes[0][t.name] = sym
            ps = []
            for p, pt in zip(t.params, pts):
                mode = "&" if p.mode == "inout" else ("" if isinstance(pt, (TInt, TBool, TChar, TDom, TMod)) else "const &")
                ps.append(f"{self.cpp_type(pt)} {mode}x{len(ps)}".replace("  ", " "))
            if isinstance(rt, TGen):
                self.funcs.insert(0, f"template <class F> bool {cpp}({', '.join(ps + ['F&& _cb'])});")
            else:
                self.funcs.insert(0, f"{self.cpp_type(rt)} {cpp}({', '.join(ps)});")

    def do_tail(self, body):
        if not body:
            return body
        last = body[-1]
        if last.kind == "exprstmt":
            return body[:-1] + [N("doassign", last.line, value=last.expr)]
        if last.kind == "if" and last.orelse:
            return body[:-1] + [N("if", last.line, cond=last.cond, body=self.do_tail(last.body),
                                  orelse=self.do_tail(last.orelse))]
        self.error(last, "a `do:` block must end with an expression")

    def s_doassign(self, s):
        cpp, ty, label, _ = self.do_stack[-1]
        c, t = self.expr(s.value, ty)
        self.emit(f"{cpp} = {self.coerce(c, t, ty, s.value)};")
        self.emit(f"goto {label};")

    def tail_returns(self, body):
        """Rewrite a block's tail expression (also inside a final if/else) into `return`."""
        if not body:
            return body
        last = body[-1]
        if last.kind == "exprstmt":
            return body[:-1] + [N("return", last.line, value=last.expr, synthetic=True)]
        if last.kind == "if" and last.orelse:
            return body[:-1] + [N("if", last.line, cond=last.cond, body=self.tail_returns(last.body),
                                  orelse=self.tail_returns(last.orelse))]
        return body

    def fn_body(self, body, rt):
        if body and not isinstance(rt, (TUnit, TGen)) and body[-1].kind == "if" and body[-1].orelse:
            body = self.tail_returns(body)
        if body and not isinstance(rt, TUnit) and body[-1].kind == "exprstmt":
            self.block(body[:-1], new_scope=False)
            c, t = self.expr(body[-1].expr, rt)
            self.emit(f"return {self.coerce(c, t, rt, body[-1])};")
        else:
            self.block(body, new_scope=False)

    def dense_set(self, t):
        """(lo, size) C++ exprs when a Set's keys have a small known range, else None."""
        e = t.elem
        if isinstance(e, TDom):
            return ("0", e.info.len_cpp)
        if isinstance(e, TInt) and e.lo is not None and e.hi is not None and e.hi - e.lo < (1 << 26):
            return (str(e.lo), str(e.hi - e.lo + 1))
        return None

    def unify(self, ast, t, tps, bind, node):
        if ast is None or t is None:
            return
        if ast.kind == "tname" and ast.name in tps and not ast.args:
            if not isinstance(t, TDom):
                self.error(node, f"type parameter {ast.name} must be a domain, got {t!r}")
            if ast.name in bind and bind[ast.name].info is not t.info:
                self.error(node, f"type parameter {ast.name} bound to both {bind[ast.name]!r} and {t!r}")
            bind[ast.name] = t
            return
        if ast.kind == "tname" and ast.args:
            inner = {TList: "elem", TSet: "elem", TOpt: "inner"}
            for cls, attr in inner.items():
                if isinstance(t, cls):
                    self.unify(ast.args[0], getattr(t, attr), tps, bind, node)
            if isinstance(t, TMap) and len(ast.args) == 2:
                self.unify(ast.args[0], t.key, tps, bind, node)
                self.unify(ast.args[1], t.val, tps, bind, node)
            return
        for a, attr in (("inner", "inner"), ("elem", "elem")):
            if hasattr(ast, a) and getattr(ast, a) is not None and hasattr(t, attr):
                self.unify(getattr(ast, a), getattr(t, attr), tps, bind, node)
        if hasattr(ast, "dims") and isinstance(t, TArr):
            for d, td in zip(ast.dims, t.dims):
                self.unify(d, td, tps, bind, node)

    def specialize(self, g, bind):
        key = (g.name,) + tuple(bind[tp].info.cpp for tp in g.tparams)
        specs = self.__dict__.setdefault("specs", {})
        if key not in specs:
            import copy
            clone = copy.copy(g)
            clone.name = "_".join(key)
            clone.tparams = []
            clone.decos = [d for d in g.decos if d.name != "brute"]
            binds = {tp: Sym("type", bind[tp], bind[tp].info.cpp) for tp in g.tparams}
            saved = (self.scopes, self.at_top, self.fn_ret, getattr(self, "loop_ctx", []), getattr(self, "in_multitest", False))
            self.scopes, self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = [self.scopes[0], binds], True, [], [], False
            specs[key] = clone.name
            try:
                self.capture(lambda: self.s_fn(clone))
            finally:
                self.scopes, self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = saved
            specs[key] = binds[clone.name]
        sym = specs[key]
        return self.lookup(sym) if isinstance(sym, str) else sym

    def call_generic(self, g, e):
        """Monomorphize a generic fn for the domains its arguments use."""
        if len(e.args) != len(g.params):
            self.error(e, f"`{g.name}` takes {len(g.params)} arguments")
        args = [self.expr(a) for a in e.args]
        bind = {}
        for (_, at), p in zip(args, g.params):
            self.unify(p.ty, at, set(g.tparams), bind, e)
        missing = [tp for tp in g.tparams if tp not in bind]
        if missing:
            self.error(e, f"cannot infer {', '.join(missing)} for `{g.name}` from its arguments")
        sym = self.specialize(g, bind)
        for h in [x.node for x in self.scopes[0].values() if getattr(x, "kind", None) == "genfn"]:
            bd = {d.name: d for d in h.decos}.get("brute")
            if bd and bd.args and getattr(bd.args[0], "name", None) == g.name and h.tparams == g.tparams:
                key = ("brute", h.name) + tuple(bind[tp].info.cpp for tp in g.tparams)
                done = self.__dict__.setdefault("brute_specs", set())
                if key not in done:
                    done.add(key)
                    self.fn_brutes.append((sym, self.specialize(h, bind), h))
        return f"{sym.cpp}({', '.join(c for c, _ in args)})", sym.ty.ret

    def narrow(self, t):
        """Range-typed array elements that fit in 32 bits are stored as int."""
        return type(t) is TInt and t.lo is not None and t.hi is not None and -2**31 <= t.lo and t.hi < 2**31

    def add_swap_hooks(self):
        """Under -DCP_SWAP_<f>, f forwards to its @brute (used by `cp locate` to blame one part)."""
        def find(cpp):
            for k, f in enumerate(self.funcs):
                head = f.split("{\n", 1)[0]
                if "{\n" in f and re.match(r"^[\w:<>, ]*\b" + re.escape(cpp) + r"\(", head) and not head.startswith("template"):
                    return k, head
            return None, None
        for fast, slow, node in self.fn_brutes:
            k, head = find(fast.cpp)
            _, shead = find(slow.cpp)
            if k is None or shead is None:
                continue
            inside = head[head.index("(") + 1:head.rindex(")")]
            names = [re.search(r"(\w+)\s*$", p).group(1) for p in inside.split(",") if p.strip()]
            body = self.funcs[k].split("{\n", 1)[1]
            self.funcs[k] = (head + "{\n#ifdef CP_SWAP_" + fast.cpp + "\n    return " + slow.cpp +
                             "(" + ", ".join(names) + ");\n#endif\n" + body)
            self.funcs.insert(0, shead.rstrip() + ";")

    def word_set(self, t):
        e = t.elem
        return isinstance(e, TDom) and e.info.len_iv[1] is not None and e.info.len_iv[1] <= 64

    def set_ctor(self, t):
        if isinstance(t, TMap) and isinstance(t.key, TDom):
            return f"{self.cpp_type(t)}({t.key.info.len_cpp})"
        if isinstance(t, TSet) and self.word_set(t):
            return "Mask64()"
        d = self.dense_set(t) if isinstance(t, TSet) else None
        return f"DenseSet({d[0]}, {d[1]})" if d else f"{self.cpp_type(t)}()"

    def gen_expr(self, t, node):
        """C++ expression producing a random value of type t (for per-fn stress tests)."""
        if isinstance(t, TBool):
            return "(bool)gen_int(0, 1)"
        if isinstance(t, TInt):
            lo = t.lo if t.lo is not None else -10
            hi = t.hi if t.hi is not None else 10
            return f"gen_int({lo}, {hi})"
        if isinstance(t, TDom):
            return f"(int)gen_int(0, {t.info.len_cpp} - 1)"
        if isinstance(t, TOpt):
            ot = self.cpp_type(t)
            return f"(gen_int(0, 3) == 0 ? {ot}() : {ot}({self.gen_expr(t.inner, node)}))"
        if isinstance(t, TTuple):
            sets = " ".join(f"_t.{self.field_cpp(f)} = {self.gen_expr(ft, node)};" for f, ft in zip(t.names, t.types))
            return f"[&]{{ {self.cpp_type(t)} _t; {sets} return _t; }}()"
        if isinstance(t, TArr) and len(t.dims) == 1:
            return (f"[&]{{ {self.cpp_type(t)} _v({t.dims[0].info.len_cpp}); "
                    f"for (size_t _i = 0; _i < _v.size(); ++_i) _v[_i] = {self.gen_expr(t.elem, node)}; return _v; }}()")
        if isinstance(t, TList):
            return (f"[&]{{ {self.cpp_type(t)} _v(gen_int(0, 8)); "
                    f"for (size_t _i = 0; _i < _v.size(); ++_i) _v[_i] = {self.gen_expr(t.elem, node)}; return _v; }}()")
        self.error(node, f"cannot generate random arguments of type {t!r} for @brute")

    def emit_fn_tests(self):
        for f, kind, node in self.fn_props:
            self.emit("for (int _it = 0; _it < cp_fn_iters; ++_it) {")
            self.ind += 1
            args = []
            for pt in f.ty.params:
                a = self.tmpname("arg")
                self.emit(f"auto {a} = {self.gen_expr(pt, node)};")
                args.append((a, pt))
            al = ", ".join(a for a, _ in args)
            self.emit(f"auto _r1 = {f.cpp}({al});")
            for a, pt in args:
                if isinstance(pt, (TArr, TList)):
                    if kind == "rev":
                        self.emit(f"reverse({a}.begin(), {a}.end());")
                    else:
                        self.emit(f"for (size_t _i = {a}.size(); _i > 1; --_i) swap({a}[_i - 1], {a}[gen_int(0, (ll)_i - 1)]);")
            self.emit(f"auto _r2 = {f.cpp}({al});")
            self.emit("if (!(_r1 == _r2)) {")
            self.ind += 1
            self.emit(f'out_str("PROPERTY FAILS: {node.name} is not invariant under {kind}\\nafter {kind}:");')
            for (a, pt), p in zip(args, node.params):
                self.emit(f'out_str(" {p.name}=");')
                self.emit_value(a, pt, node)
            self.emit(f'out_str("\\nbefore: ");')
            self.emit_value("_r1", f.ty.ret, node)
            self.emit(f'out_str("\\nafter: ");')
            self.emit_value("_r2", f.ty.ret, node)
            self.emit("out_char('\\n'); out_flush(); return 1;")
            self.ind -= 1
            self.emit("}")
            self.ind -= 1
            self.emit("}")
            self.emit(f'out_str("ok {node.name} invariant under {kind}\\n");')
        for fast, slow, node in self.fn_brutes:
            self.emit(f"for (int _it = 0; _it < cp_fn_iters; ++_it) {{")
            self.ind += 1
            args = []
            for k, pt in enumerate(fast.ty.params):
                a = self.tmpname("arg")
                self.emit(f"auto {a} = {self.gen_expr(pt, node)};")
                args.append((a, pt))
            al = ", ".join(a for a, _ in args)
            self.emit(f"auto _r1 = {fast.cpp}({al}); auto _r2 = {slow.cpp}({al});")
            self.emit("if (!(_r1 == _r2)) {")
            self.ind += 1
            self.emit(f'out_str("MISMATCH {fast.node.name} vs @brute {node.name}\\nargs:");')
            for (a, pt), p in zip(args, node.params):
                self.emit(f'out_str(" {p.name}=");')
                self.emit_value(a, pt, node)
            self.emit(f'out_str("\\n{fast.node.name}: ");')
            self.emit_value("_r1", fast.ty.ret, node)
            self.emit(f'out_str("\\n{node.name}: ");')
            self.emit_value("_r2", fast.ty.ret, node)
            self.emit("out_char('\\n'); out_flush(); return 1;")
            self.ind -= 1
            self.emit("}")
            self.ind -= 1
            self.emit("}")
            self.emit(f'out_str("ok {fast.node.name} vs {node.name}\\n");')

    def note_write(self, target, line):
        root = target
        while root.kind in ("index", "field"):
            root = root.obj
        if root.kind == "name":
            sym = self.lookup(root.name)
            if sym is not None:
                self.writes.append((sym, line))

    def check_memo_staleness(self):
        for fn, read, sym in self.memo_reads:
            for wsym, line in self.writes:
                if wsym is sym and line > fn.line:
                    self.error(read, f"@memo fn `{fn.name}` reads var `{read.name}`, which is written again on line {line}; "
                                     "cached results would go stale (pass it as a parameter, or finish building it before the fn)")

    def check_memo_reads(self, s):
        """A memoized result must not depend on mutable state, or later calls would see stale values."""
        local = {p.name for p in s.params}
        def walk(x):
            if isinstance(x, (list, tuple)):
                for y in x:
                    walk(y)
                return
            if not isinstance(x, N):
                return
            if x.kind in ("let", "for", "iflet", "whilelet") and getattr(x, "pat", None) is not None:
                local.update(pat_names(x.pat))
            for k in ("binds", "bindings"):
                for b in getattr(x, k, None) or []:
                    if isinstance(b, str):
                        local.add(b)
            if x.kind == "name" and x.name not in local:
                sym = self.lookup(x.name)
                if sym is not None and sym.kind in ("let", "var") and sym.mut:
                    self.memo_reads.append((s, x, sym))
            for v in x.__dict__.values():
                if isinstance(v, (N, list, tuple)):
                    walk(v)
        walk(s.body)

    def check_pure(self, s):
        def walk(x, local_names):
            if isinstance(x, list):
                for y in x:
                    walk(y, local_names)
                return
            if not isinstance(x, N):
                if isinstance(x, tuple):
                    for y in x:
                        walk(y, local_names)
                return
            if x.kind == "assign":
                root = x.target
                while root.kind in ("index", "field"):
                    root = root.obj
                if root.kind == "name" and root.name not in local_names:
                    sym = self.lookup(root.name)
                    if sym is not None and sym.mut:
                        self.error(x, f"pure fn `{s.name}` writes outer var `{root.name}`")
            if x.kind == "let" and x.pat.kind == "pname":
                local_names.add(x.pat.name)
            if x.kind == "for":
                for n in pat_names(x.pat):
                    local_names.add(n)
            for v in x.__dict__.values():
                if isinstance(v, (N, list, tuple)):
                    walk(v, local_names)
        walk(s.body, {p.name for p in s.params})

    def solution_fn(self, s, decos):
        name = s.name
        mt = "multitest" in decos
        self.scopes.append({})
        saved_top = self.at_top
        self.at_top = mt
        self.in_multitest = mt
        self.fn_ret.append(UNIT)
        self.sol_depth = len(self.fn_ret)
        self.read_pos = None
        holder = {}
        def body():
            holder["out"] = self.out
            self.block(s.body, new_scope=False)
        lines, _ = self.capture(body)
        self.fn_ret.pop()
        self.at_top = saved_top
        self.in_multitest = False
        self.scopes.pop()
        if mt:
            pos = self.read_pos[1] if self.read_pos and self.read_pos[0] == id(holder["out"]) else 0
            lines = lines[:pos] + ["    if (!CP_GEN) {"] + ["    " + l for l in lines[pos:]] + ["    }"]
            lines = (["    ll _T = CP_GEN ? gen_int(1, 3) : rd_int();",
                      "    if (CP_GEN) { out_int(_T); out_char('\\n'); }",
                      "    while (_T--) {"] + ["    " + l for l in lines] + ["    }"])
        when = None
        if "brute" in decos:
            for a in decos["brute"].args:
                if a.kind == "kwarg" and a.name == "when":
                    when, _ = self.capture(lambda: self.expr(a.value))[1]
        self.solutions[name] = (lines, "multitest" in decos, when)


def pat_names(p):
    if p.kind == "pname":
        return [p.name]
    out = []
    for q in p.items:
        out += pat_names(q)
    return out

# ---------- I/O emitters and driver (attached to Compiler)
def _emit_read(self, target, ty, node):
    """Emit code reading (or, in gen mode, generating) a value of type ty into target."""
    ln = node.line
    if isinstance(ty, TInt):
        lo = "LLONG_MIN" if ty.lo is None else f"{ty.lo}LL"
        hi = "LLONG_MAX" if ty.hi is None else f"{ty.hi}LL"
        self.emit(f"{target} = CP_GEN ? gen_int({lo}, {hi}) : rd_range({lo}, {hi}, {ln});")
        self.emit(f"if (CP_GEN) {{ out_int({target}); out_char(' '); }}")
    elif isinstance(ty, TDom):
        info = ty.info
        self.emit(f"{target} = CP_GEN ? (int)gen_int(0, {info.len_cpp} - 1) : rd_dom({info.lo_cpp}, {info.len_cpp}, {ln});")
        self.emit(f"if (CP_GEN) {{ out_int({self.dom_label(info, target)}); out_char(' '); }}")
    elif isinstance(ty, TBool):
        self.emit(f"{target} = CP_GEN ? gen_int(0, 1) : rd_int() != 0;")
        self.emit(f"if (CP_GEN) {{ out_int({target}); out_char(' '); }}")
    elif isinstance(ty, TChar):
        self.emit(f"{target} = CP_GEN ? (char)('a' + gen_int(0, 25)) : rd_char();")
        self.emit(f"if (CP_GEN) {{ out_char({target}); out_char(' '); }}")
    elif isinstance(ty, TStr):
        self.emit(f"{target} = rd_word();")
    elif isinstance(ty, TTuple):
        for fn, ft in zip(ty.names, ty.types):
            self.emit_read(f"{target}.{self.field_cpp(fn)}", ft, node)
    elif isinstance(ty, TEnum) and ty.tags:
        names = ", ".join(json.dumps(t) for t in ty.tags)
        chain = " : ".join(f'w == {json.dumps(t)} ? {k}' for k, t in enumerate(ty.tags))
        self.emit(f"if (CP_GEN) {{ {target}.tag = (int)gen_int(0, {len(ty.tags) - 1}); static const char* _tn[] = {{{names}}}; out_str(_tn[{target}.tag]); out_char(' '); }}")
        self.emit(f"else {{ string w = rd_word(); {target}.tag = {chain} : (cp_fail(\"unknown tag\", {ln}), 0); }}")
        for k, (vn, fns, fts) in enumerate(ty.variants):
            if not fns:
                continue
            self.emit(f"if ({target}.tag == {k}) {{")
            self.ind += 1
            for f, ft in zip(fns, fts):
                self.emit_read(f"{target}.{vn}_{f}", ft, node)
            self.ind -= 1
            self.emit("}")
    elif isinstance(ty, TEnum):
        self.emit(f"{target}.tag = CP_GEN ? (int)gen_int(0, {len(ty.variants) - 1}) : (int)rd_range(0, {len(ty.variants) - 1}, {ln});")
        self.emit(f"if (CP_GEN) {{ out_int({target}.tag); out_char(' '); }}")
        for k, (vn, fns, fts) in enumerate(ty.variants):
            if not fns:
                continue
            self.emit(f"if ({target}.tag == {k}) {{")
            self.ind += 1
            for f, ft in zip(fns, fts):
                self.emit_read(f"{target}.{vn}_{f}", ft, node)
            self.ind -= 1
            self.emit("}")
    elif isinstance(ty, TOpt) and isinstance(ty.inner, TDom):
        info = ty.inner.info
        x = self.tmpname("r")
        self.emit(f"{{ ll {x} = CP_GEN ? (gen_int(0, 1) ? -1 : {info.lo_cpp} + gen_int(0, {info.len_cpp} - 1)) : rd_int();")
        self.emit(f"if (CP_GEN) {{ out_int({x}); out_char(' '); }}")
        self.emit(f"{target} = {x} == -1 ? Opt<int>() : Opt<int>(chk_dom({x} - {info.lo_cpp}, {info.len_cpp}, {ln})); }}")
    elif isinstance(ty, TPerm):
        d = ty.elem.info
        n = f"{d.len_cpp}"
        self.emit(f"{target}.assign({n}, 0);")
        k = self.tmpname("k")
        if d.labels == "char":
            w = self.tmpname("w")
            self.emit(f"if (CP_GEN) {{ for (int {k} = 0; {k} < {n}; ++{k}) {target}[{k}] = {k}; "
                      f"shuffle({target}.begin(), {target}.end(), cp_rng); "
                      f"for (int {k} = 0; {k} < {n}; ++{k}) out_char((char)({target}[{k}] + {d.lo_cpp})); out_char(' '); }}")
            self.emit(f"else {{ string {w} = rd_word(); CP_ASSERT((ll){w}.size() == {n}, \"Perm has the wrong length\", {ln}); "
                      f"for (int {k} = 0; {k} < {n}; ++{k}) {target}[{k}] = chk_dom({w}[{k}] - {d.lo_cpp}, {n}, {ln}); }}")
        else:
            self.emit(f"if (CP_GEN) {{ for (int {k} = 0; {k} < {n}; ++{k}) {target}[{k}] = {k}; "
                      f"shuffle({target}.begin(), {target}.end(), cp_rng); "
                      f"for (int {k} = 0; {k} < {n}; ++{k}) {{ out_int({target}[{k}] + {d.lo_cpp}); out_char(' '); }} }}")
            self.emit(f"else for (int {k} = 0; {k} < {n}; ++{k}) {target}[{k}] = rd_dom({d.lo_cpp}, {n}, {ln});")
    elif isinstance(ty, TList):
        cnt = self.tmpname("c")
        self.emit(f"{{ ll {cnt} = CP_GEN ? gen_int(0, cp_size) : rd_int();")
        self.emit(f"if (CP_GEN) {{ out_int({cnt}); out_char(' '); }}")
        self.emit(f"{target}.assign({cnt}, {self.cpp_type(ty.elem)}());")
        v = self.tmpname("k")
        self.emit(f"for (ll {v} = 0; {v} < {cnt}; ++{v}) {{")
        self.ind += 1
        self.emit_read(f"{target}[{v}]", ty.elem, node)
        self.ind -= 1
        self.emit("} }")
    elif isinstance(ty, TArr):
        size = " * ".join(f"(size_t){d.info.len_cpp}" for d in ty.dims)
        self.emit(f"{target}.assign({size}, {self.cpp_type(ty.elem)}());")
        v = self.tmpname("k")
        self.emit(f"for (size_t {v} = 0; {v} < {target}.size(); ++{v}) {{")
        self.ind += 1
        self.emit_read(f"{target}[{v}]", ty.elem, node)
        self.ind -= 1
        self.emit("}")
    else:
        self.error(node, f"cannot read {ty!r}")

def _emit_print(self, args, newline, node):
    # in input-generation mode the whole program runs, but only the generated input is printed
    self.emit("if (!CP_GEN) {")
    self.ind += 1
    _emit_print_body(self, args, newline, node)
    self.ind -= 1
    self.emit("}")

def _emit_print_body(self, args, newline, node):
    first = True
    for a in args:
        c, t = self.expr(a)
        if not first:
            self.emit("out_char(' ');")
        first = False
        self.emit_value(c, t, a)
    if newline:
        self.emit("out_char('\\n');")

def _emit_value(self, c, t, node):
    if isinstance(t, TInt):
        self.emit(f"out_int({c});")
    elif isinstance(t, TDom):
        self.emit(f"out_int({self.dom_label(t.info, c)});")
    elif isinstance(t, TBool):
        self.emit(f"out_str(({c}) ? \"YES\" : \"NO\");")
    elif isinstance(t, TChar):
        self.emit(f"out_char({c});")
    elif isinstance(t, TStr):
        self.emit(f"out_str({c});")
    elif isinstance(t, TMod):
        self.emit(f"out_int(({c}).v);")
    elif isinstance(t, TOpt):
        o = self.tmpname("o")
        self.emit(f"{{ auto {o} = {c}; if (!{o}.has()) out_int(-1); else {{")
        self.ind += 1
        self.emit_value(f"{o}.get()", t.inner, node)
        self.ind -= 1
        self.emit("} }")
    elif isinstance(t, TTuple):
        tmp = self.tmpname("p")
        self.emit(f"{{ const auto& {tmp} = {c};")
        for i, (fn, ft) in enumerate(zip(t.names, t.types)):
            if i:
                self.emit("out_char(' ');")
            self.emit_value(f"{tmp}.{self.field_cpp(fn)}", ft, node)
        self.emit("}")
    elif isinstance(t, (TArr, TList)):
        tmp, k = self.tmpname("p"), self.tmpname("k")
        sep = "" if isinstance(t.elem, TChar) else f"if ({k}) out_char(' ');"
        if isinstance(t, TArr) and len(t.dims) == 2:
            w = t.dims[1].info.len_cpp
            sep = (f"if ({k} && {k} % ({w}) == 0) out_char('\\n');" +
                   ("" if isinstance(t.elem, TChar) else f" else if ({k}) out_char(' ');"))
        self.emit(f"{{ const auto& {tmp} = {c}; for (size_t {k} = 0; {k} < {tmp}.size(); ++{k}) {{ {sep}")
        self.ind += 1
        self.emit_value(f"{tmp}[{k}]", t.elem, node)
        self.ind -= 1
        self.emit("} }")
    elif isinstance(t, TEnum):
        tmp = self.tmpname("e")
        self.emit(f"{{ const auto& {tmp} = {c};")
        if t.tags:
            names = ", ".join(json.dumps(x) for x in t.tags)
            self.emit(f"{{ static const char* _tn[] = {{{names}}}; out_str(_tn[{tmp}.tag]); }}")
        else:
            self.emit(f"out_int({tmp}.tag);")
        for k, (vn, fns, fts) in enumerate(t.variants):
            if not fns:
                continue
            self.emit(f"if ({tmp}.tag == {k}) {{")
            self.ind += 1
            for f, ft in zip(fns, fts):
                self.emit("out_char(' ');")
                self.emit_value(f"{tmp}.{vn}_{f}", ft, node)
            self.ind -= 1
            self.emit("}")
        self.emit("}")
    else:
        self.error(node, f"cannot print {t!r}")

Compiler.emit_read = _emit_read
Compiler.emit_print = _emit_print
Compiler.emit_value = _emit_value


def _program(self):
    self.out = self.init_lines
    self.memo_reads, self.writes, self.fn_brutes, self.fn_props = [], [], [], []
    for s in self.prog.body:
        self.stmt(s)
    self.check_memo_staleness()
    self.fntest_lines, _ = self.capture(self.emit_fn_tests)
    self.add_swap_hooks()

Compiler.program = _program


def compile_program(prog, max_passes=6):
    var_iv = {}
    comp = None
    for _ in range(max_passes):
        comp = Compiler(prog, var_iv)
        comp.program()
        nxt = comp.finish_ranges()
        merged = dict(var_iv)
        changed = False
        for k, v in nxt.items():
            if merged.get(k) != v:
                merged[k] = v
                changed = True
        if not changed:
            break
        var_iv = merged
    else:
        comp = Compiler(prog, {k: (None, None) for k in var_iv})
        comp.program()
    return render(comp)


def render(comp):
    res = _render(comp)
    code = res[0] if isinstance(res, tuple) else res
    def keep(m):
        name, body = m.group(1), m.group(2)
        rest = code[:m.start()] + code[m.end():]
        return body if re.search(r"\b(?:" + name + r")\b", rest) else ""
    code = re.sub(r"// @part ([\w|]+)\n(.*?)// @end\n", keep, code, flags=re.S)
    return (code,) + tuple(res[1:]) if isinstance(res, tuple) else code


def _render(comp):
    import os
    rt = open(os.path.join(os.path.dirname(__file__), "runtime.hpp")).read()
    sols = comp.solutions
    init = comp.init_lines
    gen_exit = "    if (CP_GEN && !CP_FNTEST_ON) { out_flush(); return 0; }"
    if not sols:
        sols = {"main": (init[comp.last_read:], False, None)}
        init = init[:comp.last_read]
    parts = [rt, "#ifndef CP_GEN_MODE\n#define CP_GEN 0\n#else\n#define CP_GEN 1\n#endif",
             "#ifndef CP_FNTEST\n#define CP_FNTEST_ON 0\n#else\n#define CP_FNTEST_ON 1\n#endif"]
    parts += comp.struct_defs + comp.globals + comp.funcs
    out = []
    multitest = any(m for _, m, _ in sols.values())
    out.append("int main(int argc, char** argv) {")
    out.append("    if (CP_GEN) { if (argc > 1) cp_rng.seed(atoll(argv[1])); if (argc > 2) cp_size = atoll(argv[2]); }")
    out.append("    const char* cp_sol = getenv(\"CP_SOLUTION\");")
    out += init
    if not any(m for _, m, _ in sols.values()) and not comp.reads_in_fns:
        out.append(gen_exit)
    names = list(sols)
    main_name = next((n for n in names if n == "main"), names[0])
    for n in names:
        lines, mt, when = sols[n]
        if n == main_name:
            continue
        out.append(f"    if (cp_sol && string(cp_sol) == \"{n}\") {{")
        out += ["    " + l for l in lines]
        out.append("        out_flush(); return 0;\n    }")
    out.append("    {")
    out += ["    " + l for l in sols[main_name][0]]
    out.append("    }")
    if comp.fntest_lines:
        out.append("#ifdef CP_FNTEST\n    {\n    out_flush(); printf(\"@@FNTEST\\n\"); fflush(stdout);")
        out += ["    " + l for l in comp.fntest_lines]
        out.append("    out_flush(); return 0;\n    }\n#endif")
    out.append("    out_flush();\n    return 0;\n}")
    code = "\n\n".join(parts) + "\n\n" + "\n".join(out) + "\n"
    return code, comp
