import copy
import re
import json
"""Type checker + C++ emitter (one pass). See PLAN.md for representation decisions."""
from .ast import N
from .types import *
from .lends import LendMixin, Hold, Proj, SCALARS, IO_FNS, show, walk, root_name, written_names, names_in

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
    def __init__(self, elem, hashed=False, bag=False): self.elem, self.hashed, self.bag = elem, hashed, bag
    def __eq__(self, o): return isinstance(o, TSet) and (o.elem, o.hashed, o.bag) == (self.elem, self.hashed, self.bag)
    def __hash__(self): return hash(("set", self.elem, self.hashed, self.bag))
    def __repr__(self): return f"{'HashSet' if self.hashed else 'SortedBag' if self.bag else 'Set'}<{self.elem!r}>"

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

class TAssignSum(T):
    def __init__(self, dom): self.dom = dom
    def __eq__(self, o): return isinstance(o, TAssignSum) and o.dom == self.dom
    def __hash__(self): return hash(("assignsum", self.dom))
    def __repr__(self): return f"AssignSum<{self.dom!r}>"

class TAddMax(T):
    def __init__(self, dom): self.dom = dom
    def __eq__(self, o): return isinstance(o, TAddMax) and o.dom == self.dom
    def __hash__(self): return hash(("addmax", self.dom))
    def __repr__(self): return f"AddMax<{self.dom!r}>"

class TSeg(T):
    def __init__(self, dom, elem): self.dom, self.elem = dom, elem
    def __eq__(self, o): return isinstance(o, TSeg) and o.elem == self.elem and o.dom == self.dom
    def __hash__(self): return hash(("seg", self.dom, self.elem))
    def __repr__(self): return f"SegTree<{self.dom!r}, {self.elem!r}>"

class TCounter(T):
    """Counter<K>: a count per key (0 for a key never touched) and `distinct`, the number of non-zero counts."""
    def __init__(self, key): self.key = key
    def __eq__(self, o): return isinstance(o, TCounter) and o.key == self.key
    def __hash__(self): return hash(("counter", self.key))
    def __repr__(self): return f"Counter<{self.key!r}>"

class TPrefix(T):
    """prefix(a): prefix sums of a 1-D or 2-D array, read with `pre.sum(range)` / `pre.sum(rows, cols)`."""
    def __init__(self, dims, narrow=False): self.dims = tuple(dims); self.narrow = narrow
    def __eq__(self, o): return isinstance(o, TPrefix) and o.dims == self.dims and o.narrow == self.narrow
    def __hash__(self): return hash(("prefix", self.dims, self.narrow))
    def __repr__(self): return "Prefix<" + ", ".join(map(repr, self.dims)) + ">"

class THash(T):
    def __init__(self, dom): self.dom = dom
    def __eq__(self, o): return isinstance(o, THash) and o.dom == self.dom
    def __hash__(self): return hash(("strhash", self.dom))
    def __repr__(self): return f"StrHash<{self.dom!r}>"

class TTree(T):
    weighted = False                    # built with `weight:`, so `dist` adds lengths up
    def __init__(self, dom): self.dom = dom
    def __eq__(self, o): return isinstance(o, TTree) and o.dom == self.dom
    def __hash__(self): return hash(("tree", self.dom))
    def __repr__(self): return f"Rooted<{self.dom!r}>"

class TDsu(T):
    def __init__(self, dom): self.dom = dom
    def __eq__(self, o): return isinstance(o, TDsu) and o.dom == self.dom
    def __hash__(self): return hash(("dsu", self.dom))
    def __repr__(self): return f"Dsu<{self.dom!r}>"

class TLib(T):
    """A struct of the standard library that the prototype compiles in (see LIBS): its name and type arguments."""
    def __init__(self, name, args): self.name, self.args = name, tuple(args)
    def __eq__(self, o): return isinstance(o, TLib) and (o.name, o.args) == (self.name, self.args)
    def __hash__(self): return hash(("lib", self.name, self.args))
    def __repr__(self): return f"{self.name}<{', '.join(map(repr, self.args))}>"

class THeap(T):
    def __init__(self, elem, largest=False): self.elem, self.largest = elem, largest
    def __eq__(self, o): return isinstance(o, THeap) and (o.elem, o.largest) == (self.elem, self.largest)
    def __hash__(self): return hash(("heap", self.elem, self.largest))
    def __repr__(self): return f"{'Max' if self.largest else 'Min'}Heap<{self.elem!r}>"

class TRt(T):
    """A v2 prelude container the prototype backs with a C++ class in runtime.hpp: Deque<T>, MonoDeque<T>, Desc<T>."""
    def __init__(self, name, args, cpp): self.name, self.args, self.cpp = name, tuple(args), cpp
    def __eq__(self, o): return isinstance(o, TRt) and (o.name, o.args) == (self.name, self.args)
    def __hash__(self): return hash(("rt", self.name, self.args))
    def __repr__(self): return f"{self.name}<{', '.join(map(repr, self.args))}>"
    @property
    def elem(self): return self.args[0]

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
    "empty", "ws", "ends", "endl", "flush", "dec", "hex", "fixed", "bitset", "deque", "complex", "ref", "apply",
    "visit", "any", "all", "signal", "div", "ldiv", "exit", "free", "link", "read", "write",
    "select", "stdin", "stdout", "floor", "ceil", "round", "lower_bound", "upper_bound",
}

# Library names the prototype has a C++ stand-in for: name -> (module under stdlib/, C++ name). `use std.dsu.{Dsu}`
# is checked against the real module, so a port can only import what the library really exports.
MEMO_DENSE_MAX = 1 << 26     # slots of a dense @memo table (at most ~1 GB of optional Ints); larger products are hashed
LIBS = {"Steps": ("steps", "CpSteps")}
BUILTIN_TYPE_ARGS = {"Fenwick": 2, "LazySegTree": 2, "SparseTable": 2, "Deque": 1, "MonoDeque": 1, "Desc": 1, "MaxHeap": 1, "List": 1, "Map": 2, "HashMap": 2, "Set": 1, "HashSet": 1, "SortedBag": 1, "Bitset": 1, "Marks": 1, "Perm": 1, "MinHeap": 1, "Dsu": 1, "AssignSum": 1, "AddMax": 1,
                     "SegTree": 2, "Subset": 1, "Counter": 1, "Range": 1, "Fenwick2D": 3, "Matrix": 3, "FlowGraph": 1, "TwoSat": 1, "Steps": 2}
LIB_FNS = {"Dsu": "dsu", "Graph": "graph", "bfs": "graph", "dijkstra": "graph", "choose": "math", "gcd": "math", "lcm": "math", "scan": "seq",
           "running_sum": "seq", "SegTree": "segtree", "Rooted": "tree", "Fenwick": "fenwick"}

# The identity each built-in two-argument operation folds from (std.algebra: Sum, Prod, Xor and friends).
# `min` and `max` have none over Int; their fold starts at the first value (the `opt` monoid).
FOLD_IDENTITY = {"+": "0", "-": "0", "*": "1", "|": "0", "^": "0", "gcd": "0", "lcm": "1"}

class ConstStruct:
    """Compile-time value of a `const struct` POD config (Synthesis #3 Leap 4)."""
    def __init__(self, name, fields):
        self.name = name
        self.fields = dict(fields)  # field_name -> python const value

    def key(self):
        return (self.name, tuple((k, v.key() if isinstance(v, ConstStruct) else v) for k, v in sorted(self.fields.items())))

    def __eq__(self, o):
        return isinstance(o, ConstStruct) and self.key() == o.key()

    def __hash__(self):
        return hash(self.key())

    def __repr__(self):
        return f"{self.name}({', '.join(f'{k}: {v!r}' for k, v in self.fields.items())})"


class Sym:
    def __init__(self, kind, ty, cpp, mut=False, node=None, const_val=None):
        self.kind, self.ty, self.cpp, self.mut, self.node = kind, ty, cpp, mut, node
        self.const_val = const_val
        self.fact = None

def unparse(e):
    k = e.kind
    if k == "ctemp": return unparse(e.src)
    if k == "num": return str(e.value)
    if k == "name": return e.name
    if k == "binop": return f"{unparse(e.a)} {e.op} {unparse(e.b)}"
    if k == "unop": return f"{e.op}{' ' if e.op == 'not' else ''}{unparse(e.a)}"
    if k == "index": return f"{unparse(e.obj)}[{', '.join(map(unparse, e.idx))}]"
    if k == "field": return f"{unparse(e.obj)}.{e.name}"
    if k == "call": return f"{unparse(e.fn)}(...)"
    if k == "paren": return f"({unparse(e.a)})"
    return k

NL = "\n"


class Compiler(LendMixin):
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
        self.io_type = {}
        self.out = None
        self.ind = 1
        self.fn_ret = []
        self.used_names = [set()]
        self.at_top = True
        self.tests_ty = None
        self.tmp = 0
        self.dom_count = 0
        self.opt_narrow = {}          # name -> (Sym, inner_ty)   (Rule N1)
        self.nonempty_narrow = {}     # name -> Sym               (Rule N2)
        self.narrowed_exprs = set()   # id(expr) narrowed by N1/N2 (Invariant 4)
        self.warnings = []
        self.struct_laws = {}         # (struct_cpp, method_name) -> law cond str
        self.struct_getters = {}      # struct_cpp -> {name: fn_name}
        self.const_structs = {}       # name -> (TTuple, struct_node)
        self.gen_structs = {}         # name -> struct_node
        self.gen_struct_insts = {}    # (name, key) -> TTuple
        self.lend_init()

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
        if isinstance(t, TF64): return "double"
        if isinstance(t, TU64): return "unsigned long long"
        if isinstance(t, TI128): return "__int128"
        if isinstance(t, TUnit): return "void"
        if isinstance(t, TDom): return "int"
        if isinstance(t, TMod):
            return f"DynMod<{t.m[1]}>" if isinstance(t.m, tuple) else f"Mod<{t.m}>"
        if isinstance(t, TOpt):
            if self.narrow(t.inner) and t.inner.lo > -2**31:
                return "OptI32"
            return f"Opt<{self.cpp_type(t.inner)}>"
        if isinstance(t, TRange): return "Rng"
        if isinstance(t, TTuple): return t.cpp
        if isinstance(t, TList):
            el = 'int' if self.narrow(t.elem) else ('uint8_t' if isinstance(t.elem, TBool) else self.cpp_type(t.elem))
            return f"vector<{el}>"
        if isinstance(t, TMap):
            if isinstance(t.key, TDom):
                return f"DenseMap<{self.cpp_type(t.val)}>"
            if t.hashed:
                if not self.hashable(t.key):
                    raise CompileError(f"a HashMap key must be a number, a domain value, a Char, a Bool or a record of those, not {t.key!r}; use Map<...>")
                return f"CpHashMap<{self.cpp_type(t.key)}, {self.cpp_type(t.val)}, CpHash>"
            return f"map<{self.cpp_type(t.key)}, {self.cpp_type(t.val)}>"
        if isinstance(t, TSet):
            if t.hashed:
                return f"unordered_set<{self.cpp_type(t.elem)}, CpHash>"
            if t.bag:
                return f"multiset<{self.cpp_type(t.elem)}>"
            if self.word_set(t):
                return "Mask64"
            return "DenseSet" if self.dense_set(t) else f"set<{self.cpp_type(t.elem)}>"
        if isinstance(t, TArr):
            el = 'int' if self.narrow(t.elem) else ('uint8_t' if isinstance(t.elem, TBool) else self.cpp_type(t.elem))
            return f"CpView<{el}>" if getattr(t, "view", None) else f"vector<{el}>"
        if isinstance(t, TEnum): return t.cpp
        if isinstance(t, TCounter):
            if isinstance(t.key, TDom):
                return "CpDenseCounter"
            k = self.cpp_type(t.key)
            m = f"CpHashMap<{k}, ll, CpHash>" if isinstance(t.key, TInt) else f"map<{k}, ll>"
            return f"CpCounter<{m}, {k}>"
        if isinstance(t, TLib): return LIBS[t.name][1]
        if isinstance(t, TPrefix): return "CpPrefix<int>" if t.narrow else "CpPrefix<ll>"
        if isinstance(t, THash): return "StrHash"
        if isinstance(t, TTree): return "RootedTree"
        if isinstance(t, TSeg): return f"SegTree<{self.cpp_type(t.elem)}>"
        if isinstance(t, TAddMax): return "AddMax"
        if isinstance(t, TAssignSum): return "AssignSum"
        if isinstance(t, TDsu): return "CpDsu"
        if isinstance(t, THeap):
            e = self.cpp_type(t.elem)
            return f"priority_queue<{e}, vector<{e}>, {'less' if t.largest else 'greater'}<{e}>>"
        if isinstance(t, TRt) and t.name in ("Fenwick", "SparseTable", "LazySegTree", "Marks", "Bitset", "Pt", "FlowGraph", "TwoSat", "SuffixArray", "RollingHash", "Dir4", "Dir8"):
            return t.cpp
        if isinstance(t, TRt) and t.name == "Fenwick2D":
            return f"CpFenwick2D<{self.cpp_type(t.args[2])}>"
        if isinstance(t, TRt) and t.name == "StepsT":
            return f"CpStepsT<{self.cpp_type(t.args[0])}, {self.cpp_type(t.args[1])}>"
        if isinstance(t, TRt) and t.name == "Matrix":
            return f"CpMat<{self.cpp_type(t.args[2])}>"
        if isinstance(t, TRt):
            return f"{t.cpp}<{', '.join(self.cpp_type(a) for a in t.args)}>"
        raise CompileError(f"no C++ type for {t!r}")

    def orderable(self, t):
        # whether C++ has a `<` for the type: a record only gets one when all its fields do
        if isinstance(t, TEnum):
            return False
        if isinstance(t, (TOpt, TMod)):     # no implicit order: `none` is not "smaller", a residue has no size (v2 O1)
            return False
        if isinstance(t, (TList, TArr)):
            return self.orderable(t.elem)
        return getattr(t, "ordered", True)

    def tuple_type(self, names, types, nominal=None):
        key = (nominal, tuple(names), tuple(types)) if nominal is not None else (tuple(names), tuple(types))
        if key in self.structs:
            return self.structs[key]
        cpp = f"Tup{len(self.structs)}"
        names = [n if n else f"f{i}" for i, n in enumerate(names)]
        t = TTuple(names, list(types), cpp)
        # `<` on a record compares field by field, so every field needs one (an optional or an enum has none)
        t.ordered = all(self.orderable(x) for x in types)
        self.structs[key] = t
        if not names:
            t.hashable = True
            self.struct_defs.append(
                f"struct {cpp} {{\n"
                f"    template <class H> size_t cp_hash(const H&) const {{ return 0; }}\n"
                f"    bool operator<(const {cpp}&) const {{ return false; }}\n"
                f"    bool operator>(const {cpp}&) const {{ return false; }}\n"
                f"    bool operator==(const {cpp}&) const {{ return true; }}\n"
                f"    bool operator!=(const {cpp}&) const {{ return false; }} }};")
            return t
        fields = " ".join(f"{'I32' if self.narrow(ft) else self.cpp_type(ft)} {self.field_cpp(fn)};" for fn, ft in zip(names, types))
        tie = ", ".join(self.field_cpp(fn) for fn in names)
        # a record of hashable fields is a hashable key (HashMap<(Row, Col), V>)
        t.hashable = all(self.hashable(x) for x in types)
        hsh = ""
        if t.hashable:
            mix = " ".join(f"h = h * 0x9e3779b97f4a7c15ULL + hf({'(ll)' if not isinstance(ft, TTuple) else ''}{self.field_cpp(fn)});" for fn, ft in zip(names, types))
            hsh = f"    template <class H> size_t cp_hash(const H& hf) const {{ size_t h = 0; {mix} return h; }}\n"
        self.struct_defs.append(
            f"struct {cpp} {{ {fields}\n"
            f"    auto tie() const {{ return std::tie({tie}); }}\n"
            + hsh
            + (f"    bool operator<(const {cpp}& o) const {{ return tie() < o.tie(); }}\n"
               f"    bool operator>(const {cpp}& o) const {{ return o.tie() < tie(); }}\n"
               if t.ordered else "") +
            f"    bool operator==(const {cpp}& o) const {{ return tie() == o.tie(); }}\n"
            f"    bool operator!=(const {cpp}& o) const {{ return !(*this == o); }} }};")
        return t

    def hashable(self, t):
        return isinstance(t, (TInt, TDom, TBool, TChar, TStr)) or (isinstance(t, TTuple) and getattr(t, "hashable", False))

    def field_cpp(self, name):
        return name + "_" if name in CPP_RESERVED else name

    # ---------- domains
    def tree_pre(self, tt):
        """Rooted<V>.Pre: the positions of V's vertices in a tree's heavy-first preorder (one domain per V)."""
        if not hasattr(self, "pre_domains"):
            self.pre_domains = {}
        v = tt.dom.info
        if v not in self.pre_domains:
            info, _, _ = self.new_domain(f"Pre_{v.name}", "0", "", (0, 0), v.len_iv)
            info.lo_src, info.hi_src = "0", v.len_cpp
            self.pre_domains[v] = info
        info = self.pre_domains[v]
        self.emit(f"{info.cpp}_lo = 0; {info.cpp}_len = {v.len_cpp};")
        return TDom(info)

    def adjacency(self, t, node, what):
        """An adjacency `[V]List<V>` or `[V]List<(V, ...)>`: returns (V, a C++ lambda from an arc to its target)."""
        if isinstance(t, TArr) and len(t.dims) == 1 and isinstance(t.elem, TList):
            a, vt = t.elem.elem, t.dims[0]
            if a == vt:
                return vt, "[](int _a) { return _a; }"
            if isinstance(a, TTuple) and a.types[0] == vt:
                return vt, f"[](const {a.cpp}& _a) {{ return (int)_a.{self.field_cpp(a.names[0])}; }}"
        self.error(node, f"{what} needs an adjacency [V]List<V> or [V]List<(V, ...)>, got {t!r}")

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
        # A domain value is a 32-bit position (decision 4.3): a domain that cannot fit is an error, never a wrap.
        if len_iv[0] is not None and len_iv[0] > DOM_MAX:
            self.error(node, f"domain {name} would have at least {len_iv[0]} values; a domain holds at most 2^31 - 1 "
                             f"(its values are 32-bit positions)\n  help: to loop over a range this large, "
                             f"write `for i in {show(rng)}` without naming a domain")
        if len_iv[1] is None or len_iv[1] > DOM_MAX:
            # the difference of two Int bounds can itself overflow, so it is computed in 128 bits
            self.emit(f"{info.cpp}_lo = {lo_c}; {info.cpp}_len = chk_dom_len((__int128)({hi_c}) - (__int128)({lo_c})"
                      f"{' + 1' if rng.incl else ''}, {node.line});")
        else:
            self.emit(f"{info.cpp}_lo = {lo_c}; {info.cpp}_len = max(0LL, (ll)({len_c}));")
        return info

    @staticmethod
    def step_fn(kt):
        """The runtime check for a position moved by an Int of type `kt`: plain 64-bit when it can't overflow."""
        small = None not in (kt.lo, kt.hi) and -2**62 < kt.lo and kt.hi < 2**62
        return "chk_step" if small else "chk_step_wide"

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
        if k == "tunit":
            return UNIT
        if k == "tifconst":
            cond_val = self.eval_const(tn.cond)
            if not isinstance(cond_val, bool):
                self.error(tn, f"`if const` condition must be Bool, got {type(cond_val).__name__}")
            return self.resolve_type(tn.then_ty if cond_val else tn.else_ty)
        if k == "tname":
            nm = tn.name
            prim = {"Int": INT, "Bool": BOOL, "Char": CHAR, "Str": STR, "F64": F64, "U64": U64, "I128": I128}
            if nm in prim:
                return prim[nm]
            declared = self.lookup(nm)      # a `struct Dsu` or `enum Set` of the program is that name from there on
            if declared and declared.kind == "genstruct":
                return self.specialize_struct(declared.node, tn.args, tn)
            if declared and declared.kind == "type":
                return declared.ty
            need = BUILTIN_TYPE_ARGS.get(nm)
            if need and len(tn.args) != need:
                self.error(tn, f"`{nm}` is a built-in type that takes {need} type argument{'s' if need > 1 else ''}: "
                               f"`{nm}<{', '.join('DKTV'[i] for i in range(need))}>`")
            if nm == "List":
                return TList(self.resolve_type(tn.args[0]))
            if nm in ("Map", "HashMap"):
                return TMap(self.resolve_type(tn.args[0]), self.resolve_type(tn.args[1]), nm == "HashMap")
            if nm in ("Set", "HashSet", "SortedBag"):
                return TSet(self.resolve_type(tn.args[0]), hashed=(nm == "HashSet"), bag=(nm == "SortedBag"))
            if nm == "Dir4" and not tn.args:
                return TRt("Dir4", [], "CpDir4")
            if nm == "Dir8" and not tn.args:
                return TRt("Dir8", [], "CpDir8")
            if nm == "Pt" and not tn.args:
                return TRt("Pt", [], "CpPt")
            if nm == "Matrix" and len(tn.args) == 3:
                return TRt("Matrix", [self.dim_type(tn.args[0]), self.dim_type(tn.args[1]), self.resolve_type(tn.args[2])], "CpMat")
            if nm == "Fenwick2D" and len(tn.args) == 3:
                return TRt("Fenwick2D", [self.dim_type(tn.args[0]), self.dim_type(tn.args[1]), self.resolve_type(tn.args[2])], "CpFenwick2D")
            if nm == "FlowGraph" and len(tn.args) == 1:
                return TRt("FlowGraph", [self.resolve_type(tn.args[0])], "CpFlowGraph")
            if nm == "TwoSat" and len(tn.args) == 1:
                return TRt("TwoSat", [self.dim_type(tn.args[0])], "CpTwoSat")
            if nm == "Steps" and len(tn.args) == 2:
                return TRt("StepsT", [self.resolve_type(tn.args[0]), self.resolve_type(tn.args[1])], "CpStepsT")
            if nm == "Bitset":
                d = self.dim_type(tn.args[0])
                return TRt("Bitset", [d], "CpBitset")
            if nm == "Perm":
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom):
                    self.error(tn, "Perm<D> needs a domain")
                pos, _, _ = self.new_domain(f"Pos<{d.info.name}>", "0", d.info.len_cpp, (0, 0), d.info.len_iv, anon=True)
                pos.lo_src, pos.hi_src = "0", d.info.len_cpp
                self.emit(f"{pos.cpp}_lo = 0; {pos.cpp}_len = {d.info.len_cpp};")
                return TPerm([TDom(pos)], d)
            if nm in ("MinHeap", "MaxHeap"):
                et = self.resolve_type(tn.args[0])
                if not self.orderable(et):
                    self.error(tn, f"{nm}<T> needs an ordered T, and {et!r} has no `<`")
                return THeap(et, nm == "MaxHeap")
            if nm in ("Deque", "MonoDeque", "Desc"):
                et = self.resolve_type(tn.args[0])
                if nm != "Deque" and not self.orderable(et):
                    self.error(tn, f"{nm}<T> needs an ordered T, and {et!r} has no `<`")
                if is_int(et):
                    et = TInt()
                return TRt(nm, [et], {"Deque": "deque", "MonoDeque": "CpMonoDeque", "Desc": "CpDesc"}[nm])
            if nm == "Marks":
                d = self.dim_type(tn.args[0])
                return TRt("Marks", [d], "CpMarks")
            if nm == "Dsu":
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom):
                    self.error(tn, f"Dsu<V> needs a domain, got {d!r}")
                return TDsu(d)
            if nm == "AssignSum":
                d = self.dim_type(tn.args[0]) if tn.args[0].kind.startswith("t") else self.resolve_type(N("texpr", tn.line, expr=tn.args[0]))
                if not isinstance(d, TDom):
                    self.error(tn, "AssignSum<D> needs a domain D")
                return TAssignSum(d)
            if nm == "AddMax":
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom):
                    self.error(tn, "AddMax<D> needs a domain D")
                return TAddMax(d)
            if nm == "SegTree":
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom):
                    self.error(tn, "SegTree<D, T> needs a domain D")
                return TSeg(d, self.resolve_type(tn.args[1]))
            if nm == "Subset":
                # Subset<D>: the domain of all subsets of a small domain D, stored as a bitmask ordinal
                d = self.resolve_type(tn.args[0])
                if not isinstance(d, TDom) or d.info.len_iv[1] is None or d.info.len_iv[1] > 30:
                    self.error(tn, "Subset<D> needs a domain with at most 30 values")
                if getattr(d.info, "subsets", None) is None:
                    b = d.info
                    w, _, _ = self.new_domain(f"Subset<{b.name}>", "", "", (0, 0), (1, 1 << b.len_iv[1]), anon=True)
                    w.name, w.subset_of, w.anon = f"Subset<{b.name}>", b, False
                    w.lo_src, w.hi_src = "0", f"(1LL << {b.len_cpp})"
                    self.emit(f"{w.cpp}_lo = 0; {w.cpp}_len = 1LL << {b.len_cpp};")
                    b.subsets = TDom(w)
                return d.info.subsets
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
                if a.kind == "tname" and not a.args:
                    a = N("name", a.line, name=a.name)
                if a.kind == "name":
                    sym = self.lookup(a.name)
                    if sym and sym.kind in ("let", "var") and is_int(sym.ty):
                        dmods = self.__dict__.setdefault("dyn_mods", {})
                        if sym.cpp not in dmods:
                            tag = len(dmods)
                            dmods[sym.cpp] = tag
                            self.emit(f"DynMod<{tag}>::M = {sym.cpp};")
                        return TMod(("dyn", dmods[sym.cpp]))
                v = a.value if a.kind == "num" else self.const_value(a)
                return TMod(v)
            if nm == "Counter":
                return TCounter(self.resolve_type(tn.args[0]))
            if nm in LIBS and not self.lookup(nm):
                return self.lib_type(nm, [self.resolve_type(a) for a in tn.args], tn)
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
            t = TInt(getattr(lo_t, "lo", None), hi_v)
            # a bound that is a run-time value (`1..=w` with w read earlier): reading checks against it and the
            # input generator stays inside it, not just inside the widest range the bound could have
            if r.lo.kind != "num" and is_int(lo_t):
                t.lo_cpp = f"(ll)({lo_c})"
            if r.hi is not None and r.hi.kind != "num" and is_int(hi_t):
                t.hi_cpp = f"(ll)({hi_c})" + ("" if r.incl else " - 1")
            return t
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

    def lib_type(self, nm, args, node):
        if nm == "Steps" and args:
            self.error(node, "Steps takes no type arguments: its coordinates are Ints (`Steps.new()`)")
        self.need_use(nm, node)
        return TLib(nm, [INT] if nm == "Steps" else list(args))

    def need_use(self, nm, node):
        """Library names are not in scope until a `use` brings them in, the same as in the full language."""
        if nm not in getattr(self, "imported", ()):
            mod = LIBS[nm][0] if nm in LIBS else LIB_FNS[nm]
            self.error(node, f"`{nm}` is in the standard library: add `use std.{mod}.{{{nm}}}` at the top")

    def lib_new(self, t, args, node):
        """`Steps.new()`: the empty value of a compiled-in library struct."""
        if args:
            self.error(node, f"{t!r}.new() takes no arguments")
        return f"{self.cpp_type(t)}()", t

    def s_use(self, s):
        """`use std.dsu.{Dsu}`: the names must be `pub` in stdlib/dsu.cp, and the prototype must have a stand-in."""
        import os
        if s.path[0] != "std" or len(s.path) != 2 or s.names is None:
            self.error(s, "the prototype compiler imports names from the standard library only: `use std.module.{Name}`")
        root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        path = os.path.join(root, "stdlib", s.path[1] + ".cp")
        if not os.path.exists(path):
            # v2.2 moved some opt-in modules (e.g. `steps.cp`) into examples/local-lib/<group>/
            for grp in ("ds", "graph", "grid", "geom", "math", "seq", "str", "dp", "spec"):
                alt = os.path.join(root, "examples", "local-lib", grp, s.path[1] + ".cp")
                if os.path.exists(alt):
                    path = alt
                    break
        if not os.path.exists(path):
            self.error(s, f"there is no module std.{s.path[1]} (stdlib/{s.path[1]}.cp)")
        src = open(path).read()
        for nm in s.names:
            if not re.search(r"^pub (?:@\w+ )*(?:struct|fn|const|enum) " + re.escape(nm) + r"\b", src, re.M):
                self.error(s, f"std.{s.path[1]} has no pub `{nm}`")
            mod = LIBS[nm][0] if nm in LIBS else LIB_FNS.get(nm)
            if mod != s.path[1]:
                self.error(s, f"std.{s.path[1]}.{nm} is library code the prototype compiler has no stand-in for yet "
                              f"(it runs on proto/cpi)")
            self.imported = getattr(self, "imported", set()) | {nm}

    def anon_count_domain(self, name, lo_c, hi_c_excl, lo_iv, hi_iv, node):
        # `[0..n]T` is structural: the same written bounds are the same type wherever they are written (v2 T3)
        cache = self.__dict__.setdefault("anon_doms", {})
        def _norm(s):
            s = s.strip()
            while s.startswith("(") and s.endswith(")"):
                d = 0
                ok = True
                for i, ch in enumerate(s):
                    if ch == "(": d += 1
                    elif ch == ")": d -= 1
                    if d == 0 and i < len(s) - 1:
                        ok = False
                        break
                if ok: s = s[1:-1].strip()
                else: break
            return s
        key = (_norm(lo_c), _norm(hi_c_excl))
        if key in cache and re.fullmatch(r"[\w() +\-*']*$", lo_c + hi_c_excl):
            t = cache[key]
            # If first created inside a nested scope that may not dominate here, re-init bounds idempotently
            if getattr(t.info, "_init_depth", 0) > len(self.scopes) or getattr(t.info, "_init_fn", 0) != len(self.fn_ret):
                self.emit(f"{t.info.cpp}_lo = {lo_c}; {t.info.cpp}_len = max(0LL, (ll)(({hi_c_excl}) - ({lo_c})));")
            return t
        cache[key] = t = self.anon_count_domain_new(name, lo_c, hi_c_excl, lo_iv, hi_iv, node)
        t.info._init_depth = len(self.scopes)
        t.info._init_fn = len(self.fn_ret)
        return t

    def anon_count_domain_new(self, name, lo_c, hi_c_excl, lo_iv, hi_iv, node):
        info, _, _ = self.new_domain(name, lo_c, f"({hi_c_excl}) - ({lo_c})", lo_iv,
                                     (None if hi_iv[0] is None else hi_iv[0] - (lo_iv[1] or 0),
                                      None if hi_iv[1] is None else hi_iv[1] - (lo_iv[0] or 0)), anon=True)
        info.lo_src, info.hi_src = lo_c, hi_c_excl
        self.emit(f"{info.cpp}_lo = {lo_c}; {info.cpp}_len = max(0LL, (ll)(({hi_c_excl}) - ({lo_c})));")
        return TDom(info)

    def bool_domain(self):
        if not hasattr(self, "_bool_dom"):
            info = DomInfo("Bool", "cp_bool_dom", "0", "2", (0, 0), (2, 2), anon=False)
            info.is_bool = True
            self._bool_dom = info
        return self._bool_dom

    def dim_type(self, d):
        if d.kind == "tname" and not d.args:
            sym = self.lookup(d.name)
            if sym is not None and sym.kind == "type" and getattr(sym.node, "kind", None) == "typedef" and sym.node.ty.kind == "trange":
                return self.dim_type(sym.node.ty)
        if d.kind == "trange":
            r = d.range
            lo_c, lo_t = self.expr(r.lo)
            hi_c, hi_t = self.expr(r.hi)
            is_char = isinstance(lo_t, TChar) and isinstance(hi_t, TChar)
            if is_char:
                lo_c = f"(ll)(unsigned char)({lo_c})"
                hi_c = f"(ll)(unsigned char)({hi_c})"
                lo_t, hi_t = TInt(0, 127), TInt(0, 127)
            hi_excl = f"({hi_c}) + 1" if r.incl else hi_c
            hiv = (getattr(hi_t, "lo", None), getattr(hi_t, "hi", None))
            if r.incl:
                hiv = tuple(None if x is None else x + 1 for x in hiv)
            td = self.anon_count_domain(unparse(r), lo_c, hi_excl,
                                        (getattr(lo_t, "lo", None), getattr(lo_t, "hi", None)), hiv, d)
            if is_char:
                td.info.labels = "char"
            return td
        if d.kind == "texpr":
            # `[n - 1]T`: any Int expression is a count, like `[n]T`; the positions are 0..count
            _, (c, t) = self.capture(lambda: self.expr(d.expr))
            if is_int(t):
                c, t = self.expr(d.expr)
                return self.anon_count_domain(f"0..{unparse(d.expr)}", "0", c, (0, 0), (t.lo, t.hi), d)
        t = self.resolve_type(d)
        if isinstance(t, TBool):
            return TDom(self.bool_domain())
        if isinstance(t, TEnum) and getattr(t, "index_info", None) is not None:
            return TDom(t.index_info)
        if isinstance(t, TTuple) and t.types and all(isinstance(ft, TDom) for ft in t.types):
            prod_cpp = " * ".join(f"(ll){ft.info.len_cpp}" for ft in t.types)
            lo_iv, hi_iv = 1, 1
            for ft in t.types:
                lo_iv = None if (lo_iv is None or ft.info.len_iv[0] is None) else lo_iv * ft.info.len_iv[0]
                hi_iv = None if (hi_iv is None or ft.info.len_iv[1] is None) else hi_iv * ft.info.len_iv[1]
            info = DomInfo(f"({', '.join(ft.info.name for ft in t.types)})", f"prod_{t.cpp}", "0", prod_cpp, (0, 0), (lo_iv, hi_iv), anon=False)
            info.prod_tuple = t
            return TDom(info)
        if isinstance(t, TTuple) and t.cpp in self.index_impls:
            return TDom(self.index_impls[t.cpp])     # any struct with `impl Index` is a dimension (plan/19 §3.1)
        if not isinstance(t, TDom):
            self.error(d, f"array dimension must be a domain, a range, or a struct with `impl Index`, got {t!r}")
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
        if isinstance(want, (TF64, TU64, TI128)) and isinstance(t, TInt):
            lit = node.a if getattr(node, "kind", None) == "unop" and node.op == "-" else node
            if getattr(lit, "kind", None) == "num":
                return f"(({self.cpp_type(want)})({c}))"
            self.error(node, f"an Int is not a {want!r}; convert with `{want!r}(x)`")
        if isinstance(want, TMod) and isinstance(t, TInt):
            return f"{self.cpp_type(want)}({c})"
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
            if self.cpp_type(want) != self.cpp_type(t):     # elements stored at another width: copy across
                return f"([&]{{ auto&& _a = {c}; return {self.cpp_type(want)}(_a.begin(), _a.end()); }}())"
            return c
        if isinstance(want, (TMap, TSet)) and isinstance(t, TList) and t.elem is None:
            return self.set_ctor(want)
        if isinstance(want, TList) and isinstance(t, TList) and (t.elem == want.elem or t.elem is None):
            return c if t.elem is not None else f"{self.cpp_type(want)}()"
        if isinstance(t, TOpt) and t.inner == want and isinstance(want, TDom):
            self.error(node, f"expected {want!r}, got {t!r}: `{unparse(node)}` may step outside {want!r}. Bind it with "
                             f"`if let`, give the edge a value with `??`, or assert with `!`")
        self.error(node, f"expected {want!r}, got {t!r}")

    def expr(self, e, want=None):
        m = getattr(self, "x_" + e.kind, None)
        if m is None:
            self.error(e, f"unsupported expression {e.kind}")
        if self.holds and (e.kind in ("name", "field", "index") or self.is_proj_call(e)):
            # only the outermost place expression is checked against the live lends; `a[i]` is not also a use of all of `a`
            if id(e) not in self.covered:
                self.check_access(e, False)
            if e.kind != "name":
                self.covered.add(id(e.fn.obj if e.kind == "call" else e.obj))
        if e.kind in ("list", "tuple"):
            for x in e.items:
                self.escapes[id(x)] = "storing it in a list or tuple"
        elif e.kind == "call":
            if e.fn.kind == "field" and e.fn.name in ("push", "insert", "set", "add"):
                for x in e.args:
                    self.escapes[id(x)] = f"`.{e.fn.name}(...)`"
            elif e.fn.kind == "name" and getattr(self.lookup(e.fn.name), "kind", None) in ("ctor", "type"):
                for x in e.args:
                    self.escapes[id(x)] = f"storing it in a `{e.fn.name}`"
        r = self.ordered(e, want, m) if e.kind in ("call", "binop", "cmp") else m(e, want)
        self.last_type[id(e)] = r[1]
        if getattr(r[1], "view", None):
            if id(e) in self.escapes:
                self.no_escape(r[1], e, self.escapes[id(e)])
        return r

    # ---------- evaluation order
    def operand_slots(self, e):
        """(get, set) pairs for the operands of `e` that C++ may evaluate in any order."""
        def attr(name):
            return (lambda: getattr(e, name)), (lambda v: setattr(e, name, v))
        def item(lst, k):
            return (lambda: lst[k]), (lambda v: lst.__setitem__(k, v))
        if e.kind == "call":
            if e.fn.kind == "name" and e.fn.name in IO_FNS and not self.lookup(e.fn.name):
                return []           # print and read emit one statement per argument already
            return [item(e.args, k) for k in range(len(e.args))]
        if e.kind == "binop":
            return [] if e.op in ("and", "or", "??") else [attr("a"), attr("b")]
        if any(op in ("in", "not in") for op in e.ops):
            return []
        return [attr("first")] + [item(e.rest, k) for k in range(len(e.rest))]

    def ordered(self, e, want, m):
        """Operands and arguments run left to right (plan/19 section 3.7). C++ leaves that order open, so when it
        can be observed (two operands with effects, or one that changes what another reads), every operand but
        the last is evaluated into a temporary first."""
        slots = self.operand_slots(e)
        if len(slots) < 2:
            return m(e, want)
        nodes = [get() for get, _ in slots]
        fx = [self.effects_in(n.value if n.kind == "kwarg" else n) for n in nodes]
        acting = [k for k, (w, _, io) in enumerate(fx) if w or io]
        if not acting:
            return m(e, want)
        if len(acting) == 1:
            w = fx[acting[0]][0]
            if not any((names_in(n) | fx[k][1]) & w for k, n in enumerate(nodes) if k != acting[0]):
                return m(e, want)
        wants = [None] * len(nodes)
        if e.kind == "call" and e.fn.kind == "name":
            fsym = self.lookup(e.fn.name)
            if fsym is not None and fsym.kind == "fn" and isinstance(fsym.ty, TFn):
                wants = list(fsym.ty.params) + wants
        pre, restore = [], []
        try:
            for k, (n, (get, put)) in enumerate(zip(nodes[:-1], slots)):
                if getattr(n, "mut", False) or n.kind in ("lambda", "kwarg", "num", "char", "bool", "str", "none", "range", "gen"):
                    continue
                if n.kind == "name" and getattr(self.lookup(n.name), "kind", None) not in ("let", "var"):
                    continue
                c, t = self.expr(self.typed(n, wants[k]), wants[k])
                if isinstance(t, (TType, TFn)) or not c:
                    continue
                tmp = self.tmpname("s")
                pre.append(f"auto{'' if isinstance(t, SCALARS) else '&&'} {tmp} = {c};")
                put(N("ctemp", n.line, code=tmp, ty=t, src=n))
                restore.append((put, n))
            c, t = m(e, want)
        finally:
            for put, n in restore:
                put(n)
        if not pre:
            return c, t
        # a scalar result is returned by value (`min(_s1, x)` would otherwise be a reference to the temporary);
        # a place being written through, or a view, keeps its exact type
        lvalue = getattr(self, "lvalue", None)   # unset until the first assignment is compiled
        place = getattr(e, "mut", False) or (lvalue is not None and lvalue[0] is e)
        ret = "auto" if isinstance(t, SCALARS) and not place else "decltype(auto)"
        return f"([&]() -> {ret} {{ {' '.join(pre)} return {c}; }}())", t

    def x_ctemp(self, e, want):
        return e.code, e.ty

    def x_num(self, e, want):
        v = e.value
        if isinstance(v, float) or getattr(e, "is_float", False):
            return repr(float(v)), F64
        if isinstance(want, TF64):      # an integer literal unifies with F64: `x * 2`, `let h: F64 = 0`
            return f"{v}.0", F64
        if isinstance(want, TU64):
            return f"{v}ULL", U64
        if isinstance(want, TI128):
            return f"((__int128){v}LL)", I128
        if isinstance(want, TMod):
            return f"{self.cpp_type(want)}({v}LL)", want
        return (f"{v}LL" if abs(v) > 2**31 - 1 else str(v)), TInt(v, v)

    def x_bool(self, e, want):
        return ("true" if e.value else "false"), BOOL

    def x_char(self, e, want):
        return repr(e.value).replace('"', '\\"') if e.value != "'" else "'\\''", CHAR

    def x_str(self, e, want):
        parts = self.str_parts(e)
        if all(isinstance(p, str) for p in parts):
            return f'string("{"".join(parts)}")', STR
        out = []
        for p in parts:
            if isinstance(p, str):
                if p:
                    out.append(f'string("{p}")')
                continue
            x, spec = p
            c, t = self.expr(x)
            out.append(self.show(c, t, spec, e))
        return "(" + " + ".join(out) + ")", STR

    def str_parts(self, e):
        """Split a literal at its `{expr}` and `{expr:spec}` holes; `\\{` and `\\}` are plain braces."""
        from .parser import Parser, ParseError
        s, parts, buf, i = e.value, [], "", 0
        while i < len(s):
            c = s[i]
            if c == "\\" and i + 1 < len(s):
                buf += s[i + 1] if s[i + 1] in "{}" else s[i:i + 2]
                i += 2
            elif c == "{" and "}" in s[i:]:
                j = s.index("}", i)
                body, _, spec = s[i + 1:j].partition(":")
                try:
                    p = Parser(body)
                    x = p.expr()
                    if not p.at_kind("NEWLINE"):
                        p.err("expected `}`")
                except ParseError as err:
                    self.error(e, f"in `{{{s[i + 1:j]}}}`: {str(err).split(': ', 1)[-1]}")
                for n in walk(x):
                    n.line = e.line
                parts += [buf, (x, spec)]
                buf, i = "", j + 1
            else:
                buf += c
                i += 1
        return parts + [buf]

    def show(self, c, t, spec, node):
        """The text of a value inside `"{x}"`: what `print(x)` writes, or a number under a checked format."""
        if spec:
            # the whole grammar: `{v:W}` width, `{v:0W}` zero-filled, `{x:.D}` / `{x:.Df}` fixed decimals,
            # `{n:x}` hex and `{n:b}` binary (each after an optional width). No alignment, no nested widths.
            m = re.fullmatch(r"(0?)(\d*)(?:\.(\d+)f?|([xb]))?", spec)
            help = "use `:.6f` for an F64, `:04` / `:x` / `:b` for an Int"
            if not m or not (is_int(t) or isinstance(t, TF64)):
                self.error(node, f"`:{spec}` is not a number format {t!r} takes ({help})")
            zero, width = "true" if m.group(1) else "false", m.group(2) or 0
            if m.group(3) is not None:
                return f"cp_fmt_f((double)({c}), {m.group(3)}, {width}, {zero})"
            if not is_int(t):
                self.error(node, f"`:{spec}` does not say how many decimals an F64 gets ({help})")
            if m.group(4):
                return f"cp_fmt_radix({c}, {16 if m.group(4) == 'x' else 2}, {width}, {zero})"
            return f"cp_fmt_i({c}, {width}, {zero})"
        if isinstance(t, TF64):
            self.error(node, "an F64 has no default format: write `{x:.6f}` with the digits you want")
        if isinstance(t, (TU64, TI128)): return f"cp_wide_str({c})"
        if isinstance(t, TInt): return f"to_string((ll)({c}))"
        if isinstance(t, TDom):
            lab = self.dom_label(t.info, c)
            return f"string(1, (char){lab})" if t.info.labels == "char" else f"to_string((ll)({lab}))"
        if isinstance(t, TBool): return f'string(({c}) ? "YES" : "NO")'
        if isinstance(t, TChar): return f"string(1, {c})"
        if isinstance(t, TStr): return c
        if isinstance(t, TMod): return f"to_string((ll)({c}).v)"
        self.error(node, f"a {t!r} cannot be written inside a string; format its parts")

    def x_paren(self, e, want):
        c, t = self.expr(e.a, want)
        if id(e.a) in self.narrowed_exprs:
            self.narrowed_exprs.add(id(e))
        return f"({c})", t

    def x_none(self, e, want):
        if not isinstance(want, TOpt):
            self.error(e, "`none` needs a known ?T type here")
        return f"{self.cpp_type(want)}()", want

    def x_name(self, e, want):
        if e.name == "inf" and not self.lookup("inf"):
            return "CP_INF", TInt(4e18, 4e18)
        if e.name in ("Dir4", "Dir8") and not self.lookup(e.name):
            return e.name, TType(TRt(e.name, [], f"Cp{e.name}"))
        s = self.lookup(e.name)
        if s is None:
            prim = {"Int": INT, "Bool": BOOL, "Char": CHAR, "F64": F64, "U64": U64, "I128": I128}
            if e.name == "PI":
                return "3.14159265358979323846", F64
            if e.name in prim:
                return "", TType(prim[e.name])
            if e.name == "Steps":
                return "", TType(self.lib_type("Steps", [], e))
            self.error(e, f"unknown name {e.name}")
        if s.kind == "type":
            return "", TType(s.ty)
        if s.kind == "genstruct":
            # Bare generic struct with all defaults, e.g. `Rooted(...)`
            return "", TType(self.specialize_struct(s.node, [], e))
        if s.kind == "ctor":
            return self.construct(s, [], e)
        if s.kind == "fn":
            return s.cpp, s.ty
        if s.kind == "genfn":
            tps = ", ".join(s.node.tparams)
            self.error(e, f"`{e.name}` is a generic fn and is only called, as `{e.name}(...)`: its type parameters come "
                          f"from its arguments (`{e.name}<{tps}>(...)` would be two comparisons)")
        # Rule N1: local binding narrowed from ?T to T by a syntactic guard (not when used as lvalue/mut arg)
        lv = getattr(self, "lvalue", None)
        is_lv = (lv is not None and lv[0] is e) or getattr(self, "_raw_name_node", None) is e
        if not is_lv and e.name in self.opt_narrow:
            nsym, inner_t = self.opt_narrow[e.name]
            if nsym is s:
                self.narrowed_exprs.add(id(e))
                return f"({s.cpp}).get()", inner_t
        t = s.ty
        if isinstance(t, TInt) and s.node is not None and id(s.node) in self.var_iv:
            lo, hi = self.var_iv[id(s.node)]
            t = TInt(lo, hi)
        return s.cpp, t

    def x_generic(self, e, want):
        return "", TType(self.resolve_type(N("tname", e.line, name=e.name, args=e.args)))

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
        self.error(e, f"`{unparse(e.a)}?` is inside a branch, a lambda or a comprehension, where leaving the fn early "
                      f"is not in the prototype\n  help: bind it first: `let x = {unparse(e.a)}?`")

    def try_exit(self, node):
        """The C++ that leaves with `none` when a `?` meets one: out of the `do:` block, or out of the fn."""
        ds = getattr(self, "do_stack", [])
        if ds and ds[-1][3] == len(self.fn_ret):
            dcpp, dty, dlabel, _ = ds[-1]
            if not isinstance(dty, TOpt):
                self.error(node, "`?` inside `do:` needs the block to have type ?T")
            return f"{{ {dcpp} = {{}}; goto {dlabel}; }}"
        if not self.fn_ret or not isinstance(self.fn_ret[-1], TOpt):
            self.error(node, "`?` is only allowed in a fn returning ?T")
        return "return {};"

    def lift_tries(self, e, put, undo):
        """`e?` anywhere in a statement's expression passes `none` along: the fn (or `do:` block) gives `none`
        as soon as one is met. Every `?` that is always evaluated is taken out, left to right, into a temporary
        tested before the statement; one under `and` / `or` / `??`, an inline `if` or a lambda is left for x_try."""
        k = e.kind
        def attr(n, name):
            return getattr(n, name), (lambda v: setattr(n, name, v))
        def item(lst, i):
            return lst[i], (lambda v: lst.__setitem__(i, v))
        kids = []
        if k in ("paren", "unop", "unwrap", "try"):
            kids = [attr(e, "a")]
        elif k == "binop" and e.op not in ("and", "or", "??"):
            kids = [attr(e, "a"), attr(e, "b")]
        elif k == "cmp" and len(e.ops) == 1:
            kids = [attr(e, "first"), item(e.rest, 0)]
        elif k == "field":
            kids = [attr(e, "obj")]
        elif k == "index":
            kids = [attr(e, "obj")] + [item(e.idx, i) for i in range(len(e.idx))]
        elif k == "tuple":
            kids = [item(e.items, i) for i in range(len(e.items))]
        elif k == "kwarg":
            kids = [attr(e, "value")]
        elif k == "call":
            kids = ([attr(e.fn, "obj")] if e.fn.kind == "field" else []) + [item(e.args, i) for i in range(len(e.args))]
        for child, setter in kids:
            self.lift_tries(child, setter, undo)
        if k == "try":
            c, t = self.expr(e.a)
            if not isinstance(t, TOpt):
                self.error(e, f"`?` needs a ?T, got {t!r}")
            o = self.tmpname("o")
            self.emit(f"auto {o} = {c};")
            self.emit(f"if (!{o}.has()) {self.try_exit(e)}")
            put(N("ctemp", e.line, code=f"{o}.get()", ty=t.inner, src=e))
            undo.append((put, e))

    def with_tries(self, node, name, run):
        """Run `run()` with every liftable `?` under `node.<name>` taken out first; the tree is restored after."""
        e = getattr(node, name, None)
        if e is None or not any(x.kind == "try" for x in walk([e])):
            return run()
        undo = []
        try:
            self.lift_tries(e, lambda v: setattr(node, name, v), undo)
            return run()
        finally:
            for put, old in reversed(undo):
                put(old)

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
        if (isinstance(want, THeap) or (isinstance(want, TRt) and want.name == "Deque")) and not e.items:
            return f"{self.cpp_type(want)}()", want
        if isinstance(want, TCounter) and not e.items:      # `var c: Counter<K> = []`: every count is 0
            return f"{self.cpp_type(want)}{self.counter_init(want)}", want
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

    def x_maplit(self, e, want):
        """`[k: v, ...]`: a Map (ordered) unless the declared type says HashMap."""
        kt, vt = (want.key, want.val) if isinstance(want, TMap) else (None, None)
        parts = []
        for k, v in e.pairs:
            kc, t = self.expr(k, kt)
            kt = kt or (TInt() if is_int(t) else t)
            kc = self.coerce(kc, t, kt, k)
            vc, t = self.expr(v, vt)
            vt = vt or (TInt() if is_int(t) else t)
            parts.append(f"_m[{kc}] = {self.coerce(vc, t, vt, v)};")
        mt = want if isinstance(want, TMap) else TMap(kt, vt)
        return f"([&]{{ auto _m = {self.set_ctor(mt)}; {' '.join(parts)} return _m; }}())", mt

    def x_unop(self, e, want):
        c, t = self.expr(e.a, want)
        if e.op == "not":
            if not isinstance(t, TBool):
                self.error(e, f"`not` needs Bool, got {t!r}")
            return f"!({c})", BOOL
        if e.op == "~":
            return self.x_bitnot(c, t, e)
        if isinstance(t, (TF64, TI128)):
            return f"(-({c}))", t
        if isinstance(t, TInt):
            lo, hi = iv_neg(t)
            if hi is None:              # -x overflows for the minimum Int only
                return f"cp_sub(0, {c}, {e.line})", self.int_result(e, lo, hi)
            return f"-({c})", self.int_result(e, lo, hi)
        if isinstance(t, TOpt) and isinstance(t.inner, (TInt, TF64)):
            ct = self.cpp_type(t)
            return f"([]({ct} _a){{ return _a.has() ? {ct}(-{('_a.get()' if isinstance(t.inner, TF64) else f'cp_sub(0, _a.get(), {e.line})')}) : {ct}(); }}({c}))", TOpt(TInt() if isinstance(t.inner, TInt) else TF64())
        if isinstance(t, TRt) and t.name == "Pt":
            return f"(-({c}))", t
        if isinstance(t, TMod):
            return f"-({c})", t
        self.error(e, f"cannot negate {t!r}")

    def is_literal(self, x):
        return x.kind == "num" or (x.kind == "unop" and x.op == "-" and x.a.kind == "num") or (x.kind == "paren" and self.is_literal(x.a))

    def x_bitnot(self, c, t, e):
        if not isinstance(t, TInt):
            self.error(e, f"`~` needs an Int, got {t!r}")
        lo, hi = iv_neg(t)
        return f"(~(ll)({c}))", TInt(None if lo is None else lo - 1, None if hi is None else hi - 1)

    def x_binop(self, e, want):
        op = e.op
        if op in ("|", "&", "-", "^") and e.a.kind == "name" and isinstance(getattr(self.lookup(e.a.name), "ty", None), TSet):
            a, at = self.expr(e.a)
            b, bt = self.expr(e.b, at)
            if bt != at:
                self.error(e, f"`{op}` needs two {at!r}, got {bt!r}")
            if self.word_set(at):
                m = {"|": "a.m | b.m", "&": "a.m & b.m", "-": "a.m & ~b.m", "^": "a.m ^ b.m"}[op]
                return f"([](Mask64 a, Mask64 b) {{ Mask64 r; r.m = {m}; return r; }}({a}, {b}))", at
            fn = {"|": "cp_set_union", "&": "cp_set_inter", "-": "cp_set_diff", "^": "cp_set_symdiff"}[op]
            return f"{fn}({a}, {b})", at
        if op in ("|", "&", "-", "^"):
            _, (_, at0) = self.capture(lambda: self.expr(e.a))
            if isinstance(at0, TRt) and at0.name == "Bitset":
                a, at = self.expr(e.a)
                b, bt = self.expr(e.b, at)
                if bt != at:
                    self.error(e, f"`{op}` needs two {at!r}, got {bt!r}")
                return f"(({a}) {op} ({b}))", at
            if isinstance(at0, TSet):
                a, at = self.expr(e.a)
                b, bt = self.expr(e.b, at)
                if bt != at:
                    self.error(e, f"`{op}` needs two {at!r}, got {bt!r}")
                fn = {"|": "cp_set_union", "&": "cp_set_inter", "-": "cp_set_diff", "^": "cp_set_symdiff"}[op]
                return f"{fn}({a}, {b})", at
        if op in ("and", "or"):
            a, at = self.expr(e.a)
            saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
            try:
                self.apply_narrow(self.extract_narrow(e.a, truthy=(op == "and")))
                b, bt = self.expr(e.b)
            finally:
                self.opt_narrow, self.nonempty_narrow = saved_n
            if not (isinstance(at, TBool) and isinstance(bt, TBool)):
                self.error(e, f"`{op}` needs Bool operands, got {at!r} and {bt!r}")
            return f"({a} {'&&' if op == 'and' else '||'} {b})", BOOL
        if op == "??":
            a, at = self.expr(e.a)
            if not isinstance(at, TOpt):
                if id(e.a) in self.narrowed_exprs:
                    # Invariant 4: guarding never breaks existing `x ?? fallback`
                    self.warnings.append(f"line {e.line}: redundant `??` on already-narrowed `{unparse(e.a)}`")
                    b, bt = self.expr(e.b, at)
                    self.coerce(b, bt, at, e.b)
                    return a, at
                self.error(e, f"`??` needs a ?T on the left, got {at!r}")
            b, bt = self.expr(e.b, at.inner)
            b = self.coerce(b, bt, at.inner, e.b)
            tmp = self.tmpname("o")
            return f"([&]{{ auto {tmp} = {a}; return {tmp}.has() ? {tmp}.get() : {self.cpp_type(at.inner)}({b}); }}())", at.inner
        a, at = self.expr(e.a, want if isinstance(want, TMod) else None)
        b, bt = self.expr(e.b, at if isinstance(at, TMod) and op != "**" else None)      # an exponent is an Int
        if isinstance(at, TRange) and isinstance(bt, TRange) and op == "&":
            if at.elem != bt.elem and not (is_int(at.elem) and is_int(bt.elem)):
                self.error(e, f"`&` between ranges of {at.elem!r} and {bt.elem!r}")
            return f"cp_rinter({a}, {b})", at
        if isinstance(at, TRt) and at.name == "Pt" and bt == at and op in ("+", "-"):
            return f"(({a}) {op} ({b}))", at
        if isinstance(at, TRt) and at.name == "Matrix" and isinstance(bt, TRt) and bt.name == "Matrix" and op == "*":
            if at.args[1] != bt.args[0] or at.args[2] != bt.args[2]:
                self.error(e, f"matrix multiply dimension/type mismatch: {at!r} * {bt!r}")
            return f"(({a}) * ({b}))", TRt("Matrix", [at.args[0], bt.args[1], at.args[2]], "CpMat")
        if isinstance(at, TOpt) or isinstance(bt, TOpt):
            if op in ("+", "-"):
                ai = at.inner if isinstance(at, TOpt) else at
                bi = bt.inner if isinstance(bt, TOpt) else bt
                if (is_int(ai) and is_int(bi)) or (isinstance(ai, TF64) and (isinstance(bi, TF64) or (is_int(bi) and self.is_literal(e.b)))) or (isinstance(bi, TF64) and is_int(ai) and self.is_literal(e.a)):
                    bt_base = TF64() if (isinstance(ai, TF64) or isinstance(bi, TF64)) else TInt()
                    ot = TOpt(bt_base)
                    octy = self.cpp_type(ot)
                    ac = a if isinstance(at, TOpt) else f"{octy}({self.coerce(a, at, bt_base, e.a)})"
                    bc = b if isinstance(bt, TOpt) else f"{octy}({self.coerce(b, bt, bt_base, e.b)})"
                    if isinstance(bt_base, TF64):
                        comb = f"cp_fchk(_a.get() {op} _b.get(), {e.line})"
                    else:
                        fn = "cp_add" if op == "+" else "cp_sub"
                        comb = f"{fn}(_a.get(), _b.get(), {e.line})"
                    return f"([&](auto _a, auto _b) -> {octy} {{ return (_a.has() && _b.has()) ? {octy}({comb}) : {octy}(); }}({ac}, {bc}))", ot
            self.error(e, f"`{op}` on an optional ({at!r}, {bt!r}); unwrap with `!`, `??` or `if let` first")
        WIDE = (TF64, TU64, TI128)
        if isinstance(at, WIDE) or isinstance(bt, WIDE) or (isinstance(want, WIDE) and is_int(at) and is_int(bt)
                                                             and self.is_literal(e.a) and self.is_literal(e.b)):
            # F64, U64 and I128 never mix silently with Int: only an integer literal unifies with them
            wt = at if isinstance(at, WIDE) else bt if isinstance(bt, WIDE) else want
            if isinstance(wt, TF64) and op in ("+", "-", "*", "/", "**"):
                # an Int next to an F64 widens, as an integer literal does. `Int / Int` is still integer
                # division (neither side is an F64), and an F64 never narrows back without floor/round/trunc.
                if is_int(at):
                    a, at = f"((double)({a}))", wt
                if is_int(bt):
                    b, bt = f"((double)({b}))", wt
                if op == "**" and isinstance(at, TF64) and isinstance(bt, TF64):
                    return f"cp_fchk(std::pow({a}, {b}), {e.line})", wt
            if not isinstance(at, WIDE):
                a, at = self.expr(e.a, wt)
                a = self.coerce(a, at, wt, e.a)
            if op == "**" and isinstance(wt, TI128) and is_int(bt):
                return f"cp_ipow128({a}, {b}, {e.line})", wt
            if op in ("<<", ">>") and isinstance(wt, (TU64, TI128)) and is_int(bt):
                return f"(({a}) {op} ({b}))", wt
            if not isinstance(bt, WIDE):
                b, bt = self.expr(e.b, wt)
                b = self.coerce(b, bt, wt, e.b)
            if type(bt) is not type(wt) or type(at) is not type(wt):
                self.error(e, f"`{op}` between {at!r} and {bt!r}; convert one side explicitly")
            ops = {TF64: "+-*/", TU64: "+-*/%&|^", TI128: "+-*/%"}[type(wt)]
            if op not in ops or len(op) != 1:
                self.error(e, f"`{op}` is not defined on {wt!r}")
            if op in "/%" and not isinstance(wt, TF64):
                return f"cp_wdiv{'mod' if op == '%' else ''}({a}, {b}, {e.line})", wt
            return f"(({a}) {op} ({b}))", wt
        if isinstance(at, TInt) and isinstance(bt, TMod):
            a, at = f"{self.cpp_type(bt)}({a})", bt
        if isinstance(at, TMod):
            if isinstance(bt, TInt) and op != "**":
                b = f"{self.cpp_type(at)}({b})"
            elif op == "**":
                return f"({a}).pow({b})", at
            if op not in "+-*/":
                self.error(e, f"`{op}` is not defined on Mod")
            return f"({a} {op} {b})", at
        if isinstance(at, TDom) and getattr(at.info, "subset_of", None) is not None:
            # Subset<D> is a set: union, intersection, difference, symmetric difference
            if not (isinstance(bt, TDom) and bt.info is at.info) or op not in ("|", "&", "-", "^"):
                self.error(e, f"`{op}` between {at!r} and {bt!r}; sets support `s | t`, `s & t`, `s - t`, `s ^ t`, "
                              f"and `s.with(x)` / `s.without(x)` for one element")
            return (f"(({a}) & ~({b}))" if op == "-" else f"(({a}) {op} ({b}))"), at
        if isinstance(at, TDom):
            info = at.info
            if op in "+-" and isinstance(bt, TInt):
                # a step that can leave the domain gives an optional (12 §1.3): `if let j = i - k`,
                # `i + k ?? D.last`, `(i + k)!`. The sum is taken in 128 bits, so a huge k never wraps back in.
                return f"cp_step({a}, {b}, {'true' if op == '-' else 'false'}, {info.len_cpp})", TOpt(at)
            if op == "-" and isinstance(bt, TDom) and bt.info is info:
                return f"((ll)({a}) - ({b}))", TInt(None if info.len_iv[1] is None else -info.len_iv[1],
                                                     None if info.len_iv[1] is None else info.len_iv[1])
            self.error(e, f"`{op}` between {at!r} and {bt!r}; domain values support `i + k` and `i - k` (a ?{at!r}) and `i - j` (an Int)")
        if isinstance(bt, TDom):
            self.error(e, f"`{op}` between {at!r} and {bt!r}; use `{unparse(e.b)}.ord` for its position")
        if isinstance(at, TChar) and isinstance(bt, TInt) and op in "+-":
            # Char + Int -> Char, the inverse of Char - Char; the result must still be one ASCII byte
            return f"cp_char_step({a}, {'-' if op == '-' else ''}(ll)({b}), {e.line})", CHAR
        if isinstance(at, TChar) and isinstance(bt, TChar) and op == "-":
            return f"(ll)({a} - {b})", TInt(-255, 255)
        if isinstance(at, TStr) and op == "+" and isinstance(bt, (TStr, TChar)):
            return f"({a} + {b})", STR
        if isinstance(at, TStr) and op == "*" and is_int(bt):
            return f"cp_str_repeat({a}, {b}, {e.line})", STR
        if isinstance(at, TList) and isinstance(bt, TList) and op == "+":
            # xs + ys: a new list, xs then ys
            if is_int(at.elem) and is_int(bt.elem):
                return f"cp_concat({a}, {b})", TList(TInt(*iv_join(at.elem, bt.elem)))
            if at.elem != bt.elem:
                self.error(e, f"`+` joins two lists of one element type, got {at!r} and {bt!r}")
            return f"cp_concat({a}, {b})", at
        if not (isinstance(at, TInt) and isinstance(bt, TInt)):
            self.error(e, f"`{op}` needs Int operands, got {at!r} and {bt!r}")
        if op in "+-*":
            # Operands that fit in 32 bits can both be C++ `int` (literals are); when the
            # result may not fit, compute it in 64 bits.
            small = lambda t: None not in (t.lo, t.hi) and -2**31 <= t.lo and t.hi < 2**31
            lo, hi = {"+": iv_add, "*": iv_mul}[op](at, bt) if op != "-" else iv_add(at, TInt(*iv_neg(bt)))
            if small(at) and small(bt) and not small(TInt(lo, hi)):
                a = f"(ll){a}"
        if op in "+-*":
            rt = self.int_result(e, lo, hi)
            if None in (lo, hi):        # not proven to fit: a local build checks it
                return f"cp_{ {'+': 'add', '-': 'sub', '*': 'mul'}[op] }({a}, {b}, {e.line})", rt
            return f"({a} {op} {b})", rt
        if op in ("/", "%"):
            nz = e.b.kind == "name" and getattr(self.lookup(e.b.name), "nonzero", False)
            if not nz and bt.lo is not None and bt.hi is not None and bt.lo <= 0 <= bt.hi:
                self.error(e, f"`{unparse(e.b)}` may be 0 in `{unparse(e)}`")
            plain = at.lo is not None and at.lo >= 0 and bt.lo is not None and bt.lo > 0
            pos_b = bt.lo is not None and bt.lo > 0
            if op == "/":
                c = f"({a} / {b})" if plain else (f"cp_ediv_pos({a}, {b})" if pos_b else f"cp_ediv({a}, {b}, {e.line})")
                if pos_b and None not in (at.lo, at.hi, bt.hi):
                    qs = [at.lo // bt.lo, at.lo // bt.hi, at.hi // bt.lo, at.hi // bt.hi]
                    return c, TInt(min(qs), max(qs))
                return c, TInt()
            c = f"({a} % {b})" if plain else (f"cp_emod_pos({a}, {b})" if pos_b else f"cp_emod({a}, {b}, {e.line})")
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
                # the left operand is widened first: `1 << 40` on a C++ `int` is undefined (backlog W5)
                return f"((ll)({a}) << {b})", self.int_result(e, 0, at.hi << bt.hi)
            return f"cp_shl({a}, {b}, {e.line})", TInt()
        if op == ">>":
            rt = TInt(0 if (at.lo or 0) >= 0 else None, at.hi)
            if bt.lo is None or bt.lo < 0 or bt.hi is None or bt.hi > 63:
                return f"cp_shr({a}, {b}, {e.line})", rt
            return f"({a} >> {b})", rt
        self.error(e, f"unknown operator {op}")

    def x_cmp(self, e, want):
        parts = []
        want_first = None
        if e.ops[0] in ("in", "not in") and e.first.kind == "tuple":
            rt0 = self.capture(lambda: self.expr(e.rest[0]))[1][1]
            want_first = rt0.elem if isinstance(rt0, TSet) else rt0.key if isinstance(rt0, TMap) else None
        if e.first.kind == "none" and len(e.ops) == 1 and e.ops[0] in ("==", "!="):
            # Support `none == x` / `none != x` symmetrically with `x == none` / `x != none`
            rn = e.rest[0]
            saved_raw = getattr(self, "_raw_name_node", None)
            if rn.kind == "name" and rn.name in self.opt_narrow:
                self._raw_name_node = rn
            try:
                rc, rt = self.expr(rn)
            finally:
                self._raw_name_node = saved_raw
            if not isinstance(rt, TOpt):
                self.error(e, f"`none {e.ops[0]}` needs a ?T and == / !=")
            return f"({'!' if e.ops[0] == '==' else ''}({rc}).has())", BOOL
        saved_raw = getattr(self, "_raw_name_node", None)
        if len(e.ops) == 1 and e.ops[0] in ("==", "!=") and e.rest[0].kind == "none" \
                and e.first.kind == "name" and e.first.name in self.opt_narrow:
            # Invariant 4: redundant `x != none` / `x == none` inside an already-guarded block
            self._raw_name_node = e.first
        try:
            left_c, left_t = self.expr(e.first, want_first)
        finally:
            self._raw_name_node = saved_raw
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
                WIDE = (TF64, TU64, TI128)
                if isinstance(lt, WIDE) != isinstance(rt, WIDE) or (isinstance(lt, WIDE) and type(lt) is not type(rt)):
                    if isinstance(lt, WIDE) and is_int(rt):
                        rc, rt = self.coerce(rc, rt, lt, rn), lt
                    elif isinstance(rt, WIDE) and is_int(lt) and len(parts) == 0:
                        left_c, lt = self.coerce(left_c, lt, rt, e.first), rt
                    else:
                        self.error(e, f"comparing {lt!r} with {rt!r}; convert one side explicitly")
                if (isinstance(lt, TMod) or isinstance(rt, TMod)) and op not in ("==", "!="):
                    self.error(e, f"`{op}` on a Mod ({lt!r} vs {rt!r}): a residue has no ordering; "
                                  f"compare `.val` if you mean the representative in 0..P")
                if op in ("==", "!=") and isinstance(lt, TOpt) != isinstance(rt, TOpt):
                    # ?T == T: `none` equals no value, so this is "present and equal"
                    o_c, o_t, x_c, x_t = (left_c, lt, rc, rt) if isinstance(lt, TOpt) else (rc, rt, left_c, lt)
                    if o_t.inner != x_t and not (is_int(o_t.inner) and is_int(x_t)):
                        self.error(e, f"comparing {lt!r} with {rt!r}")
                    eq = f"([&](const auto& _o, const auto& _x) {{ return _o.has() && _o.get() == _x; }}({o_c}, {x_c}))"
                    parts.append(eq if op == "==" else f"!{eq}")
                    left_c, left_t = rc, rt
                    continue
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

    def lambda2(self, lam, ta, tb, tr):
        """Compile `(a, b) => body` to a C++ lambda taking (ta, tb) and returning tr."""
        if lam.kind != "lambda" or len(lam.params) != 2:
            self.error(lam, "expected a two-parameter lambda `(a, b) => ...`")
        self.scopes.append({})
        pa, pb = self.fresh("a"), self.fresh("b")
        self.bind_lambda_param(lam.params[0], pa, ta, lam)
        self.bind_lambda_param(lam.params[1], pb, tb, lam)
        bc, bt = self.expr(lam.body, tr)
        bc = self.coerce(bc, bt, tr, lam.body)
        self.scopes.pop()
        return (f"[&](const {self.cpp_type(ta)}& {pa}, const {self.cpp_type(tb)}& {pb}) -> {self.cpp_type(tr)} "
                f"{{ return {bc}; }}")

    def binary_fn(self, f, t):
        """C++ for a two-argument operation on `t`: a lambda, an operator (`+`), or a fn by name (`min`, `gcd`)."""
        if f.kind == "name":
            a, b = N("name", f.line, name="_a"), N("name", f.line, name="_b")
            f = N("lambda", f.line, params=[N("pname", f.line, name="_a"), N("pname", f.line, name="_b")],
                  body=N("call", f.line, fn=f, args=[a, b]), op=f.name)
        return self.lambda2(f, t, t, t)

    def monoid(self, f, t):
        """(identity, operation) in C++ for an operation given by name where a Monoid<T> is expected: `+` is
        Sum, `*` Prod, `min` / `max` over ?T start from none (std.algebra `opt`), over Int from the far end."""
        name = getattr(f, "op", None) or (f.name if f.kind == "name" else None)
        ct = self.cpp_type(t)
        if name in ("min", "max") and isinstance(t, TOpt):
            ident = f"{ct}()"
        elif name in ("min", "max") and is_int(t):
            ident = "LLONG_MAX" if name == "min" else "-LLONG_MAX"
        elif name in FOLD_IDENTITY and name != "-" and (is_int(t) or isinstance(t, TMod)) and not (f.kind == "name" and self.lookup(name)):
            ident = f"{ct}({FOLD_IDENTITY[name]})"
        else:
            self.error(f, f"`{unparse(f) if f.kind == 'name' else 'this operation'}` has no identity the library knows for {t!r}; "
                          f"give one: `SegTree<D, T>(identity, (a, b) => ...)`")
        return ident, self.binary_fn(f, t)

    def membership(self, xc, xt, rn, e):
        rc, rt = self.expr(rn)
        if isinstance(rt, TType) and isinstance(rt.t, TDom):
            info = rt.t.info
            if isinstance(xt, TDom):
                return "true"
            if not isinstance(xt, TInt):
                self.error(e, f"`in {info.name}` needs an Int, got {xt!r}")
            return f"((ll)({xc}) - {info.lo_cpp} >= 0 && (ll)({xc}) - {info.lo_cpp} < {info.len_cpp})"
        if isinstance(rt, TDom) and getattr(rt.info, "subset_of", None) is not None:
            if not (isinstance(xt, TDom) and xt.info is rt.info.subset_of):
                self.error(e, f"`in` a {rt.info.name} needs a {rt.info.subset_of.name}, got {xt!r}")
            return f"((({rc}) >> ({xc})) & 1)"
        if isinstance(rt, TRange):
            return f"([&](ll _x, Rng _r){{ return _r.lo <= _x && _x < _r.hi; }}({xc}, {rc}))"
        if isinstance(rt, (TMap, TSet)):
            kt = rt.key if isinstance(rt, TMap) else rt.elem
            return f"({rc}.count({self.coerce(xc, xt, kt, e)}) > 0)"
        if isinstance(rt, TRt) and rt.name in ("Marks", "Bitset"):
            ic = self.index_into(e.first, rt.args[0].info, e)
            return f"{rc}.has({ic})"
        if isinstance(rt, (TList, TArr)):
            return f"(find({rc}.begin(), {rc}.end(), {xc}) != {rc}.end())"
        self.error(e, f"cannot test membership in {rt!r}")

    def counter_init(self, t):
        return f"({t.key.info.len_cpp})" if isinstance(t.key, TDom) else "()"

    def range_end(self, x, e, hi, incl=False):
        """One end of a range: (C++ boundary ordinal, the domain the range runs over, or None for Int)."""
        c, t = self.expr(x)
        if isinstance(t, TDom):
            if t.info.of is not None:           # a boundary delimits the positions on either side of it
                if incl:
                    self.error(e, f"`..=` ends at a position; a range up to the boundary `{unparse(x)}` is written `..`")
                return f"(ll)({c})", t.info.of
            return f"(ll)({c})" + (" + 1" if hi and incl else ""), t.info
        if isinstance(t, TInt):
            return f"(ll)({c})" + (" + 1" if hi and incl else ""), None
        self.error(e, f"a range can't start or end at a {t!r}")

    def x_range(self, e, want):
        """`l..r`, `l..=r`, `..r`, `..=r`, `l..`: a Range<I> is the positions between two boundaries
        (plan/19 §4.1), so each end may be a position or a boundary of the same domain."""
        lo = self.range_end(e.lo, e, False) if e.lo is not None else None
        hi = self.range_end(e.hi, e, True, e.incl) if e.hi is not None else None
        info = (lo or hi)[1]
        if lo is not None and hi is not None and lo[1] is not hi[1]:
            self.error(e, f"range between {self.expr(e.lo)[1]!r} and {self.expr(e.hi)[1]!r}")
        if info is None:
            if lo is None:
                return f"Rng{{0, {hi[0]}}}", TRange(INT)
            if hi is None:
                self.error(e, "an open range of Ints needs an upper bound")
            return f"Rng{{{lo[0]}, {hi[0]}}}", TRange(INT)
        return f"Rng{{{lo[0] if lo else '0'}, {hi[0] if hi else f'(ll){info.len_cpp}'}}}", TRange(TDom(info))

    def x_unwrap(self, e, want):
        c, t = self.expr(e.a)
        if not isinstance(t, TOpt):
            if id(e.a) in self.narrowed_exprs:
                # Synthesis #3 Invariant 4: guarding never breaks existing `!` (redundant unwrap -> warning, not error)
                self.warnings.append(f"line {e.line}: redundant `!` on already-narrowed `{unparse(e.a)}`")
                return c, t
            self.error(e, f"`!` on a non-optional {t!r}")
        self.note_check(e, f"`{unparse(e.a)}!` is not none")
        spine = list(self.place_spine(e.a))
        if not isinstance(t.inner, SCALARS) and spine[-1].kind == "name" and self.cpp_type(t).startswith("Opt<"):
            # `state!.h[p] = cost`, `xs!.push(v)`: `x!` of a place is the value in place, not a copy of it
            return f"opt_place({c}, {e.line})", t.inner
        return f"opt_unwrap({c}, {e.line})", t.inner

    def place_spine(self, t):
        """The nodes a write to `t` goes through: `t` itself and each object it is a field, element or `!` of."""
        while t is not None:
            yield t
            if t.kind not in ("paren", "unwrap", "index", "field"):
                break
            t = t.a if t.kind in ("paren", "unwrap") else t.obj

    def x_ifexpr(self, e, want):
        if getattr(e, "is_const", False):
            cv = self.eval_const(e.cond)
            if not isinstance(cv, bool):
                self.error(e, f"`if const` condition must be Bool, got {type(cv).__name__}")
            return self.expr(e.a if cv else e.b, want)
        cc, ct = self.expr(e.cond)
        if not isinstance(ct, TBool):
            self.error(e, f"condition must be Bool, got {ct!r}")
        if want is None and "none" in (e.a.kind, e.b.kind) and e.a.kind != e.b.kind:
            # `if ok: x else: none` is a ?T of the other branch
            other = e.b if e.a.kind == "none" else e.a
            saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
            try:
                self.apply_narrow(self.extract_narrow(e.cond, truthy=(other is e.a)))
                ot = self.capture(lambda: self.expr(other))[1][1]
            finally:
                self.opt_narrow, self.nonempty_narrow = saved_n
            want = ot if isinstance(ot, TOpt) else TOpt(TInt() if is_int(ot) else ot)
        saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
        try:
            self.apply_narrow(self.extract_narrow(e.cond, truthy=True))
            a, at = self.expr(e.a, want)
        finally:
            self.opt_narrow, self.nonempty_narrow = saved_n
        saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
        try:
            self.apply_narrow(self.extract_narrow(e.cond, truthy=False))
            b, bt = self.expr(e.b, want or at)
        finally:
            self.opt_narrow, self.nonempty_narrow = saved_n
        t = want or at
        if is_int(at) and is_int(bt) and want is None:
            t = TInt(*iv_join(at, bt))
        return f"({cc} ? {self.coerce(a, at, t, e.a)} : {self.coerce(b, bt, t, e.b)})", t

    def x_ifletexpr(self, e, want):
        """`if let p = opt: a else: b` as a value: `a` sees the unwrapped p."""
        vc, vt = self.expr(e.value)
        if not isinstance(vt, TOpt):
            self.error(e, f"`if let` unwraps an optional, got {vt!r}")
        o = self.tmpname("o")
        self.scopes.append({})
        try:
            (lines, (a, at)) = self.capture(lambda: (self.bind_pattern(e.pat, f"{o}.get()", vt.inner, e), self.expr(e.a, want))[1])
        finally:
            self.scopes.pop()
        b, bt = self.expr(e.b, want or at)
        t = want or at
        if is_int(at) and is_int(bt) and want is None:
            t = TInt(*iv_join(at, bt))
        binds = " ".join(l.strip() for l in lines)
        ct = self.cpp_type(t)
        return (f"([&]() -> {ct} {{ const auto {o} = {vc}; if ({o}.has()) {{ {binds} return {self.coerce(a, at, t, e.a)}; }} "
                f"return {self.coerce(b, bt, t, e.b)}; }}())"), t

    def x_index(self, e, want):
        if e.obj.kind == "field" and e.obj.name == "raw" and len(e.idx) == 1:
            # a.raw[i]: 0-based Int indexing of a 1-D array, whatever its domain (checked in a local run)
            rc, rt = self.expr(e.obj.obj)
            if isinstance(rt, TArr) and len(rt.dims) == 1:
                ic, it = self.expr(e.idx[0])
                if not is_int(it):
                    self.error(e, f"`.raw[i]` takes an Int position, got {it!r}")
                self.note_check(e, "raw index in range")
                return f"{rc}[(size_t)chk_dom({ic}, (ll){rc}.size(), {e.line})]", rt.elem
        oc, ot = self.expr(e.obj)
        if isinstance(ot, TType) and isinstance(ot.t, TDom) and len(e.idx) == 1 and e.idx[0].kind == "range":
            # D[..k], D[l..r], etc.: clamped sub-range of domain D
            info = ot.t.info
            ix = e.idx[0]
            lo_c = "0" if ix.lo is None else f"max(0LL, min((ll){info.len_cpp}, (ll)({self.expr(ix.lo)[0]})))"
            if ix.hi is None:
                hi_c = f"(ll){info.len_cpp}"
            else:
                hc, _ = self.expr(ix.hi)
                hi_c = f"max(0LL, min((ll){info.len_cpp}, (ll)({hc}){' + 1' if ix.incl else ''}))"
            return f"([&](ll _lo, ll _hi){{ return Rng{{_lo, max(_lo, _hi)}}; }}({lo_c}, {hi_c}))", TRange(ot.t)
        if isinstance(ot, TRt) and ot.name == "Matrix" and len(e.idx) == 2:
            ri = self.index_into(e.idx[0], ot.args[0].info, e)
            ci = self.index_into(e.idx[1], ot.args[1].info, e)
            return f"{oc}.at({ri}, {ci})", ot.args[2]
        if isinstance(ot, TMap):
            kc, kt = self.expr(e.idx[0], ot.key)
            kc = self.coerce(kc, kt, ot.key, e.idx[0])
            lv = getattr(self, "lvalue", None)
            if lv is not None and lv[0] is e:
                # `m[k] = v` inserts. `m[k] f= v` folds v into the entry, and a missing key starts from the
                # identity of f (decisions.md section 12): one rule for `+=` (counting), `min=` and the rest.
                f = lv[1][:-1]
                if lv[1] == "=" or isinstance(ot.val, TOpt):    # a missing ?T entry starts as `none`
                    return f"{oc}[{kc}]", ot.val
                if f not in FOLD_IDENTITY or not (is_int(ot.val) or isinstance(ot.val, TMod)):
                    self.no_identity(e, f)
                if FOLD_IDENTITY[f] == "0":
                    return f"{oc}[{kc}]", ot.val
                return f"cp_mslot({oc}, {kc}, {self.cpp_type(ot.val)}({FOLD_IDENTITY[f]}))", ot.val
            self.note_check(e, f"key `{unparse(e.idx[0])}` is in the map")
            return f"cp_mget({oc}, {kc}, {e.line})", ot.val
        if isinstance(ot, TCounter):
            kc, kt = self.expr(e.idx[0], ot.key)
            kc = self.coerce(kc, kt, ot.key, e.idx[0])
            lv = getattr(self, "lvalue", None)
            if lv is not None and lv[0] is e:
                return f"{oc}.at({kc})", INT
            return f"{oc}.get({kc})", INT
        if isinstance(ot, TList):
            if len(e.idx) != 1:
                self.error(e, "lists take one index")
            if e.idx[0].kind == "range":
                rc, rt = self.expr(e.idx[0])
                if not isinstance(rt, TRange):
                    self.error(e, f"list slice needs a range, got {rt!r}")
                el = 'int' if self.narrow(ot.elem) else ('uint8_t' if isinstance(ot.elem, TBool) else self.cpp_type(ot.elem))
                lo_c = "0" if e.idx[0].lo is None else f"(ll)({self.expr(e.idx[0].lo)[0]})"
                hi_c = f"(ll){oc}.size()" if e.idx[0].hi is None else f"((ll)({self.expr(e.idx[0].hi)[0]}){' + 1' if e.idx[0].incl else ''})"
                vt = TArr([TDom(DomInfo("slice", "sl", "0", f"({hi_c} - {lo_c})", (0, 0), (0, None), anon=True))], ot.elem)
                vt.view = "slice"
                return f"cp_slice({oc}, Rng{{{lo_c}, {hi_c}}}, {e.line})", vt
            ic, it = self.expr(e.idx[0])
            if not is_int(it):
                self.error(e, f"list index must be Int, got {it!r}")
            self.note_check(e, f"`{unparse(e)}` in bounds")
            lv = getattr(self, "lvalue", None)
            if self.narrow(ot.elem) and not (lv is not None and lv[0] is e):
                return f"(ll)cp_at({oc}, {ic}, {e.line})", ot.elem
            return f"cp_at({oc}, {ic}, {e.line})", ot.elem
        if isinstance(ot, TRt) and ot.name == "Deque":
            if len(e.idx) != 1:
                self.error(e, "a Deque takes one index")
            ic = self.str_pos(e.idx[0])
            self.note_check(e, f"`{unparse(e)}` in bounds")
            return f"cp_at({oc}, {ic}, {e.line})", ot.elem
        if isinstance(ot, TStr):
            # a Str is a flat run of ASCII bytes at positions 0..len: `s[i]` is a Char, `s[l..r]` a Str
            if len(e.idx) != 1:
                self.error(e, "a Str takes one index")
            ix = e.idx[0]
            if ix.kind == "range":
                lo = "0" if ix.lo is None else self.str_pos(ix.lo)
                hi = "-1" if ix.hi is None else self.str_pos(ix.hi)
                self.note_check(e, f"`{unparse(e)}` in bounds")
                return f"cp_str_slice({oc}, {lo}, {hi}, {'true' if ix.incl else 'false'}, {e.line})", STR
            ic = self.str_pos(ix)
            self.note_check(e, f"`{unparse(e)}` in bounds")
            return f"cp_at({oc}, {ic}, {e.line})", CHAR
        if not isinstance(ot, TArr):
            self.error(e, f"cannot index {ot!r}")
        view = getattr(ot, "view", None)
        if len(e.idx) == 1 and len(ot.dims) == 1 and e.idx[0].kind in ("range", "name", "field", "paren", "call"):
            _, (_, it) = self.capture(lambda: self.expr(e.idx[0])) if e.idx[0].kind != "range" else (None, (None, None))
            if e.idx[0].kind == "range" or isinstance(it, TRange):
                # a[s]: Array.at(Range) lends the window as a slice, indexed by the same positions (plan/19 §4.2)
                rc, rt = self.expr(e.idx[0])
                base = ot.dims[0].info
                # `l..=r` between positions, or `x..y` between boundaries (v.l, v.r): both are stored as two boundaries
                if not (isinstance(rt, TRange) and isinstance(rt.elem, TDom) and rt.elem.info is base):
                    self.error(e, f"a slice of {ot!r} takes a range of {ot.dims[0]!r}, got {rt!r}")
                if isinstance(ot.elem, TBool):
                    self.error(e, "slices of a Bool array are not in the prototype")
                self.idx_kind[id(e)] = "slice"
                st = TArr(ot.dims, ot.elem)
                st.view = "slice"
                return f"cp_slice({oc}, {rc}, {e.line})", st
        if len(e.idx) == 1 and len(ot.dims) > 1:
            pc, pt = self.capture(lambda: self.expr(e.idx[0]))[1]
            if isinstance(pt, TTuple) and len(pt.types) == len(ot.dims) and all(
                    isinstance(t, TDom) and t.info is d.info for t, d in zip(pt.types, ot.dims)):
                # g[p] with p: (Row, Col): a[i, j] is sugar for a[(i, j)], so a tuple indexes directly
                pc, pt = self.expr(e.idx[0])
                flat = None
                for k, d in enumerate(ot.dims):
                    comp = f"_p.{self.field_cpp(pt.names[k])}"
                    flat = comp if flat is None else f"({flat}) * {d.info.len_cpp} + ({comp})"
                lv = getattr(self, "lvalue", None)
                cast = "(ll)" if self.narrow(ot.elem) and not (lv is not None and lv[0] is e) else ""
                return f"([&](const auto& _p) -> decltype(auto) {{ return ({cast}{oc}[{flat}]); }}({pc}))", ot.elem
        if len(e.idx) == 1 and len(ot.dims) > 1:
            self.error(e, f"{ot!r} is indexed as g[{', '.join(d.info.name.lower()[:1] or 'i' for d in ot.dims)}]; for a whole row use .row(...)")
        if len(e.idx) != len(ot.dims):
            self.error(e, f"{ot!r} needs {len(ot.dims)} indices, got {len(e.idx)}")
        flat = None
        for ix, d in zip(e.idx, ot.dims):
            ic = self.index_into(ix, d.info, e)
            flat = ic if flat is None else f"({flat}) * {d.info.len_cpp} + ({ic})"
        if view == "slice":
            # a logic check, not a memory check: the position is valid for the base array whatever the window is
            self.note_check(e, f"`{show(e.idx[0])}` is inside the slice `{show(e.obj)}`")
            flat = f"cp_win({flat}, {oc}.lo, {oc}.hi, {e.line})"
        lv = getattr(self, "lvalue", None)
        if self.narrow(ot.elem) and not (lv is not None and lv[0] is e):
            return f"(ll){oc}[{flat}]", ot.elem
        return f"{oc}[{flat}]", ot.elem

    def index_into(self, ix, info, e):
        ic, it = self.expr(ix)
        user = getattr(info, "user", None)
        if user is not None:
            if it != user:
                self.error(ix, f"index of type {it!r} into array over {info.name}")
            c, rt = self.to_idx(info, ic, e)
            if rt.lo is not None and rt.hi is not None and rt.lo >= 0 and info.len_iv[0] is not None and rt.hi < info.len_iv[0]:
                return f"(int)({c})"
            # the real checker proves `to_idx` returns 0..K from its declared range; the prototype checks it
            self.note_check(e, f"`{show(ix)}.to_idx` is below {info.name}.K")
            return f"chk_dom({c}, {info.len_cpp}, {e.line})"
        if getattr(info, "enum_type", None) is not None:
            if it != info.enum_type:
                self.error(ix, f"index of type {it!r} into array over {info.name}")
            return f"cp_eidx_{info.enum_type.cpp}({ic})"
        if getattr(info, "prod_tuple", None) is not None:
            pt = info.prod_tuple
            if it != pt:
                self.error(ix, f"index of type {it!r} into array over {info.name}")
            flat = None
            for fnm, ft in zip(pt.names, pt.types):
                fc = f"({ic}).{self.field_cpp(fnm)}"
                flat = fc if flat is None else f"(({flat}) * {ft.info.len_cpp} + ({fc}))"
            return flat
        if isinstance(it, TDom):
            if it.info is info:
                return ic
            if getattr(info, "base", None) is it.info:
                return f"(({ic}) + 1)"
            if info.of is it.info or it.info.of is info:
                self.error(ix, f"index of type {it!r} into array over {info.name}; use `.l` / `.r` for the boundaries on either side of a position")
            self.error(ix, f"index of type {it!r} into array over {info.name}")
        if isinstance(it, TBool) and getattr(info, "is_bool", False):
            return f"(int)({ic})"
        if isinstance(it, TChar) and info.anon:
            return f"chk_dom((ll)(unsigned char)({ic}) - {info.lo_cpp}, {info.len_cpp}, {e.line})"
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
        if isinstance(it, TOpt) and isinstance(it.inner, TDom):
            self.error(ix, f"`{unparse(ix)}` is a {it!r}: it may step outside {it.inner!r}. Bind it with "
                           f"`if let j = {unparse(ix)}`, give the edge a value with `??`, or assert with `!`")
        self.error(ix, f"cannot index with {it!r}")

    def str_pos(self, x):
        c, t = self.expr(x)
        if not is_int(t):
            self.error(x, f"a Str position is an Int in 0..len, got {t!r}")
        return c

    def x_field(self, e, want):
        oc, ot = self.expr(e.obj)
        f = e.name
        if isinstance(ot, TTuple) and f in ("adj4", "adj8") and len(ot.types) == 2 and all(isinstance(t, TDom) for t in ot.types):
            # p.adj4 / p.adj8: the in-grid neighbours of a (Row, Col) point, in row-major order
            (r, c), (rn, cn) = ot.types, [self.field_cpp(n) for n in ot.names]
            steps = "{-1,0},{0,-1},{0,1},{1,0}" if f == "adj4" else "{-1,-1},{-1,0},{-1,1},{0,-1},{0,1},{1,-1},{1,0},{1,1}"
            return (f"([&](const {ot.cpp}& _p) {{ vector<{ot.cpp}> _out; static const int _d[][2] = {{{steps}}}; "
                    f"for (auto& _s : _d) {{ ll _r = (ll)_p.{rn} + _s[0], _c = (ll)_p.{cn} + _s[1]; "
                    f"if (_r >= 0 && _r < (ll){r.info.len_cpp} && _c >= 0 && _c < (ll){c.info.len_cpp}) {{ {ot.cpp} _q; _q.{rn} = (int)_r; _q.{cn} = (int)_c; _out.push_back(_q); }} }} "
                    f"return _out; }}({oc}))"), TList(ot)
        if isinstance(ot, TArr) and getattr(ot, "compressed", False) and f == "D":
            return "", TType(ot.dims[0])
        if isinstance(ot, TMod) and f == "inv":
            return f"({self.cpp_type(ot)}(1) / ({oc}))", ot
        if isinstance(ot, TMod) and f == "val":
            return f"(ll)(({oc}).v)", TInt(0, ot.m - 1)
        if isinstance(ot, (TList, TStr)) and f == "is_empty":
            return f"{oc}.empty()", BOOL
        if isinstance(ot, TTree) and f == "Pre":
            return "", TType(self.tree_pre(ot))
        if isinstance(ot, TTree) and f == "preorder":
            return f"{oc}.order", TList(ot.dom)
        if isinstance(ot, TTree) and f == "kids":       # kids[v]: the children of v
            return f"{oc}.kids", TArr([ot.dom], TList(ot.dom))
        if isinstance(ot, TTree) and f == "root":
            return f"{oc}.order[0]", ot.dom
        if isinstance(ot, TDsu) and f in ("count", "components"):       # the number of sets
            return f"(ll){oc}.parts", TInt(0, ot.dom.info.len_iv[1])
        if isinstance(ot, TSeg) and f == "all":         # the fold of every position
            return f"{oc}.all()", ot.elem
        if isinstance(ot, TDom) and f == "elem" and getattr(ot.info, "base", None) is not None:
            return f"([&](int _w) {{ return _w == 0 ? Opt<int>() : Opt<int>(_w - 1); }}({oc}))", TOpt(TDom(ot.info.base))
        if isinstance(ot, TType) and isinstance(ot.t, TDom):
            info = ot.t.info
            if f in ("len", "K"):       # K is the name the Index trait gives the size (plan/19 §3.1)
                return f"(ll){info.len_cpp}", TInt(*info.len_iv)
            if f in ("first", "last"):
                self.note_check(e, f"{info.name} is non-empty")
                what = json.dumps(f"`{info.name}.{f}`: {info.name} is empty")
                return (f"chk_nonempty(0, {info.len_cpp}, {what}, {e.line})" if f == "first"
                        else f"chk_nonempty({info.len_cpp} - 1, {info.len_cpp}, {what}, {e.line})"), ot.t
            if f == "bounds":
                return "", TType(TDom(self.bounds_of(info)))
            if f == "bottom" and getattr(info, "base", None) is not None:
                return "0", ot.t
            if f in ("empty", "full") and getattr(info, "subset_of", None) is not None:
                return ("0" if f == "empty" else f"(int)({info.len_cpp} - 1)"), ot.t
            if f == "all":
                return f"Rng{{0, (ll){info.len_cpp}}}", TRange(ot.t)
            self.error(e, f"{info.name} has no `.{f}`")
        if isinstance(ot, TDom):
            info = ot.info
            if getattr(info, "subset_of", None) is not None:
                if f == "len":
                    return f"(ll)__builtin_popcount({oc})", TInt(0, info.subset_of.len_iv[1])
                if f == "is_empty":
                    return f"(({oc}) == 0)", BOOL
            if f == "mirror":
                return f"((int)({info.len_cpp}) - 1 - ({oc}))", ot
            if f in ("ord", "idx", "to_idx"):
                return f"(ll)({oc})", TInt(0, None if info.len_iv[1] is None else info.len_iv[1] - 1)
            if f == "rank":
                return f"((ll)({oc}) + 1)", TInt(1, info.len_iv[1])
            if f == "label":
                return self.dom_label(info, oc), TInt(info.lo_iv[0], None if None in (info.lo_iv[1], info.len_iv[1]) else info.lo_iv[1] + info.len_iv[1] - 1)
            if f in ("before", "after", "l", "r"):      # v.l / v.r: the boundary on the left / right of v
                b = self.bounds_of(info)
                return (oc if f in ("before", "l") else f"({oc} + 1)"), TDom(b)
            if f in ("left", "right") and info.of is not None:      # the position on either side of a boundary
                return f"cp_step({oc}, {1 if f == 'left' else 0}, true, {info.of.len_cpp})", TOpt(TDom(info.of))
            if f in ("next", "prev"):
                d = "+" if f == "next" else "-"
                tmp = self.tmpname("i")
                return (f"([&]{{ int {tmp} = {oc} {d} 1; return {tmp} >= 0 && {tmp} < {info.len_cpp} ? "
                        f"Opt<int>({tmp}) : Opt<int>(); }}())"), TOpt(ot)
            self.error(e, f"{info.name} values have no `.{f}`")
        if isinstance(ot, TTuple):
            if f in getattr(ot, "erased_fields", ()):
                self.error(e, f"field `{f}` is erased (`()`) under this const config; guard access with `if const`")
            i = ot.field(f)
            if i is None and f == "to_idx" and ot.cpp in self.index_impls:
                return self.to_idx(self.index_impls[ot.cpp], oc, e)
            if i is None and ot.cpp in self.struct_getters and f in self.struct_getters[ot.cpp]:
                fname = self.struct_getters[ot.cpp][f]
                rc, rt = self.expr(N("call", e.line, fn=N("name", e.line, name=fname), args=[copy.copy(e.obj)]), want)
                return self.maybe_n2_field(e, ot, f, rc, rt)
            if i is None:
                self.error(e, f"{ot!r} has no field `{f}`")
            lv = getattr(self, "lvalue", None)
            if self.narrow(ot.types[i]) and not (lv is not None and lv[0] is e):
                return f"(ll){oc}.{self.field_cpp(ot.names[i])}", ot.types[i]
            return f"{oc}.{self.field_cpp(ot.names[i])}", ot.types[i]
        if isinstance(ot, TRange):
            if f in ("start", "end", "lo", "hi", "l", "r"):
                c = f"{oc}.lo" if f in ("start", "lo", "l") else f"{oc}.hi"
                el = ot.elem
                return (f"(int)({c})" if isinstance(el, TDom) else c), (TDom(self.bounds_of(el.info)) if isinstance(el, TDom) else INT)
            if f == "len":
                hi = ot.elem.info.len_iv[1] if isinstance(ot.elem, TDom) else None
                return f"max(0LL, {oc}.hi - {oc}.lo)", TInt(0, hi)
            if f == "is_empty":
                return f"({oc}.hi <= {oc}.lo)", BOOL
            if f in ("first", "last", "mid"):
                suf = "dom" if isinstance(ot.elem, TDom) else "opt"
                fn = f"cp_r{f}_{suf}"
                return f"{fn}({oc})", TOpt(ot.elem)
        if isinstance(ot, TList) and f in ("first", "last"):
            et = ot.elem
            cty = self.cpp_type(TOpt(et))
            rc = f"([&](const auto& _v) -> {cty} {{ return _v.empty() ? {cty}() : {cty}(_v.{'front' if f == 'first' else 'back'}()); }}({oc}))"
            return self.maybe_n2_field(e, ot, f, rc, TOpt(et))
        if isinstance(ot, TType) and isinstance(ot.t, TEnum):
            if f == "K" and getattr(ot.t, "index_info", None) is not None:
                return f"(ll)({ot.t.index_info.len_cpp})", TInt(*ot.t.index_info.len_iv)
            k, v = ot.t.variant(f)
            if k is not None and not v[1]:
                return f"([&]{{ {ot.t.cpp} _e; _e.tag = {k}; return _e; }}())", ot.t
        if isinstance(ot, TRt) and ot.name == "Dir4":
            if f in ("cw", "ccw", "back"):
                return f"{oc}.{f}()", ot
        if isinstance(ot, TRt) and ot.name == "SuffixArray":
            if f in ("sa", "lcp"): return f"{oc}.{f}", TList(TInt(0))
        if isinstance(ot, TRt) and ot.name == "Pt":
            if f in ("x", "y"): return f"{oc}.{f}", TInt()
            if f == "norm2": return f"{oc}.norm2()", TInt(0)
        if isinstance(ot, TArr) and len(ot.dims) == 2 and f in ("flipped", "transposed"):
            r, c = ot.dims[0], ot.dims[1]
            if f == "flipped":
                return f"cp_grid_flipped({oc}, {r.info.len_cpp}, {c.info.len_cpp})", ot
            return f"cp_grid_transposed({oc}, {r.info.len_cpp}, {c.info.len_cpp})", TArr([c, r], ot.elem)
        if isinstance(ot, TRt) and ot.name == "Marks":
            if f == "len": return f"{oc}.size()", TInt(0)
            if f == "is_empty": return f"{oc}.empty()", BOOL
        if isinstance(ot, TRt) and ot.name == "Bitset":
            if f == "len": return f"{oc}.len()", TInt(0)
            if f == "is_empty": return f"{oc}.empty()", BOOL
        if isinstance(ot, TSet):
            if f == "len": return f"(ll){oc}.size()", TInt(0, None)
            if f == "is_empty": return f"{oc}.empty()", BOOL
            if f in ("first", "last", "min", "max") and not ot.hashed:
                ov = self.cpp_type(TOpt(ot.elem))
                end = "begin()" if f in ("first", "min") else "rbegin()"
                rc = f"({oc}.empty() ? {ov}() : {ov}(*{oc}.{end}))"
                return self.maybe_n2_field(e, ot, f, rc, TOpt(ot.elem))
        if isinstance(ot, TMap):
            if f == "len": return f"(ll){oc}.size()", TInt(0, None)
            if f == "is_empty": return f"{oc}.empty()", BOOL
            if f in ("first", "last") and not ot.hashed:
                pt = self.tuple_type(["f0", "f1"], [ot.key, ot.val])
                ov = self.cpp_type(TOpt(pt))
                end = "begin()" if f == "first" else "rbegin()"
                rc = f"([&](const auto& _m) -> {ov} {{ if (_m.empty()) return {ov}(); auto _it = _m.{end}; return {ov}({pt.cpp}{{_it->first, _it->second}}); }}({oc}))"
                return self.maybe_n2_field(e, ot, f, rc, TOpt(pt))
        if isinstance(ot, TCounter) and f == "distinct":
            return f"{oc}.distinct", TInt(0, None)
        if isinstance(ot, (TList, TStr, TMap, TSet)):
            if f == "len":
                return f"(ll){oc}.size()", TInt(0, None)
        if isinstance(ot, THeap) or (isinstance(ot, TRt) and ot.name == "Deque"):
            if f == "len":
                return f"(ll){oc}.size()", TInt(0, None)
            if f == "is_empty":
                return f"{oc}.empty()", BOOL
            el = self.cpp_type(TOpt(ot.elem))
            if isinstance(ot, THeap) and f == "peek":
                rc = f"({oc}.empty() ? {el}() : {el}({oc}.top()))"
                return self.maybe_n2_field(e, ot, f, rc, TOpt(ot.elem))
            if isinstance(ot, TRt) and f in ("first", "last", "front", "back"):
                rc = f"({oc}.empty() ? {el}() : {el}({oc}.{'front' if f in ('first', 'front') else 'back'}()))"
                return self.maybe_n2_field(e, ot, f, rc, TOpt(ot.elem))
        if isinstance(ot, TRt) and ot.name == "Desc" and f == "val":
            return f"{oc}.val", ot.elem
        if isinstance(ot, TRt) and ot.name == "MonoDeque" and f == "best":
            return self.maybe_n2_field(e, ot, f, f"{oc}.best()", TOpt(ot.elem))
        if isinstance(ot, TArr) and len(ot.dims) == 1:
            view = getattr(ot, "view", None)
            if f == "len":
                return f"(ll){oc}.size()", TInt(0, ot.dims[0].info.len_iv[1])
            if f == "positions":        # the window of a slice, as a plain Range value
                return (f"Rng{{{oc}.lo, {oc}.hi}}" if view else f"Rng{{0, (ll){ot.dims[0].info.len_cpp}}}"), TRange(ot.dims[0])
        if isinstance(ot, TType) and isinstance(ot.t, TTuple) and ot.t.cpp in self.index_impls and f == "K":
            info = self.index_impls[ot.t.cpp]
            return f"(ll){info.len_cpp}", TInt(*info.len_iv)
        if isinstance(ot, TTuple) and ot.cpp in self.index_impls and f == "to_idx" and ot.field(f) is None:
            return self.to_idx(self.index_impls[ot.cpp], oc, e)
        if isinstance(ot, TMod) and f == "value":
            return f"(ll)({oc}.v)", TInt(0, ot.m - 1)
        self.error(e, f"{ot!r} has no `.{f}`")

    # ---------- calls
    REDUCERS = {"sum", "count", "min", "max", "any", "all", "product", "prod"}

    def io_call(self, e):
        """`read()` / `ask(...)`, bare or as `read::<T>()`, when the program has not defined its own."""
        while e is not None and e.kind == "paren":
            e = e.a
        if e is None or e.kind != "call":
            return None
        f = e.fn.obj if e.fn.kind == "tapp" else e.fn
        if f.kind == "name" and f.name in ("read", "ask") and not self.lookup(f.name):
            return e
        return None

    def typed(self, e, ty):
        """The value of `e` is going somewhere of type `ty`: a bare `read()` / `ask()` there reads a `ty`."""
        c = self.io_call(e)
        if c is not None and ty is not None:
            self.io_type[id(c)] = ty
        return e

    def x_io(self, e):
        fn = e.fn
        name = (fn.obj if fn.kind == "tapp" else fn).name
        if fn.kind == "tapp":
            if len(fn.targs) != 1:
                self.error(e, f"`{name}::<T>()` takes one type")
            ty = self.resolve_type(fn.targs[0])
        else:
            ty = self.io_type.get(id(e))
            if ty is None:
                self.error(e, f"{name}() cannot tell what type to read here\n"
                              f"  help: give the place a type (`let x: T = {name}(...)`), or say `{name}::<T>(...)`")
        if name == "read" and e.args:
            self.error(e, "read(...) with arguments is the right-hand side of a typed `let`")
        if len(self.fn_ret) > getattr(self, "sol_depth", 0) and len(self.fn_ret) > (1 if getattr(self, "in_multitest", False) else 0):
            self.reads_in_fns = True
        tmp = self.tmpname("rd")

        def body():
            if name == "ask":
                pargs = [a for a in e.args if not (a.kind == "kwarg" and a.name == "limit")]
                self.emit_print(pargs, newline=True, node=e)
                self.emit("out_flush(); fflush(stdout);")
            self.emit(f"{self.cpp_type(ty)} {tmp};")
            self.emit_read(tmp, ty, e)
        # one expression, so it runs where it is written: after the operands to its left, and not at all on
        # the unused side of `and` / `or` / `if .. else`
        lines = self.capture(body)[0]
        return f"([&]() {{ {NL.join(lines)} return {tmp}; }})()", ty

    def x_call(self, e, want):
        fn = e.fn
        if self.io_call(e) is e:
            return self.x_io(e)
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
                if any(not self.orderable(t.inner if isinstance(t, TOpt) else t) for t in ts):
                    self.error(e, f"{fn.name} needs ordered values, and {ts[0]!r} has no `<`")
                if all(t == ts[0] for t in ts) and not isinstance(ts[0], TOpt):
                    c = cs[0][0]
                    for x, _ in cs[1:]:
                        c = f"{fn.name}({c}, {x})"
                    return c, ts[0]
                # a mix of T and ?T: `none` is "no value here" and is skipped, as in a reduction. The result
                # is a T when at least one argument is, else a ?T.
                inner = [t.inner if isinstance(t, TOpt) else t for t in ts]
                if all(is_int(t) for t in inner) or all(t == inner[0] for t in inner):
                    ints = is_int(inner[0])
                    plain = [(c, t) for c, t in cs if not isinstance(t, TOpt)]
                    opts = [(c, t.inner) for c, t in cs if isinstance(t, TOpt)]
                    if ints:
                        # the result is one of the arguments; a plain argument also bounds it from its own side
                        every = lambda xs: None if None in xs else xs
                        los, his = every([t.lo for t in inner]), every([t.hi for t in inner])
                        firm = [t for _, t in plain] or inner
                        if fn.name == "min":
                            cap = every([t.hi for t in firm])
                            rt = TInt(los and min(los), cap and (min(cap) if plain else max(cap)))
                        else:
                            floor = every([t.lo for t in firm])
                            rt = TInt(floor and (max(floor) if plain else min(floor)), his and max(his))
                    else:
                        rt = inner[0]
                    ct = self.cpp_type(rt)
                    better = "<" if fn.name == "min" else ">"
                    if plain:
                        c = plain[0][0]
                        for x, _ in plain[1:]:
                            c = f"{fn.name}<{ct}>({c}, {x})"
                        for x, _ in opts:
                            c = (f"([&]({ct} _a, const auto& _o) -> {ct} {{ return _o.has() && ({ct})_o.get() {better} _a "
                                 f"? ({ct})_o.get() : _a; }}({c}, {x}))")
                        return c, rt
                    ot = TOpt(rt)
                    oc = self.cpp_type(ot)
                    c = f"{oc}({opts[0][0]})"
                    for x, _ in opts[1:]:
                        c = (f"([&]({oc} _a, const auto& _o) -> {oc} {{ return _o.has() && (!_a.has() || ({ct})_o.get() "
                             f"{better} _a.get()) ? {oc}(({ct})_o.get()) : _a; }}({c}, {x}))")
                    return c, ot
                self.error(e, f"{fn.name} of {ts!r}")
        if fn.kind == "name" and fn.name == "rand" and not self.lookup("rand"):
            (lc, lt), (hc, ht) = self.expr(e.args[0]), self.expr(e.args[1])
            if not (is_int(lt) and is_int(ht)):
                self.error(e, "rand(lo, hi) needs Ints")
            return f"gen_int({lc}, {hc})", TInt(lt.lo, ht.hi)
        if (fn.kind == "field" and fn.obj.kind == "name" and fn.obj.name == "Graph" and fn.name in ("undirected", "directed")
                and len(e.args) == 1 and not self.lookup("Graph")):
            # Graph.undirected(edges) / Graph.directed(edges): the adjacency of an edge list (stdlib/graph.cp).
            # Edges are (u, v) or (u, v, w); a third field rides along, so `for (v, w) in g[u]` sees it.
            c, t = self.expr(e.args[0])
            et = t.elem if (isinstance(t, TArr) and len(t.dims) == 1) or isinstance(t, TList) else None
            if not (isinstance(et, TTuple) and len(et.types) in (2, 3) and isinstance(et.types[0], TDom)
                    and et.types[1] == et.types[0]):
                self.error(e, f"Graph.{fn.name} needs a list of edges (u: V, v: V) or (u: V, v: V, w: W), got {t!r}")
            vt = et.types[0]
            fs = [self.field_cpp(n) for n in et.names]
            if len(et.types) == 2:
                at, mk = vt, (lambda to: f"(int)_e.{to}")
            else:
                at = self.tuple_type(["", ""], [vt, et.types[2]])
                mk = lambda to: f"{at.cpp}{{(int)_e.{to}, _e.{fs[2]}}}"
            gt = TArr([vt], TList(at))
            back = f" _g[(int)_e.{fs[1]}].push_back({mk(fs[0])});" if fn.name == "undirected" else ""
            return (f"([&]{{ {self.cpp_type(gt)} _g({vt.info.len_cpp}); for (const auto& _e : {c}) {{ "
                    f"_g[(int)_e.{fs[0]}].push_back({mk(fs[1])});{back} }} return _g; }}())"), gt
        if fn.kind == "name" and fn.name == "condense" and len(e.args) == 2 and not self.lookup(fn.name):
            # condense(adj, comp): the graph of components, one arc per pair of joined components
            c, t = self.expr(e.args[0])
            vt, to = self.adjacency(t, e, "condense")
            cc, ct = self.expr(e.args[1])
            if not (isinstance(ct, TArr) and ct.dims == [vt] and isinstance(ct.elem, TDom)):
                self.error(e, f"condense needs the components as a [{vt!r}]C, got {ct!r}")
            return f"cp_condense({c}, {cc}, {ct.elem.info.len_cpp}, {to})", TArr([ct.elem], TList(ct.elem))
        if fn.kind == "name" and fn.name == "Rooted" and len(e.args) in (2, 3) and not self.lookup(fn.name):
            # Rooted(g, root) / Rooted(g, root: v, weight: e => e.len): a tree hung from `root`.
            pos = [a for a in e.args if a.kind != "kwarg"]
            kw = {a.name: a.value for a in e.args if a.kind == "kwarg"}
            weight = [N("kwarg", e.line, name="weight", value=kw["weight"])] if "weight" in kw else []
            root_arg = kw["root"] if "root" in kw else (pos[1] if len(pos) > 1 else None)
            if root_arg is None or len(pos) < 1:
                self.error(e, "Rooted(g, root) or Rooted(g, root, weight: e => e.len)")
            # Fuse Rooted(Graph.undirected(edges), root) directly onto the edge list to avoid intermediate vector<vector<int>>
            g_arg = pos[0]
            if (not weight and g_arg.kind == "call" and g_arg.fn.kind == "field" and g_arg.fn.obj.kind == "name"
                    and g_arg.fn.obj.name == "Graph" and g_arg.fn.name == "undirected"
                    and len(g_arg.args) == 1 and not self.lookup("Graph")):
                g_arg = g_arg.args[0]
            c, t = self.expr(g_arg)
            kind = None
            if isinstance(t, TArr) and len(t.dims) == 1 and isinstance(t.elem, TList):
                V, arc = t.dims[0], t.elem.elem
                to = "_a" if arc == V else f"_a.{self.field_cpp('to')}" if isinstance(arc, TTuple) and \
                    arc.field("to") is not None and arc.types[arc.field("to")] == V else None
                kind = "adj"
            elif isinstance(t, (TArr, TList)) and isinstance(t.elem, TTuple) and len(t.elem.types) >= 2 and \
                    isinstance(t.elem.types[0], TDom) and t.elem.types[0] == t.elem.types[1]:
                V, arc = t.elem.types[0], t.elem
                u, v = (f"_a.{self.field_cpp(n)}" for n in arc.names[:2])
                to = u
                kind = "edges"
            else:
                to = None
            if to is None:
                self.error(e, f"Rooted needs adjacency lists ([V]List<V>, or arcs with a `to: V`) or a list of edges "
                              f"(records that start with two V), got {t!r}")
            w = "1"
            if weight:
                pn, wc, wt = self.lambda_key(weight[0].value, arc)
                if not is_int(wt):
                    self.error(weight[0], f"`weight:` gives each edge an Int length, got {wt!r}")
                w = f"([&](const {self.cpp_type(arc)}& {pn}) -> ll {{ return {wc}; }}(_a))"
            rc, rt = self.expr(root_arg, V)
            if rt != V:
                self.error(e, f"the root must be a {V!r}, got {rt!r}")
            tt = TTree(V)
            tt.weighted = bool(weight)
            n_cpp = f"(int)({V.info.len_cpp})"
            if kind == "adj" and not weight and to == "_a":
                return f"RootedTree({c}, {rc})", tt
            if kind == "edges":
                if not weight:
                    return (f"([&]{{ const auto& _g = {c}; int _n = {n_cpp}; vector<int> _hd(_n + 1, 0); "
                            f"for (const auto& _a : _g) {{ ++_hd[(int)({u}) + 1]; ++_hd[(int)({v}) + 1]; }} "
                            f"for (int _i = 0; _i < _n; ++_i) _hd[_i + 1] += _hd[_i]; "
                            f"vector<int> _to(_hd[_n]), _cur(_hd.begin(), _hd.begin() + _n); "
                            f"for (const auto& _a : _g) {{ int _u = (int)({u}), _v = (int)({v}); _to[_cur[_u]++] = _v; _to[_cur[_v]++] = _u; }} "
                            f"return RootedTree(_n, _hd, _to, {rc}); }}())"), tt
                return (f"([&]{{ const auto& _g = {c}; int _n = {n_cpp}; vector<int> _hd(_n + 1, 0); "
                        f"for (const auto& _a : _g) {{ ++_hd[(int)({u}) + 1]; ++_hd[(int)({v}) + 1]; }} "
                        f"for (int _i = 0; _i < _n; ++_i) _hd[_i + 1] += _hd[_i]; "
                        f"vector<int> _to(_hd[_n]), _cur(_hd.begin(), _hd.begin() + _n); vector<ll> _wt(_hd[_n]); "
                        f"for (const auto& _a : _g) {{ int _u = (int)({u}), _v = (int)({v}); ll _w = {w}; "
                        f"int _pu = _cur[_u]++, _pv = _cur[_v]++; _to[_pu] = _v; _wt[_pu] = _w; _to[_pv] = _u; _wt[_pv] = _w; }} "
                        f"return RootedTree(_n, _hd, _to, _wt, {rc}); }}())"), tt
            if not weight:
                return (f"([&]{{ const auto& _g = {c}; int _n = {n_cpp}; vector<int> _hd(_n + 1, 0); "
                        f"for (int _u = 0; _u < _n; ++_u) _hd[_u + 1] = _hd[_u] + (int)_g[_u].size(); "
                        f"vector<int> _to(_hd[_n]); int _p = 0; "
                        f"for (int _u = 0; _u < _n; ++_u) for (const auto& _a : _g[_u]) _to[_p++] = (int)({to}); "
                        f"return RootedTree(_n, _hd, _to, {rc}); }}())"), tt
            return (f"([&]{{ const auto& _g = {c}; int _n = {n_cpp}; vector<int> _hd(_n + 1, 0); "
                    f"for (int _u = 0; _u < _n; ++_u) _hd[_u + 1] = _hd[_u] + (int)_g[_u].size(); "
                    f"vector<int> _to(_hd[_n]); vector<ll> _wt(_hd[_n]); int _p = 0; "
                    f"for (int _u = 0; _u < _n; ++_u) for (const auto& _a : _g[_u]) {{ _to[_p] = (int)({to}); _wt[_p] = {w}; ++_p; }} "
                    f"return RootedTree(_n, _hd, _to, _wt, {rc}); }}())"), tt
        if fn.kind == "generic" and fn.name in LIBS and not fn.args and not self.lookup(fn.name):
            return self.lib_new(self.resolve_type(N("tname", fn.line, name=fn.name, args=fn.args)), e.args, e)
        if fn.kind == "name" and fn.name == "Set" and len(e.args) == 1 and not self.lookup("Set"):
            # Set(xs): the distinct elements of a list or an array
            c, t = self.expr(e.args[0])
            if not isinstance(t, (TList, TArr)):
                self.error(e, f"Set(xs) takes a list or an array, got {t!r}")
            st = TSet(TInt() if is_int(t.elem) else t.elem)
            return f"([&]{{ auto _s = {self.set_ctor(st)}; for (const auto& _x : {c}) _s.insert(_x); return _s; }}())", st
        if fn.kind == "name" and fn.name == "bfs" and len(e.args) == 2 and not self.lookup("bfs"):
            # bfs(adj, from: sources) OR bfs(from: sources, next: u => adj[u])
            kw = {a.name: a.value for a in e.args if a.kind == "kwarg"}
            pos = [a for a in e.args if a.kind != "kwarg"]
            if "from" in kw and "next" in kw:
                src, adj_arg = kw["from"], kw["next"]
            elif "from" in kw and len(pos) == 1:
                adj_arg, src = pos[0], kw["from"]
            elif len(pos) == 2:
                adj_arg, src = pos[0], pos[1]
            else:
                self.error(e, "bfs(adj, from: sources) or bfs(from: sources, next: u => adj[u])")
            fc, ft = self.expr(src)
            if not (isinstance(ft, (TList, TArr)) and isinstance(ft.elem, TDom)):
                self.error(src, f"bfs(adj, from) starts from a list of vertices, got {ft!r}")
            V = ft.elem
            pn, ac, at = self.lambda_key(adj_arg, V)
            arc = at.elem if isinstance(at, (TList, TArr)) else None
            if not (arc == V or (isinstance(arc, TTuple) and arc.field("to") is not None and arc.types[arc.field("to")] == V)):
                self.error(adj_arg, f"bfs(adj, from): `adj` gives the neighbours of a {V!r} as a list of {V!r} "
                                      f"(or of arcs with a `to: {V!r}`), got {at!r}")
            rt = TArr([V], TOpt(TInt(0, V.info.len_iv[1])))
            return (f"cp_bfs<{self.cpp_type(rt.elem)}>({V.info.len_cpp}, {fc}, [&](int {pn}) -> decltype(auto) "
                    f"{{ return ({ac}); }})"), rt
        if fn.kind == "name" and fn.name == "choose" and len(e.args) == 2 and not self.lookup("choose"):
            # choose(n, k): exact; 0 when k is outside 0..=n (std.math)
            (nc, nt), (kc, kt) = self.expr(e.args[0]), self.expr(e.args[1])
            if not (is_int(nt) and is_int(kt)):
                self.error(e, "choose(n, k) needs Ints")
            import math
            hi = None
            if nt.hi is not None and kt.lo is not None and kt.lo == kt.hi:
                hi = math.comb(max(nt.hi, 0), kt.lo) if kt.lo >= 0 else 0
            elif nt.hi is not None and nt.hi <= 62:
                hi = math.comb(max(nt.hi, 0), max(nt.hi, 0) // 2)
            return f"cp_choose({nc}, {kc}, {e.line})", TInt(0, hi if hi is not None and hi <= INT_MAX else None)
        if fn.kind == "name" and fn.name in ("scan", "running_sum") and not self.lookup(fn.name) and \
                len(e.args) == (2 if fn.name == "scan" else 1):
            # scan(xs, f): the running fold x0, f(x0, x1), ...; running_sum(xs) is scan(xs, +) (std.seq)
            c, t = self.expr(e.args[0])
            if not isinstance(t, (TList, TArr)) or (isinstance(t, TArr) and len(t.dims) != 1):
                self.error(e, f"{fn.name} takes a list or a 1-D array, got {t!r}")
            et = TInt() if is_int(t.elem) else t.elem
            f = e.args[1] if fn.name == "scan" else N("lambda", e.line, op="+",
                params=[N("pname", e.line, name="_a"), N("pname", e.line, name="_b")],
                body=N("binop", e.line, op="+", a=N("name", e.line, name="_a"), b=N("name", e.line, name="_b")))
            ec = self.cpp_type(et)
            return (f"([&]{{ auto _f = {self.binary_fn(f, et)}; vector<{ec}> _r; const auto& _xs = {c}; _r.reserve(_xs.size()); "
                    f"for (const auto& _x : _xs) _r.push_back(_r.empty() ? ({ec})_x : _f(_r.back(), ({ec})_x)); return _r; }}())"), TList(et)
        if fn.kind == "name" and fn.name == "take" and len(e.args) == 1 and not self.lookup("take"):
            # take(place): moves the value out and leaves the type's empty value, so nothing is copied
            x = e.args[0]
            if x.kind == "name":
                if not getattr(self.lookup(x.name), "mut", False):
                    self.error(x, f"`{x.name}` is not a var")
            else:
                self.require_mut(x, x)
            self.note_write(x, e.line)
            self.lvalue = (x, "=")
            try:
                c, t = self.expr(x)
            finally:
                self.lvalue = None
            if not isinstance(t, (TList, TMap, TSet, TOpt, TStr)):
                self.error(e, f"take(place) needs a List, a Map, a Set, a Str or a ?T (types with an empty value), got {t!r}")
            return f"cp_take({c}, {self.set_ctor(t) if isinstance(t, (TMap, TSet)) else self.cpp_type(t) + '()'})", t
        if fn.kind == "name" and fn.name == "StrHash" and len(e.args) == 1 and not self.lookup(fn.name):
            c, t = self.expr(e.args[0])
            if not (isinstance(t, TArr) and len(t.dims) == 1 and
                    (isinstance(t.elem, TChar) or (isinstance(t.elem, TDom) and t.elem.info.labels == "char"))):
                self.error(e, f"StrHash needs a [S]Char, got {t!r}")
            return f"StrHash({c})", THash(t.dims[0])
        if fn.kind == "name" and fn.name in ("gcd", "lcm") and len(e.args) == 2 and not self.lookup(fn.name):
            (ac, at), (bc, bt) = self.expr(e.args[0]), self.expr(e.args[1])
            if not (is_int(at) and is_int(bt)):
                self.error(e, f"{fn.name} needs Ints")
            if fn.name == "gcd":
                his = [x for x in (at.hi, bt.hi) if x is not None]
                return f"std::gcd<ll, ll>({ac}, {bc})", TInt(0, max(his) if len(his) == 2 else None)
            hi = None if None in (at.hi, bt.hi) else at.hi * bt.hi
            return f"std::lcm<ll, ll>({ac}, {bc})", self.int_result(e, 0, hi)
        if fn.kind == "name" and fn.name == "counts" and len(e.args) == 1 and not self.lookup("counts"):
            # counts(xs): a Counter of the elements of a list, an array or a slice
            c, t = self.expr(e.args[0])
            if not isinstance(t, (TList, TArr)):
                self.error(e, f"counts takes a list or an array, got {t!r}")
            ct = TCounter(t.elem)
            x, r, s = self.tmpname("x"), self.tmpname("c"), self.tmpname("s")
            return (f"([&]{{ const auto& {s} = {c}; auto {r} = {self.cpp_type(ct)}{self.counter_init(ct)}; "
                    f"{r}.reserve_hint({s}.size()); for (const auto& {x} : {s}) {r}.inc({x}); return {r}; }}())"), ct
        if fn.kind == "name" and fn.name == "prefix" and len(e.args) == 1 and not self.lookup("prefix"):
            # prefix(a): the sum of everything before each boundary, so any block's sum is O(1)
            def prefix_narrow(dims, et):
                cells = 1
                for d in dims:
                    n = d.info.len_iv[1]
                    if n is None: return False
                    cells *= n
                if isinstance(et, TBool): return cells < 2**31
                if isinstance(et, TInt) and et.lo is not None and et.hi is not None:
                    return cells * max(abs(et.lo), abs(et.hi)) < 2**31
                return False
            # Fuse prefix([D1, D2] (r, c) => expr) directly into CpPrefix without materializing the table
            if e.args[0].kind == "table" and len(e.args[0].dims) in (1, 2):
                te = e.args[0]
                dims = []
                for d in te.dims:
                    if d.kind == "range":
                        dims.append(self.dim_type(N("trange", d.line, range=d)))
                    else:
                        _, dt = self.expr(d)
                        if not (isinstance(dt, TType) and isinstance(dt.t, TDom)):
                            self.error(d, f"table dimension must be a domain or range, got {dt!r}")
                        dims.append(dt.t)
                pats = [te.pat] if len(dims) == 1 else (te.pat.items if te.pat.kind == "ptuple" else None)
                if pats is None or len(pats) != len(dims):
                    self.error(te, "table pattern must bind one name per dimension")
                pre = self.tmpname("pre")
                box = {}
                def body():
                    self.scopes.append({})
                    vs = [self.fresh(p.name if p.name != "_" else "i") for p in pats]
                    for p, v, d in zip(pats, vs, dims):
                        self.scopes[-1][p.name] = Sym("let", d, v, node=p)
                    if len(dims) == 1:
                        v0, d0 = vs[0], dims[0]
                        self.emit(f"for (int {v0} = 0; {v0} < {d0.info.len_cpp}; ++{v0}) {{")
                        self.ind += 1
                        bc, bt = self.expr(te.body)
                        if not (is_int(bt) or isinstance(bt, TBool)):
                            self.error(e, f"prefix takes a 1-D or 2-D array of Int or Bool, got {bt!r}")
                        box["t"] = bt
                        ct = "int" if prefix_narrow(dims, bt) else "ll"
                        self.emit(f"{pre}.p[({v0} + 1) * 2 + 1] = {pre}.p[{v0} * 2 + 1] + ({ct})({bc});")
                        self.ind -= 1
                        self.emit("}")
                    else:
                        (v0, v1), (d0, d1) = vs, dims
                        self.emit(f"ll _w = {pre}.w;")
                        self.emit(f"for (int {v0} = 0; {v0} < {d0.info.len_cpp}; ++{v0}) {{")
                        self.ind += 1
                        self.emit(f"ll _r0 = (ll){v0} * _w, _r1 = _r0 + _w;")
                        self.emit(f"for (int {v1} = 0; {v1} < {d1.info.len_cpp}; ++{v1}) {{")
                        self.ind += 1
                        bc, bt = self.expr(te.body)
                        if not (is_int(bt) or isinstance(bt, TBool)):
                            self.error(e, f"prefix takes a 1-D or 2-D array of Int or Bool, got {bt!r}")
                        box["t"] = bt
                        ct = "int" if prefix_narrow(dims, bt) else "ll"
                        self.emit(f"{pre}.p[_r1 + {v1} + 1] = {pre}.p[_r0 + {v1} + 1] + {pre}.p[_r1 + {v1}] - {pre}.p[_r0 + {v1}] + ({ct})({bc});")
                        self.ind -= 1
                        self.emit("}")
                        self.ind -= 1
                        self.emit("}")
                    self.scopes.pop()
                lines, _ = self.capture(body)
                nar = prefix_narrow(dims, box["t"])
                rows = dims[0].info.len_cpp
                cols = dims[1].info.len_cpp if len(dims) == 2 else "1"
                pt = TPrefix(dims, narrow=nar)
                return f"([&]() {{ {self.cpp_type(pt)} {pre}({rows}, {cols});\n" + "\n".join(lines) + f"\n return {pre}; }}())", pt
            c, t = self.expr(e.args[0])
            # a Bool array counts its true cells
            if not (isinstance(t, TArr) and len(t.dims) in (1, 2) and (is_int(t.elem) or isinstance(t.elem, TBool))):
                self.error(e, f"prefix takes a 1-D or 2-D array of Int or Bool, got {t!r}")
            nar = prefix_narrow(t.dims, t.elem)
            rows = t.dims[0].info.len_cpp
            cols = t.dims[1].info.len_cpp if len(t.dims) == 2 else "1"
            pt = TPrefix(t.dims, narrow=nar)
            return f"{self.cpp_type(pt)}({c}, {rows}, {cols})", pt
        if fn.kind == "name" and fn.name == "span" and len(e.args) == 2 and not self.lookup("span"):
            # span(a, b): the smallest range that covers a and b, given in either order. Each is a
            # position (covered) or a boundary (the range reaches it) of one domain.
            ends = []
            for a in e.args:
                c, t = self.expr(a)
                if not isinstance(t, TDom):
                    self.error(a, f"span takes positions or boundaries of a domain, got {t!r}")
                ends.append((c, t.info.of or t.info, 0 if t.info.of is not None else 1))
            if ends[0][1] is not ends[1][1]:
                self.error(e, f"span of a {ends[0][1].name} and a {ends[1][1].name}")
            (a, info, wa), (b, _, wb) = ends
            return f"cp_span({a}, {wa}, {b}, {wb})", TRange(TDom(info))
        if fn.kind == "name" and fn.name in ("popcount", "ctz", "clz", "bit_len") and len(e.args) == 1 and not self.lookup(fn.name):
            # the bit helpers work on the 64-bit pattern; ctz(0) and clz(0) are 64, never undefined
            c, t = self.expr(e.args[0])
            if not is_int(t):
                self.error(e, f"{fn.name} needs an Int, got {t!r}")
            return f"cp_{fn.name}({c})", TInt(0, 64)
        if fn.kind == "name" and fn.name in ("bsearch_first", "bsearch_last", "bsearch_first_some") and len(e.args) == 2 \
                and not self.lookup(fn.name):
            # binary search over a range whose predicate changes once: bsearch_first finds the first true of
            # false..true, bsearch_last the last true of true..false, bsearch_first_some the first non-none
            rc, rt = self.expr(e.args[0])
            if not isinstance(rt, TRange):
                self.error(e, f"{fn.name}(range, i => ...) searches a range, got {rt!r}")
            el = rt.elem
            cty = "int" if isinstance(el, TDom) else "ll"
            pn, kc, kt = self.lambda_key(e.args[1], el)
            head = f"ll _lo = _r.lo, _hi = max(_r.lo, _r.hi); while (_lo < _hi) {{ ll _m = _lo + (_hi - _lo) / 2; const {cty} {pn} = ({cty})_m; "
            if fn.name == "bsearch_first_some":
                if not isinstance(kt, TOpt):
                    self.error(e, f"bsearch_first_some needs a fn returning an optional, got {kt!r}")
                pair = self.tuple_type([None, None], [el, kt.inner])
                res = self.cpp_type(TOpt(pair))
                vt = self.cpp_type(kt)
                return (f"([&](Rng _r) -> {res} {{ {vt} _best; {head}{vt} _v = {kc}; if (_v.has()) {{ _best = _v; _hi = _m; }} else _lo = _m + 1; }} "
                        f"return _lo < _r.hi ? {res}({pair.cpp}{{({cty})_lo, _best.get()}}) : {res}(); }}({rc}))"), TOpt(pair)
            if not isinstance(kt, TBool):
                self.error(e, f"{fn.name} needs a Bool predicate, got {kt!r}")
            res = self.cpp_type(TOpt(el))
            if fn.name == "bsearch_first":
                return (f"([&](Rng _r) -> {res} {{ {head}if ({kc}) _hi = _m; else _lo = _m + 1; }} "
                        f"return _lo < _r.hi ? {res}(({cty})_lo) : {res}(); }}({rc}))"), TOpt(el)
            return (f"([&](Rng _r) -> {res} {{ {head}if ({kc}) _lo = _m + 1; else _hi = _m; }} "
                    f"return _lo > _r.lo ? {res}(({cty})(_lo - 1)) : {res}(); }}({rc}))"), TOpt(el)
        if fn.kind == "name" and fn.name == "first" and len(e.args) == 1 and e.args[0].kind == "gen" and not self.lookup("first"):
            # first(x for x in xs if ok(x)): the first value produced, or none (the loop stops there)
            g = e.args[0]
            box = {}
            def inner():
                c, t = self.expr(g.elem)
                box["t"] = TInt() if is_int(t) else t
                self.emit(f"return {self.cpp_type(TOpt(box['t']))}({c});")
            lines, _ = self.capture(lambda: self.gen_loops(g.clauses, 0, inner, g))
            rt = TOpt(box["t"])
            return f"([&]() -> {self.cpp_type(rt)} {{ {NL.join(lines)} return {self.cpp_type(rt)}(); }}())", rt
        if fn.kind == "name" and fn.name in ("min_by", "max_by") and len(e.args) == 2 and not self.lookup(fn.name):
            # max_by(r, i => key): the element of a range whose key is largest (the first such); traps when empty
            rc, rt = self.expr(e.args[0])
            if not isinstance(rt, TRange):
                self.error(e, f"{fn.name}(range, i => key) takes a range, got {rt!r}")
            el = rt.elem
            cty = "int" if isinstance(el, TDom) else "ll"
            pn, kc, kt = self.lambda_key(e.args[1], el)
            if not self.orderable(kt):
                self.error(e, f"{fn.name} needs an ordered key, and {kt!r} has no `<`")
            cmp = "_k < _bk" if fn.name == "min_by" else "_bk < _k"
            self.note_check(e, "range is non-empty")
            return (f"([&](Rng _r) {{ if (_r.lo >= _r.hi) cp_fail(\"{fn.name} of an empty range\", {e.line}); "
                    f"auto _key = [&](const {cty} {pn}) {{ return {kc}; }}; {cty} _b = ({cty})_r.lo; auto _bk = _key(_b); "
                    f"for (ll _i = _r.lo + 1; _i < _r.hi; ++_i) {{ auto _k = _key(({cty})_i); if ({cmp}) {{ _b = ({cty})_i; _bk = _k; }} }} "
                    f"return _b; }}({rc}))"), el
        if fn.kind == "name" and fn.name == "read_line" and not e.args and not self.lookup("read_line"):
            return "rd_line()", STR
        if fn.kind == "name" and fn.name == "abs" and not self.lookup("abs"):
            c, t = self.expr(e.args[0])
            if not is_int(t):
                self.error(e, "abs needs Int")
            m = None if None in (t.lo, t.hi) else max(abs(t.lo), abs(t.hi))
            if t.lo is None:            # abs of the minimum Int does not fit
                return f"cp_abs({c}, {e.line})", TInt(0, None)
            return f"llabs({c})", TInt(0, m)
        if fn.kind == "name" and fn.name in ("sorted", "sort") and not self.lookup(fn.name):
            # sort(xs) / sort(xs, by: key) in place; sorted(...) returns a sorted copy
            c, t = self.expr(e.args[0])
            if not isinstance(t, (TList, TArr)) or (isinstance(t, TArr) and len(t.dims) != 1):
                self.error(e, f"{fn.name} needs a list or 1-D array")
            et = t.elem
            ec = self.cpp_type(et)
            by = [a for a in e.args[1:] if a.kind == "kwarg" and a.name == "by"]
            less = [a for a in e.args[1:] if a.kind == "kwarg" and a.name == "less"]
            if not by and not less and not self.orderable(et):
                self.error(e, f"{fn.name} needs an order, and {et!r} has none\n"
                              f"  help: give the key, e.g. `{fn.name}(xs, by: x => (x == none, x ?? 0))`")
            if len(e.args) > 1 and len(by) + len(less) != 1:
                self.error(e, f"{fn.name}(xs), {fn.name}(xs, by: x => key) or {fn.name}(xs, less: (a, b) => a_first)")
            cmp = ""
            if less:
                # for orders that have no key, e.g. by angle with exact cross products
                cmp = ", " + self.lambda2(less[0].value, et, et, BOOL)
                by = less
            elif by:
                pn, kc, kt = self.lambda_key(by[0].value, et)
                cmp = (f", [&](const auto& A, const auto& B) {{ auto _key = [&](const auto& {pn}) {{ return {kc}; }}; "
                       f"return _key(A) < _key(B); }}")
            algo = "stable_sort" if by else "sort"
            if fn.name == "sort":
                return f"{algo}({c}.begin(), {c}.end(){cmp})", UNIT
            tmp = self.tmpname("s")
            return (f"([&]{{ auto {tmp} = {c}; {algo}({tmp}.begin(), {tmp}.end(){cmp}); "
                    f"return {tmp}; }}())"), (t if isinstance(t, TArr) and not by else TList(et))
        if fn.kind == "name" and fn.name == "distinct" and len(e.args) == 1 and not self.lookup("distinct"):
            # distinct(xs): no two elements are equal (mostly for input `where` clauses)
            c, t = self.expr(e.args[0])
            if not isinstance(t, (TList, TArr)) or (isinstance(t, TArr) and len(t.dims) != 1):
                self.error(e, "distinct needs a list or 1-D array")
            return (f"([&]{{ auto _s = {c}; sort(_s.begin(), _s.end()); "
                    f"return adjacent_find(_s.begin(), _s.end()) == _s.end(); }}())"), BOOL
        if fn.kind == "name" and fn.name == "runs" and not self.lookup("runs"):
            # runs(xs) or runs(xs, same: (a, b) => Bool): maximal stretches of equal/`same` neighbours
            c, t = self.expr(e.args[0])
            same = [a for a in e.args[1:] if a.kind == "kwarg" and a.name == "same"]
            if not isinstance(t, (TList, TArr)) or (isinstance(t, TArr) and len(t.dims) != 1) or len(same) > 1 or len(e.args) > 2:
                self.error(e, "runs(xs[, same: (a, b) => Bool]) over a list or 1-D array")
            ec = self.cpp_type(t.elem)
            eq = self.lambda2(same[0].value, t.elem, t.elem, BOOL) if same else "[](const auto& _a, const auto& _b){ return _a == _b; }"
            return (f"([&]{{ auto _eq = {eq}; vector<vector<{ec}>> _r; const auto& _xs = {c}; "
                    f"for (size_t _i = 0; _i < _xs.size(); ++_i) {{ if (_i == 0 || !_eq(_xs[_i - 1], _xs[_i])) _r.emplace_back(); "
                    f"_r.back().push_back(_xs[_i]); }} return _r; }}())"), TList(TList(t.elem))
        if fn.kind == "name" and fn.name == "Pt" and len(e.args) == 2 and not self.lookup("Pt"):
            xc, _ = self.expr(e.args[0])
            yc, _ = self.expr(e.args[1])
            return f"CpPt({xc}, {yc})", TRt("Pt", [], "CpPt")
        if fn.kind == "name" and fn.name == "turn" and len(e.args) == 3 and not self.lookup("turn"):
            cs = [self.expr(a)[0] for a in e.args]
            return f"cp_turn({cs[0]}, {cs[1]}, {cs[2]})", TInt(-1, 1)
        if fn.kind == "name" and fn.name == "convex_hull" and len(e.args) == 1 and not self.lookup("convex_hull"):
            pc, pt = self.expr(e.args[0])
            return f"cp_convex_hull({pc})", TList(TRt("Pt", [], "CpPt"))
        if fn.kind == "name" and fn.name == "isqrt" and len(e.args) == 1 and not self.lookup("isqrt"):
            nc, _ = self.expr(e.args[0])
            return f"cp_isqrt({nc}, {e.line})", TInt(0)
        if fn.kind == "name" and fn.name == "pow_mod" and len(e.args) == 3 and not self.lookup("pow_mod"):
            ac, _ = self.expr(e.args[0])
            ec, _ = self.expr(e.args[1])
            mc, _ = self.expr(e.args[2])
            return f"cp_pow_mod({ac}, {ec}, {mc}, {e.line})", TInt(0)
        if fn.kind == "name" and fn.name == "factor" and len(e.args) == 1 and not self.lookup("factor"):
            nc, _ = self.expr(e.args[0])
            pt = self.tuple_type(["f0", "f1"], [TInt(), TInt()])
            return f"cp_factor<{pt.cpp}>({nc})", TList(pt)
        if fn.kind == "name" and fn.name == "sieve" and len(e.args) == 1 and not self.lookup("sieve"):
            nc, _ = self.expr(e.args[0])
            return f"cp_sieve({nc})", TList(TInt(2))
        if fn.kind == "name" and fn.name in ("sliding_min", "sliding_max") and len(e.args) == 2 and not self.lookup(fn.name):
            ac, at = self.expr(e.args[0])
            kc, _ = self.expr(e.args[1])
            is_min = "true" if fn.name == "sliding_min" else "false"
            return f"cp_sliding_minmax({ac}, {kc}, {is_min})", TList(at.elem)
        if fn.kind == "name" and fn.name == "reverse" and len(e.args) == 1 and not self.lookup("reverse"):
            self.require_mut(e.args[0], e)
            ac, _ = self.expr(e.args[0])
            return f"std::reverse({ac}.begin(), {ac}.end())", UNIT
        if fn.kind == "name" and fn.name == "dedup" and len(e.args) == 1 and not self.lookup("dedup"):
            self.require_mut(e.args[0], e)
            ac, _ = self.expr(e.args[0])
            return f"([&](auto& _v){{ _v.erase(unique(_v.begin(), _v.end()), _v.end()); }}({ac}))", UNIT
        if fn.kind == "name" and fn.name == "rotate_left" and len(e.args) == 2 and not self.lookup("rotate_left"):
            self.require_mut(e.args[0], e)
            ac, _ = self.expr(e.args[0])
            kc, _ = self.expr(e.args[1])
            return f"([&](auto& _v, ll _k){{ if (!_v.empty()) {{ _k = cp_emod_pos(_k, (ll)_v.size()); std::rotate(_v.begin(), _v.begin() + _k, _v.end()); }} }}({ac}, {kc}))", UNIT
        if fn.kind == "name" and fn.name == "next_permutation" and len(e.args) == 1 and not self.lookup("next_permutation"):
            self.require_mut(e.args[0], e)
            ac, _ = self.expr(e.args[0])
            return f"std::next_permutation({ac}.begin(), {ac}.end())", BOOL
        if fn.kind == "name" and fn.name == "bsearch_first" and not self.lookup("bsearch_first"):
            pos = [a for a in e.args if a.kind != "kwarg"]
            kw = {a.name: a.value for a in e.args if a.kind == "kwarg"}
            if len(pos) == 2 and pos[0].kind == "range" and pos[1].kind == "lambda" and len(pos[1].params) == 1:
                lc, lt = self.expr(pos[0].lo)
                hc, ht = self.expr(pos[0].hi)
                if isinstance(lt, TF64) or isinstance(ht, TF64):
                    steps_c = self.expr(kw["steps"])[0] if "steps" in kw else "80"
                    pn, bc, bt = self.lambda_key(pos[1], F64)
                    return (f"([&]() -> Opt<double> {{ double _lo = (double)({lc}), _hi = (double)({hc}); "
                            f"auto _p = [&](double {pn}) {{ return ({bc}); }}; "
                            f"if (!_p(_hi)) return Opt<double>(); "
                            f"for (ll _it = 0; _it < (ll)({steps_c}); ++_it) {{ double _m = 0.5 * (_lo + _hi); if (_p(_m)) _hi = _m; else _lo = _m; }} "
                            f"return Opt<double>(_hi); }}())"), TOpt(F64)
                et = lt if isinstance(lt, TDom) else (ht if isinstance(ht, TDom) else TInt())
                pn, bc, bt = self.lambda_key(pos[1], et)
                cty = self.cpp_type(TOpt(et))
                vty = self.cpp_type(et)
                hi_x = f"(ll)({hc}) + 1" if pos[0].incl else f"(ll)({hc})"
                return (f"([&]() -> {cty} {{ ll _lo = (ll)({lc}), _hi = {hi_x}, _ans = _hi; "
                        f"auto _p = [&]({vty} {pn}) {{ return ({bc}); }}; "
                        f"while (_lo < _hi) {{ ll _m = _lo + (_hi - _lo) / 2; if (_p(({vty})_m)) {{ _ans = _m; _hi = _m; }} else _lo = _m + 1; }} "
                        f"return _ans < {hi_x} ? {cty}(({vty})_ans) : {cty}(); }}())"), TOpt(et)
        if fn.kind == "lambda" and not fn.params and not e.args:
            return self.expr(fn.body, want)
        if fn.kind == "name" and fn.name == "read_output" and not e.args and not self.lookup("read_output"):
            rt = want if want is not None else INT
            tmp = self.tmpname("ro")
            self.emit(f"{self.cpp_type(rt)} {tmp};")
            self.emit_read(tmp, rt, e)
            return tmp, rt
        if fn.kind == "name" and fn.name == "topo" and len(e.args) == 1 and not self.lookup("topo"):
            gc, gt = self.expr(e.args[0])
            if not (isinstance(gt, TArr) and len(gt.dims) == 1 and isinstance(gt.elem, TList) and gt.elem.elem == gt.dims[0]):
                self.error(e, f"topo needs [V]List<V>, got {gt!r}")
            return f"cp_topo({gc})", TOpt(TList(gt.dims[0]))
        if fn.kind == "name" and fn.name == "scc" and len(e.args) == 1 and not self.lookup("scc"):
            gc, gt = self.expr(e.args[0])
            if not (isinstance(gt, TArr) and len(gt.dims) == 1 and isinstance(gt.elem, TList) and gt.elem.elem == gt.dims[0]):
                self.error(e, f"scc needs [V]List<V>, got {gt!r}")
            return f"cp_scc_list({gc})", TList(TList(gt.dims[0]))
        if fn.kind == "name" and fn.name == "z_function" and len(e.args) == 1 and not self.lookup("z_function"):
            sc, _ = self.expr(e.args[0])
            return f"cp_z_function({sc})", TList(TInt(0))
        if fn.kind == "name" and fn.name == "prefix_function" and len(e.args) == 1 and not self.lookup("prefix_function"):
            sc, _ = self.expr(e.args[0])
            return f"cp_prefix_function({sc})", TList(TInt(0))
        if fn.kind == "name" and fn.name == "SuffixArray" and len(e.args) == 1 and not self.lookup("SuffixArray"):
            sc, _ = self.expr(e.args[0])
            return f"CpSuffixArray({sc})", TRt("SuffixArray", [], "CpSuffixArray")
        if fn.kind == "name" and fn.name == "RollingHash" and len(e.args) == 1 and not self.lookup("RollingHash"):
            sc, _ = self.expr(e.args[0])
            return f"CpRollingHash({sc})", TRt("RollingHash", [], "CpRollingHash")
        if fn.kind == "name" and fn.name == "palindromes" and len(e.args) == 1 and not self.lookup("palindromes"):
            sc, _ = self.expr(e.args[0])
            return f"cp_palindromes({sc})", TList(TInt(0))
        if fn.kind == "generic" and fn.name == "Fenwick2D" and len(fn.args) == 3 and not e.args:
            t = self.resolve_type(N("tname", fn.line, name="Fenwick2D", args=fn.args))
            return f"CpFenwick2D<{self.cpp_type(t.args[2])}>({t.args[0].info.len_cpp}, {t.args[1].info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "TwoSat" and len(fn.args) == 1 and not e.args:
            t = self.resolve_type(N("tname", fn.line, name="TwoSat", args=fn.args))
            return f"CpTwoSat({t.args[0].info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "FlowGraph" and len(fn.args) == 1 and not e.args:
            t = self.resolve_type(N("tname", fn.line, name="FlowGraph", args=fn.args))
            nt = t.args[0]
            nc = nt.info.len_cpp if isinstance(nt, TDom) else (str(len(nt.variants)) if isinstance(nt, TEnum) else "0")
            return f"CpFlowGraph({nc})", t
        if fn.kind == "generic" and fn.name == "Steps" and len(fn.args) == 2:
            t = self.resolve_type(N("tname", fn.line, name="Steps", args=fn.args))
            dc = self.coerce(*self.expr(e.args[0], t.args[1]), t.args[1], e.args[0]) if e.args else f"{self.cpp_type(t.args[1])}()"
            return f"CpStepsT<{self.cpp_type(t.args[0])}, {self.cpp_type(t.args[1])}>({dc})", t
        if fn.kind == "name" and fn.name == "dijkstra" and not self.lookup("dijkstra"):
            gc, gt = self.expr(e.args[0])
            ok = (isinstance(gt, TArr) and len(gt.dims) == 1 and isinstance(gt.elem, TList)
                  and isinstance(gt.elem.elem, TTuple) and len(gt.elem.elem.types) == 2
                  and gt.elem.elem.types[0] == gt.dims[0] and is_int(gt.elem.elem.types[1]))
            if not ok:
                self.error(e, f"dijkstra needs [D]List<(D, Int)>, got {gt!r}")
            s_arg = e.args[1].value if e.args[1].kind == "kwarg" and e.args[1].name == "from" else e.args[1]
            sc, st = self.expr(s_arg, gt.dims[0])
            sc = self.coerce(sc, st, gt.dims[0], s_arg)
            return f"cp_dijkstra({gc}, {sc})", TArr([gt.dims[0]], TOpt(TInt(0, None)))
        if fn.kind == "generic" and fn.name in ("Map", "Set", "HashMap", "HashSet", "SortedBag"):
            ts = [self.resolve_type(a) if a.kind.startswith("t") else self.resolve_type(N("texpr", a.line, expr=a)) for a in fn.args]
            if fn.name in ("Set", "HashSet", "SortedBag"):
                t = TSet(ts[0], hashed=(fn.name == "HashSet"), bag=(fn.name == "SortedBag"))
            else:
                t = TMap(*ts, hashed=fn.name == "HashMap")
            return self.set_ctor(t), t
        if fn.kind == "generic" and fn.name == "Bitset" and len(fn.args) == 1 and not e.args:
            t = self.resolve_type(N("tname", fn.line, name="Bitset", args=fn.args))
            return f"CpBitset({t.args[0].info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "Subset" and len(fn.args) == 1 and not e.args:
            t = self.resolve_type(N("tname", fn.line, name="Subset", args=fn.args))
            return "0", t
        if fn.kind == "generic" and fn.name == "AssignSum" and not e.args:
            # AssignSum<D>(): assign to a range, sum of a range; every cell starts at 0 (stdlib/lazy.cp)
            t = self.resolve_type(N("tname", fn.line, name="AssignSum", args=fn.args))
            return f"AssignSum({t.dom.info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "AddMax":
            # AddMax<D>(): add to a range, max of a range; every cell starts at 0
            t = self.resolve_type(N("tname", fn.line, name="AddMax", args=fn.args))
            if e.args:
                self.error(e, "AddMax<D>() takes no arguments")
            return f"AddMax({t.dom.info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "SegTree" and e.args and len(e.args) <= 2 and e.args[-1].kind != "lambda":
            return self.range_struct(e)      # SegTree<D, Int>(+, a): a stock operator and the array
        if fn.kind == "generic" and fn.name == "SegTree":
            # SegTree<D, T>(id, (a, b) => op): a monoid given as two plain values
            t = self.resolve_type(N("tname", fn.line, name="SegTree", args=fn.args))
            if len(e.args) != 2 or e.args[1].kind != "lambda" or len(e.args[1].params) != 2:
                self.error(e, "SegTree<D, T>(identity, (a, b) => combined)")
            ic, it = self.expr(e.args[0], t.elem)
            ic = self.coerce(ic, it, t.elem, e.args[0])
            ec = self.cpp_type(t.elem)
            return f"SegTree<{ec}>({t.dom.info.len_cpp}, {ic}, {self.lambda2(e.args[1], t.elem, t.elem, t.elem)})", t
        if fn.kind == "generic" and fn.name == "Marks" and len(fn.args) == 1 and not e.args:
            t = self.resolve_type(N("tname", fn.line, name="Marks", args=fn.args))
            return f"CpMarks({t.args[0].info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "Dsu" and not e.args:
            # Dsu<V>(): disjoint sets over a domain, every value alone to start (stdlib/dsu.cp)
            t = self.resolve_type(N("tname", fn.line, name="Dsu", args=fn.args))
            return f"CpDsu({t.dom.info.len_cpp})", t
        if fn.kind == "generic" and fn.name == "Comb" and len(fn.args) == 1 and len(e.args) in (0, 1):
            # Comb<P>() or Comb<P>(n): auto-growing factorials and inverse factorials modulo P
            mt = self.resolve_type(N("tname", fn.line, name="Mod", args=fn.args))
            nc = "0"
            if e.args:
                nc, nt = self.expr(e.args[0])
                if not is_int(nt):
                    self.error(e, f"Comb<P>(n) takes an Int bound, got {nt!r}")
            t = TRt("Comb", [mt], "CpComb")
            return f"{self.cpp_type(t)}({nc})", t
        if fn.kind == "generic" and fn.name in ("Fenwick", "SparseTable", "LazySegTree") and len(fn.args) == 2 \
                or (fn.kind == "generic" and fn.name == "SegTree" and len(e.args) == 2 and e.args[1].kind != "lambda"):
            return self.range_struct(e)
        if fn.kind == "generic" and fn.name == "MonoDeque":
            # MonoDeque<T>(min) / MonoDeque<T>(max): which end `.best` reports
            t = self.resolve_type(N("tname", fn.line, name="MonoDeque", args=fn.args))
            if len(e.args) != 1 or e.args[0].kind != "name" or e.args[0].name not in ("min", "max"):
                self.error(e, "MonoDeque<T>(min) or MonoDeque<T>(max)")
            return f"{self.cpp_type(t)}({'true' if e.args[0].name == 'min' else 'false'})", t
        if fn.kind == "name" and fn.name == "desc" and len(e.args) == 1 and not self.lookup("desc"):
            # desc(x): x with its order reversed, for one descending field of a key
            inner = want.args[0] if isinstance(want, TRt) and want.name == "Desc" else None
            c, t = self.expr(e.args[0], inner)
            if not self.orderable(t):
                self.error(e, f"desc(x) reverses an order, and {t!r} has none")
            rt = TRt("Desc", [inner or (TInt() if is_int(t) else t)], "CpDesc")
            return f"{self.cpp_type(rt)}{{{self.coerce(c, t, rt.args[0], e.args[0])}}}", rt
        if fn.kind == "generic" and fn.name in ("List", "MinHeap", "MaxHeap", "Deque"):
            a = fn.args[0]
            if fn.name != "List":
                t = self.resolve_type(N("tname", fn.line, name=fn.name, args=fn.args))
                if e.args:
                    self.error(e, f"`{fn.name}<T>()` takes no arguments")
                return f"{self.cpp_type(t)}()", t
            et = self.resolve_type(a) if a.kind.startswith("t") else self.resolve_type(N("texpr", a.line, expr=a))
            t = TList(et) if fn.name == "List" else THeap(et)
            if e.args:
                self.error(e, f"`{fn.name}<T>()` takes no arguments" + (
                    "; it pops the smallest element, so for another order push a key first: "
                    "`h.push((-x, x))` pops the largest `x`" if fn.name == "MinHeap" else ""))
            return f"{self.cpp_type(t)}()", t
        if fn.kind == "name" and fn.name in ("F64", "U64", "I128") and len(e.args) == 1 and not self.lookup(fn.name):
            wt = {"F64": F64, "U64": U64, "I128": I128}[fn.name]
            c, t = self.expr(e.args[0])
            if not (is_int(t) or isinstance(t, (TF64, TU64, TI128))) or (isinstance(t, TF64) and wt is not F64):
                self.error(e, f"cannot convert {t!r} to {fn.name}" + ("; round it first with floor/ceil/round/trunc" if isinstance(t, TF64) else ""))
            return f"(({self.cpp_type(wt)})({c}))", wt
        if fn.kind == "name" and fn.name in ("floor", "ceil", "round", "trunc") and len(e.args) == 1 and not self.lookup(fn.name):
            c, t = self.expr(e.args[0], F64)
            if not isinstance(t, TF64):
                self.error(e, f"{fn.name} takes an F64, got {t!r}")
            return f"cp_f2i({fn.name}({c}), {e.line})", TInt()
        F1 = {"sqrt": "sqrt", "sin": "sin", "cos": "cos", "tan": "tan", "exp": "exp", "ln": "log"}
        F2 = {"atan2": "atan2", "hypot": "hypot"}
        if fn.kind == "name" and (fn.name in F1 or fn.name in F2 or fn.name in ("close", "sign")) and not self.lookup(fn.name):
            n = 1 if fn.name in F1 else 3 if fn.name == "close" else 2
            if len(e.args) != n:
                self.error(e, f"{fn.name} takes {n} F64 argument{'s' if n > 1 else ''}")
            cs = []
            for a in e.args:
                c, t = self.expr(a, F64)
                cs.append(self.coerce(c, t, F64, a))
            if fn.name == "close":      # close(a, b, eps): |a - b| <= eps
                return f"(fabs(({cs[0]}) - ({cs[1]})) <= ({cs[2]}))", BOOL
            if fn.name == "sign":       # sign(x, eps): -1, 0 or 1, with |x| <= eps counted as 0
                return f"cp_fsign({cs[0]}, {cs[1]})", TInt(-1, 1)
            return f"cp_fchk({(F1.get(fn.name) or F2[fn.name])}({', '.join(cs)}), {e.line})", F64
        if fn.kind == "name" and fn.name in ("abs", "min", "max") and e.args and not self.lookup(fn.name) and all(a.kind != "gen" for a in e.args):
            ts = [self.capture(lambda a=a: self.expr(a))[1][1] for a in e.args]
            wide = next((t for t in ts if isinstance(t, (TF64, TU64, TI128))), None)
            if wide is not None:
                cs = []
                for a in e.args:
                    c, t = self.expr(a, wide)
                    cs.append(self.coerce(c, t, wide, a))
                ct = self.cpp_type(wide)
                if fn.name == "abs":
                    if len(cs) != 1 or isinstance(wide, TU64):
                        self.error(e, "abs takes one signed number")
                    return (f"fabs({cs[0]})" if isinstance(wide, TF64) else f"([](__int128 _x) {{ return _x < 0 ? -_x : _x; }}({cs[0]}))"), wide
                c = cs[0]
                for x in cs[1:]:
                    c = f"{fn.name}<{ct}>({c}, {x})"
                return c, wide
        if fn.kind == "tapp" and fn.obj.kind == "name" and fn.obj.name == "parse" and len(fn.targs) == 1 and len(e.args) == 1:
            tt = self.resolve_type(fn.targs[0])
            sc, st = self.expr(e.args[0])
            if not isinstance(st, TStr):
                self.error(e, f"parse::<{tt!r}> expects a Str, got {st!r}")
            if isinstance(tt, TInt):
                return f"cp_parse_int({sc}, {e.line})", tt
            if isinstance(tt, TU64):
                return f"cp_parse_u64({sc}, {e.line})", tt
            if isinstance(tt, TF64):
                return f"cp_parse_f64({sc}, {e.line})", tt
            if isinstance(tt, TChar):
                return f"cp_at({sc}, 0, {e.line})", CHAR
            self.error(e, f"parse::<{tt!r}> is not supported")
        if fn.kind == "name" and fn.name == "debug" and not self.lookup("debug"):
            parts = []
            for i, a in enumerate(e.args):
                ac, at = self.expr(a)
                if i:
                    parts.append('fputc(\' \', stderr);')
                if isinstance(at, TInt):
                    parts.append(f'fprintf(stderr, "%lld", (long long)({ac}));')
                elif isinstance(at, TBool):
                    parts.append(f'fputs(({ac}) ? "true" : "false", stderr);')
                elif isinstance(at, TChar):
                    parts.append(f'fputc({ac}, stderr);')
                elif isinstance(at, TStr):
                    parts.append(f'fwrite(({ac}).data(), 1, ({ac}).size(), stderr);')
                elif isinstance(at, TF64):
                    parts.append(f'fprintf(stderr, "%g", (double)({ac}));')
                elif isinstance(at, TDom):
                    parts.append(f'fprintf(stderr, "%lld", (long long)({ac} + {at.info.lo_cpp}));')
                else:
                    parts.append('fputs("?", stderr);')
            parts.append('fputc(\'\\n\', stderr);')
            body = " ".join(parts)
            return f"([&](){{ (void)0; /* debug */ if (CP_LOCAL) {{ {body} }} }}())", UNIT
        if fn.kind == "name" and fn.name == "Str" and len(e.args) == 1 and not self.lookup("Str"):
            if e.args[0].kind == "gen":     # `Str('0' + d for d in row)`: the Chars, in order
                e = N("call", e.line, fn=fn, args=[N("list", e.line, items=[e.args[0]])])
            ac, at = self.expr(e.args[0])
            if (isinstance(at, TList) and isinstance(at.elem, TChar)) or (isinstance(at, TArr) and len(at.dims) == 1 and isinstance(at.elem, TChar)):
                return f"([&](const auto& _cs){{ return string(_cs.begin(), _cs.end()); }}({ac}))", STR
            if isinstance(at, TStr):
                return ac, STR
            if isinstance(at, TChar):
                return f"string(1, {ac})", STR
            self.error(e, f"Str(...) expects List<Char>, [D]Char, Char, or Str, got {at!r}")
        if fn.kind == "name" and fn.name in ("Int", "Bool"):
            c, t = self.expr(e.args[0])
            if fn.name == "Int" and isinstance(t, TF64):
                self.error(e, "Int(x) on an F64 does not say how to round: write floor(x), ceil(x), round(x) or trunc(x)")
            if fn.name == "Int" and isinstance(t, (TU64, TI128)):
                return f"cp_narrow64({c}, {e.line})", TInt()
            if fn.name == "Int":
                if isinstance(t, TBool): return f"(ll)({c})", TInt(0, 1)
                if isinstance(t, TChar): return f"(ll)({c})", TInt(0, 255)
                if isinstance(t, TMod): return f"(ll)(({c}).v)", TInt(0, t.m - 1)
                if is_int(t): return c, t
                if isinstance(t, TDom) and t.info.of is None and t.info.labels == "int":
                    # the written label, the same number `print(v)` shows and `V(x)` takes back
                    return self.x_field(N("field", e.line, obj=N("ctemp", e.line, code=c, ty=t, src=e.args[0]), name="label"), None)
                if isinstance(t, TDom):
                    self.error(e, f"Int(v) is the written label of a domain over Ints, and {t!r} is not one (use `v.to_idx` for its position)")
            self.error(e, f"cannot convert {t!r} to {fn.name}")
        if fn.kind == "tapp" and fn.obj.kind == "name":
            fsym = self.lookup(fn.obj.name)
            if fsym is not None and fsym.kind == "genfn":
                return self.call_generic(fsym.node, e, explicit_targs=fn.targs)
        if fn.kind == "generic" and getattr(self.lookup(fn.name), "kind", None) == "genstruct":
            sym = self.lookup(fn.name)
            st = self.specialize_struct(sym.node, fn.args, fn)
            return self.construct_struct(st, sym.node, e)
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "genstruct":
            sym = self.lookup(fn.name)
            st = self.specialize_struct(sym.node, [], fn)
            return self.construct_struct(st, sym.node, e)
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "type" \
                and getattr(getattr(self.lookup(fn.name), "node", None), "kind", None) == "struct":
            sym = self.lookup(fn.name)
            return self.construct_struct(sym.ty, sym.node, e)
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "ctor":
            return self.construct(self.lookup(fn.name), e.args, e)
        if fn.kind == "name" and fn.name == "swap" and len(e.args) == 2 and not self.lookup("swap"):
            return self.builtin_swap(e)
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
                if fnode.body is not None:
                    self.check_call_effects(e, fsym)
        if any(getattr(a, "mut", False) for a in e.args) and not (
                fn.kind == "name" and (getattr(self.lookup(fn.name), "kind", None) in ("fn", "genfn")
                                       or fn.name in ("sort", "reverse", "dedup", "swap", "rotate_left", "next_permutation"))):
            self.error(e, "`mut` arguments are only for user fns with mut parameters")
        if fn.kind == "name" and getattr(self.lookup(fn.name), "kind", None) == "genfn":
            return self.call_generic(self.lookup(fn.name).node, e)
        fc, ft = self.expr(fn) if fn.kind != "field" else (None, None)
        if fn.kind == "field":
            if fn.name in ("push", "pop", "pop_front", "pop_back", "sort", "rev", "clear", "insert", "remove", "swap", "set", "add", "unite", "refresh", "fill"):
                self.note_write(fn.obj, e.line)
            return self.method_call(e, want)
        if isinstance(ft, TType):
            t = ft.t
            if isinstance(t, TDom) and len(e.args) == 1:
                info = t.info
                c, at = self.expr(e.args[0])
                if isinstance(at, TDom) and at.info is info:
                    return c, t
                if isinstance(at, TChar) and info.labels == "char":
                    c, at = f"(ll)({c})", TInt(0, 255)
                if not is_int(at):
                    self.error(e, f"{info.name}(x) needs an Int, got {at!r}")
                self.note_check(e, f"`{unparse(e.args[0])}` in {info.name}")
                return f"chk_dom((ll)({c}) - {info.lo_cpp}, {info.len_cpp}, {e.line})", t
            if isinstance(t, TMod) and len(e.args) == 1:
                c, at = self.expr(e.args[0])
                return f"{self.cpp_type(t)}({c})", t
            self.error(e, f"cannot call type {t!r}")
        if isinstance(ft, TFn):
            if len(e.args) != len(ft.params):
                self.error(e, f"expected {len(ft.params)} arguments, got {len(e.args)}")
            args, ats, mut_args = [], [], []
            for a, pt in zip(e.args, ft.params):
                saved_raw = getattr(self, "_raw_name_node", None)
                if getattr(a, "mut", False) and a.kind == "name":
                    self._raw_name_node = a
                    mut_args.append(a)
                try:
                    c, at = self.expr(self.typed(a, pt), pt)
                finally:
                    self._raw_name_node = saved_raw
                args.append(self.coerce(c, at, pt, a))
                ats.append(at)
            for a in mut_args:
                self.invalidate_narrow(a)
            # everything passed by reference to one call is lent for the same time (plan/19 §3.6)
            self.check_args(e, [(a, getattr(a, "mut", False)) for a, at in zip(e.args, ats)
                                if getattr(a, "mut", False) or not isinstance(at, SCALARS)])
            sym = self.lookup(fn.name) if fn.kind == "name" else None
            if sym is not None and sym.kind == "fn" and getattr(sym.node, "body", None) is not None:
                self.check_call_effects(e, sym)
            if any(getattr(at, "view", None) for at in ats):
                # a lent [B]T is a [B]T to the callee: the fn is specialised for the view (decision 4.8)
                sym = self.view_spec(sym, ats, e)
                fc = sym.cpp
            if getattr(sym, "local", False):
                args = [fc] + args
            return f"{fc}({', '.join(args)})", ft.ret
        self.error(e, f"cannot call {ft!r}")

    RANGE_OPS = {"+": ("0", "_a + _b"), "*": ("1", "_a * _b"), "^": ("0", "_a ^ _b"), "&": ("-1", "_a & _b"),
                 "|": ("0", "_a | _b"), "min": ("LLONG_MAX", "min(_a, _b)"), "max": ("LLONG_MIN", "max(_a, _b)"),
                 "gcd": ("0", "std::gcd(_a, _b)")}
    LAZY_TAGS = {"AddSum": (0, 0), "AddMin": (0, 1), "AddMax": (0, 2), "AssignSum": (1, 0), "AssignMin": (1, 1), "AssignMax": (1, 2)}

    def range_struct(self, e):
        """Fenwick<D, Int>(a), SparseTable<D, Int>(min, a), SegTree<D, T>(+, a), LazySegTree<D, Int>(AddSum, a):
        the range structures of cheat sheet section 11, over a domain D, built from an array or empty."""
        fn = e.fn
        d = self.resolve_type(fn.args[0])
        et = self.resolve_type(fn.args[1])
        if not isinstance(d, TDom):
            self.error(e, f"{fn.name}<D, T> needs a domain D, got {d!r}")
        if not is_int(et):
            self.error(e, f"the prototype's {fn.name}<D, T> with a bare operator holds Ints; give SegTree an (identity, op) pair for another T")
        args = list(e.args)
        n = d.info.len_cpp
        def source():
            if not args:
                return None
            c, t = self.expr(args.pop(0))
            if not (isinstance(t, TArr) and t.dims == [d] and is_int(t.elem)):
                self.error(e, f"{fn.name}<{d!r}, Int> is built from a [{d!r}]Int, got {t!r}")
            return c
        def op_of(a):
            name = a.op if a.kind == "lambda" and getattr(a, "op", None) else a.name if a.kind == "name" else None
            if name not in self.RANGE_OPS:
                self.error(e, f"{fn.name} takes one of the operators + * ^ & | min max gcd")
            return name
        if fn.name == "Fenwick":
            src = source()
            t = TRt("Fenwick", [d], "CpFenwick")
            return (f"CpFenwick({n}, {src})" if src else f"CpFenwick({n})"), t
        if fn.name == "LazySegTree":
            tag = args.pop(0) if args else None
            if tag is not None and tag.kind == "call" and tag.fn.kind == "name" and tag.fn.name == "Action" and len(tag.args) == 3:
                idc, _ = self.expr(tag.args[0])
                comp = self.lambda2(tag.args[1], TInt(), TInt(), TInt())
                app = self.lambda2(tag.args[2], TInt(), TInt(), TInt())
                t = TRt("LazySegTree", [d], "CpCustomLazy")
                return f"CpCustomLazy({n}, (ll)({idc}), {comp}, {app})", t
            if tag is None or tag.kind != "name" or tag.name not in self.LAZY_TAGS:
                self.error(e, f"LazySegTree<D, Int>(Tag, a) takes a stock tag: {', '.join(self.LAZY_TAGS)}")
            assign, fold = self.LAZY_TAGS[tag.name]
            src = source()
            t = TRt("LazySegTree", [d], "CpLazy")
            return f"CpLazy({n}, {assign}, {fold}{', ' + src if src else ''})", t
        if not args:
            self.error(e, f"{fn.name}<D, T>(op, a)")
        op = op_of(args.pop(0))
        ident, body = self.RANGE_OPS[op]
        src = source()
        lam = f"[](ll _a, ll _b) {{ return {body}; }}"
        if fn.name == "SparseTable":
            if op not in ("min", "max", "gcd", "&", "|") or src is None:
                self.error(e, "SparseTable<D, Int>(op, a) needs an idempotent operator (min, max, gcd, &, |) and the array")
            t = TRt("SparseTable", [d], "CpSparse")
            return f"CpSparse({src}, (ll)({ident}), {lam})", t
        t = TSeg(d, TInt())
        if src is None:
            return f"SegTree<ll>({n}, (ll)({ident}), {lam})", t
        return f"([&]{{ const auto& _src = {src}; return SegTree<ll>({n}, (ll)({ident}), {lam}, [&](int _i) {{ return (ll)_src[_i]; }}); }}())", t

    def dom_range(self, a, d):
        """The C++ of a range argument over the domain d (`l..=r`, `l..`, `..r`): a half-open Rng of ordinals."""
        c, t = self.expr(a)
        if not isinstance(t, TRange) or t.elem != d:
            self.error(a, f"expected a range of {d!r} (`l..=r`, `l..`, `..=r`), got {t!r}")
        return c

    def str_method(self, e, oc, m, args):
        """The Str vocabulary of the cheat sheet (section 4); None when `m` is not one of them."""
        pos = [a for a in args if a.kind != "kwarg"]
        kw = {a.name: a.value for a in args if a.kind == "kwarg"}
        def arg(k, want, what):
            c, t = self.expr(pos[k], want)
            if not isinstance(t, type(want)):
                self.error(e, f"`.{m}` takes {what}, got {t!r}")
            return c
        if m == "push" and len(pos) == 1:
            self.require_mut(e.fn.obj, e)
            return f"{oc}.push_back({arg(0, CHAR, 'a Char')})", UNIT
        if m == "starts_with" and len(pos) == 1:
            at = self.str_pos(kw["at"]) if "at" in kw else "0"
            return f"cp_str_starts({oc}, {arg(0, STR, 'a Str')}, {at})", BOOL
        if m == "split" and len(pos) == 1:
            return f"cp_str_split({oc}, {arg(0, CHAR, 'a Char separator')})", TList(STR)
        if m == "split_once" and len(pos) == 1:
            pair = self.tuple_type([None, None], [STR, STR])
            return (f"([&](const string& _s, char _c) {{ size_t _k = _s.find(_c); return _k == string::npos ? Opt<{pair.cpp}>() : "
                    f"Opt<{pair.cpp}>({pair.cpp}{{_s.substr(0, _k), _s.substr(_k + 1)}}); }}({oc}, {arg(0, CHAR, 'a Char separator')}))"), TOpt(pair)
        if m == "trim_start" and len(pos) == 1:
            return f"cp_str_trim_start({oc}, {arg(0, CHAR, 'a Char')})", STR
        if m == "parse_int" and not pos:
            return f"cp_parse_int({oc}, {e.line})", TInt()
        if m == "join" and len(pos) == 1:
            if pos[0].kind == "gen":
                acc, first = self.tmpname("j"), self.tmpname("f")
                lines, _ = self.capture(lambda: self.gen_loops(pos[0].clauses, 0, lambda: self.emit(
                    f"if (!{first}) {acc} += _sep; {first} = false; {acc} += {self.expr(pos[0].elem, STR)[0]};"), pos[0]))
                body = " ".join(l.strip() for l in lines)
                return f"([&](const string& _sep) {{ string {acc}; bool {first} = true; {body} return {acc}; }}({oc}))", STR
            c, t = self.expr(pos[0])
            if t != TList(STR):
                self.error(e, f"`.join` takes Strs (a List<Str> or `\"{{x}}\" for x in xs`), got {t!r}")
            return f"cp_str_join({oc}, {c})", STR
        if m == "copy" and not pos:
            return f"string({oc})", STR
        return None

    def method_call(self, e, want):
        obj, m = e.fn.obj, e.fn.name
        was_nonempty = self.is_local_nonempty(obj)
        oc, ot = self.expr(obj)
        rc, rt = self._method_call_inner(e, obj, m, oc, ot, e.args, want)
        # Rule N2: syntactic non-empty narrowing on local bindings via `@law some_when(not self.is_empty)`
        rc, rt = self.maybe_n2_method(e, obj, ot, m, rc, rt, was_nonempty=was_nonempty)
        # Invariant 3: any mutating method call immediately invalidates local narrowing on `obj`
        if self.is_mutating_method(ot, m):
            self.invalidate_narrow(obj)
        return rc, rt

    def _method_call_inner(self, e, obj, m, oc, ot, args, want):
        tbl = getattr(self, "struct_methods", {})
        key = ot.t.cpp if isinstance(ot, TType) and hasattr(ot.t, "cpp") else getattr(ot, "cpp", None)
        if key in self.projs and m in self.projs[key]:
            return self.call_proj(self.projs[key][m], e, oc, ot)
        if isinstance(ot, TArr) and m == "get":
            return self.array_get(e, oc, ot, args)
        if isinstance(ot, TList) and m == "get" and len(args) == 1:
            return self.list_get(e, oc, ot, args[0])
        if isinstance(ot, TArr) and m == "fill" and len(args) == 1:
            # `cnt.fill(0)`, `seen.fill(none)`: every cell of the array, whatever its dimensions
            self.require_mut(obj, e)
            vc, vt = self.expr(args[0], ot.elem)
            return f"cp_fill({oc}, {self.coerce(vc, vt, ot.elem, args[0])})", UNIT
        if isinstance(ot, TStr):
            r = self.str_method(e, oc, m, args)
            if r is not None:
                return r
        if isinstance(ot, TType) and isinstance(ot.t, TLib) and m == "new":
            return self.lib_new(ot.t, args, e)
        if isinstance(ot, TType) and isinstance(ot.t, TDsu) and m == "new" and not args:
            return f"CpDsu({ot.t.dom.info.len_cpp})", ot.t         # Dsu<V>.new(), as the library spells it
        if isinstance(ot, TLib) and ot.name == "Steps":
            K = ot.args[0]
            def rng(a):
                c, t = self.expr(a)
                if not (isinstance(t, TRange) and (t.elem == K or (is_int(K) and is_int(t.elem)))):
                    self.error(a, f"a {ot!r} takes a range of {K!r}, got {t!r}")
                return c
            if m == "add" and len(args) == 2:
                vc, vt = self.expr(args[1])
                if not is_int(vt):
                    self.error(args[1], f"add(range, x) needs an Int x, got {vt!r}")
                return f"{oc}.add({rng(args[0])}, {vc})", UNIT
            if m == "runs" and len(args) == 1:
                tt = self.tuple_type([None, None], [TRange(K), INT])
                return f"{oc}.runs<{tt.cpp}>({rng(args[0])})", TList(tt)
            self.error(e, f"no method {m} on {ot!r} (add(range, x), runs(range))")
        if isinstance(ot, TType) and isinstance(ot.t, TSeg) and m == "of" and len(args) == 2:
            # SegTree<D, T>.of(op, f): leaf i is f(i); `refresh(i)` recomputes it, so the formula is written once
            t = ot.t
            ident, op = self.monoid(args[0], t.elem)
            pn, fc, ft = self.lambda_key(args[1], t.dom)
            ec = self.cpp_type(t.elem)
            return (f"SegTree<{ec}>({t.dom.info.len_cpp}, {ident}, {op}, [&](int {pn}) -> {ec} "
                    f"{{ return {self.coerce(fc, ft, t.elem, args[1])}; }})"), t
        if isinstance(ot, TRange) and m == "clamp" and len(args) == 1:
            dc, dt = self.expr(args[0])
            if not (isinstance(dt, TType) and isinstance(dt.t, TDom)):
                self.error(args[0], f"Range.clamp expects a domain, got {dt!r}")
            info = dt.t.info
            return (f"([&](Rng _r){{ ll _lo = max(0LL, _r.lo - (ll){info.lo_cpp}); "
                    f"ll _hi = min((ll){info.len_cpp}, _r.hi - (ll){info.lo_cpp}); "
                    f"return Rng{{_lo, max(_lo, _hi)}}; }}({oc}))"), TRange(dt.t)
        if (isinstance(ot, TRange) or (isinstance(ot, TType) and isinstance(ot.t, TDom))) and m == "chunks" and len(args) == 1:
            rc, el, kc = self.windows_of(oc, ot, args[0], e)
            return (f"([&](Rng _r, ll _k) {{ vector<Rng> _w; for (ll _s = _r.lo; _s < _r.hi; _s += _k) "
                    f"_w.push_back(Rng{{_s, min(_r.hi, _s + _k)}}); return _w; }}({rc}, {kc}))"), TList(TRange(el))
        if (isinstance(ot, TRange) or (isinstance(ot, TType) and isinstance(ot.t, TDom))) and m == "windows" and len(args) == 1:
            # r.windows(k) / D.windows(k): every stretch of k positions in a row, each one a Range
            rc, el, kc = self.windows_of(oc, ot, args[0], e)
            return (f"([&](Rng _r, ll _k) {{ vector<Rng> _w; for (ll _s = _r.lo; _s + _k <= _r.hi; ++_s) "
                    f"_w.push_back(Rng{{_s, _s + _k}}); return _w; }}({rc}, {kc}))"), TList(TRange(el))
        if isinstance(ot, TRange) and m in ("partition_point", "first_where") and len(args) == 1:
            # binary search over any range, of Ints or of positions: the predicate is false then true
            # (first_where) or true then false (partition_point)
            el = ot.elem
            cty = "int" if isinstance(el, TDom) else "ll"
            pn, kc, kt = self.lambda_key(args[0], el)
            if not isinstance(kt, TBool):
                self.error(e, f"{m} needs a Bool predicate")
            flip = "!" if m == "first_where" else ""
            loop = (f"ll _lo = _r.lo, _hi = max(_r.lo, _r.hi); while (_lo < _hi) {{ ll _m = _lo + (_hi - _lo) / 2; "
                    f"{cty} {pn} = ({cty})_m; if ({flip}({kc})) _lo = _m + 1; else _hi = _m; }}")
            if m == "partition_point":
                return (f"([&](Rng _r) {{ {loop} return ({cty})_lo; }}({oc}))",
                        TDom(self.bounds_of(el.info)) if isinstance(el, TDom) else INT)
            rt = TOpt(el)
            return f"([&](Rng _r) {{ {loop} return _lo < _r.hi ? {self.cpp_type(rt)}(({cty})_lo) : {self.cpp_type(rt)}(); }}({oc}))", rt
        if isinstance(ot, TRange) and m in ("prefix", "suffix") and len(args) == 1:
            # r.prefix(k) / r.suffix(k): the first / last k positions of r (all of r when it is shorter)
            kc, kt = self.expr(args[0])
            if not is_int(kt):
                self.error(e, f"`.{m}(k)` takes an Int count, got {kt!r}")
            return f"cp_r{m}({oc}, {kc})", ot
        if isinstance(ot, TDom) and m == "offset" and len(args) == 1:
            # v.offset(d): the named form of `v + d`
            kc, kt = self.expr(args[0])
            if not is_int(kt):
                self.error(e, f"`.offset(d)` takes an Int, got {kt!r}")
            return f"cp_step({oc}, {kc}, false, {ot.info.len_cpp})", TOpt(ot)
        if isinstance(ot, TDom) and m in ("l", "r") and len(args) == 1:
            # v.l(k) / v.r(k): the far boundary of the k-wide window that ends / starts at v; v.l == v.l(1)
            b = self.bounds_of(ot.info)
            kc, kt = self.expr(args[0])
            if not is_int(kt):
                self.error(args[0], f"`.{m}(k)` takes an Int width, got {kt!r}")
            self.note_check(e, f"`{show(e)}` is a boundary of {ot.info.name}")
            what = json.dumps(f"`{show(e)}` leaves {b.name}")
            step = f"{oc}, 1, {kc}, true" if m == "l" else f"{oc}, 0, {kc}, false"
            return f"{self.step_fn(kt)}({step}, {b.len_cpp}, {what}, {e.line})", TDom(b)
        if isinstance(ot, TType) and isinstance(ot.t, TDom) and m == "from_idx" and len(args) == 1:
            # Synthesis #3 L3.3: trapping D.from_idx(i) -> D (consistent with Index.from_idx and D(x))
            info = ot.t.info
            ic, it = self.expr(args[0])
            if not is_int(it):
                self.error(args[0], f"{info.name}.from_idx takes an Int, got {it!r}")
            self.note_check(e, f"`{unparse(args[0])}` is a 0-based position of {info.name}")
            return f"chk_dom((ll)({ic}), {info.len_cpp}, {e.line})", ot.t
        if isinstance(ot, TType) and isinstance(ot.t, TDom) and m == "try_from_idx" and len(args) == 1:
            info = ot.t.info
            ic, it = self.expr(args[0])
            if not is_int(it):
                self.error(args[0], f"{info.name}.try_from_idx takes an Int, got {it!r}")
            return (f"([&](ll _i) -> Opt<int> {{ return _i >= 0 && _i < (ll){info.len_cpp} "
                    f"? Opt<int>((int)_i) : Opt<int>(); }}({ic}))"), TOpt(ot.t)
        if isinstance(ot, TType) and isinstance(ot.t, TTuple) and ot.t.cpp in self.index_impls and m == "from_idx":
            info = self.index_impls[ot.t.cpp]
            return self.expr(N("call", e.line, fn=N("name", e.line, name=info.from_idx), args=list(args)), want)
        if isinstance(ot, TArr) and len(ot.dims) == 2 and m == "row" and len(args) == 1:
            # Array.row: lends the contiguous block of one row as a [B]T (plan/19 §4.3)
            if getattr(ot, "view", None) or isinstance(ot.elem, TBool):
                self.error(e, "rows of a view or of a Bool grid are not in the prototype")
            rc = self.index_into(args[0], ot.dims[0].info, e)
            rt = TArr([ot.dims[1]], ot.elem)
            rt.view = "row"
            return f"cp_row({oc}, {rc}, {ot.dims[1].info.len_cpp})", rt
        if isinstance(ot, TArr) and len(ot.dims) == 1 and m == "swap" and len(args) == 2:
            # a.swap(i, j): exchange two cells; i == j is fine and changes nothing (plan/19 §4.5)
            if isinstance(ot.elem, TBool):
                self.error(e, "`.swap(i, j)` on a Bool array is not in the prototype")
            self.require_mut(obj, e)
            i, j = (self.index_into(a, ot.dims[0].info, e) for a in args)
            return f"swap({oc}[{i}], {oc}[{j}])", UNIT
        if isinstance(ot, TList) and m == "copy" and not args:
            return f"{self.cpp_type(ot)}({oc})", ot       # an owned copy: `for x in xs.copy()` may then change xs
        if isinstance(ot, TArr) and len(ot.dims) == 1 and m in ("sort", "rev", "copy") and not args:
            if isinstance(ot.elem, TBool):
                self.error(e, f"`.{m}()` on a Bool array is not in the prototype")
            if m in ("sort", "rev"):
                self.require_mut(obj, e)
                return (f"sort({oc}.begin(), {oc}.end())" if m == "sort" else f"reverse({oc}.begin(), {oc}.end())"), UNIT
            owned = TList(ot.elem) if getattr(ot, "view", None) == "slice" else TArr(ot.dims, ot.elem)
            return f"{self.cpp_type(owned)}({oc}.begin(), {oc}.end())", owned
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
        if isinstance(ot, TType) and isinstance(ot.t, TDom) and m in ("partition_point", "first_where"):
            # binary search over a domain for a predicate that is false then true (first_where)
            # or true then false (partition_point, as in plan/12)
            info = ot.t.info
            pn, kc, kt = self.lambda_key(args[0], ot.t)
            if not isinstance(kt, TBool):
                self.error(e, f"{m} needs a Bool predicate")
            flip = "!" if m == "first_where" else ""
            search = (f"([&]{{ int _lo = 0, _hi = {info.len_cpp}; while (_lo < _hi) {{ int {pn} = (_lo + _hi) / 2; "
                      f"if ({flip}({kc})) _lo = {pn} + 1; else _hi = {pn}; }} return _lo; }}())")
            if m == "partition_point":
                return search, TDom(self.bounds_of(info))
            return f"([&](int _k){{ return _k < {info.len_cpp} ? Opt<int>(_k) : Opt<int>(); }}({search}))", TOpt(ot.t)
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
        sub = ot.t if isinstance(ot, TType) else ot
        if isinstance(sub, TDom) and getattr(sub.info, "subset_of", None) is not None and m in ("with", "without", "of", "shifted"):
            if m == "shifted" and len(args) == 1:
                kc, kt = self.expr(args[0])
                b = sub.info.subset_of
                return (f"([&](int _m, ll _k){{ int _n = (int){b.len_cpp}; "
                        f"if (_k >= _n || _k <= -_n) return 0; "
                        f"return _k >= 0 ? (int)(((_m << _k) & ((1 << _n) - 1))) : (_m >> (-_k)); }}({oc}, {kc}))"), sub
            el = TDom(sub.info.subset_of)
            c, t = self.expr(args[0], el)
            if t != el:
                self.error(e, f"{m} needs a {el!r}, got {t!r}")
            if m == "of":
                return f"(1 << ({c}))", sub
            return (f"(({oc}) | (1 << ({c})))" if m == "with" else f"(({oc}) & ~(1 << ({c})))"), sub
        if isinstance(ot, (TMap, TSet)):
            kt = ot.key if isinstance(ot, TMap) else ot.elem
            def key(a):
                c, t = self.expr(a, kt)
                return self.coerce(c, t, kt, a)
            if isinstance(ot, TMap) and m == "get":
                ov = self.cpp_type(TOpt(ot.val))
                return f"([&](auto _it){{ return _it == {oc}.end() ? {ov}() : {ov}(_it->second); }}({oc}.find({key(args[0])})))", TOpt(ot.val)
            if isinstance(ot, TMap) and not ot.hashed and m in ("first_ge", "first_gt", "last_le", "last_lt", "ge", "gt", "le", "lt") and len(args) == 1:
                pt = self.tuple_type(["f0", "f1"], [ot.key, ot.val])
                ov = self.cpp_type(TOpt(pt))
                k = key(args[0])
                if m in ("first_ge", "ge"):
                    return f"([&](const auto& _m){{ auto _it = _m.lower_bound({k}); return _it == _m.end() ? {ov}() : {ov}({pt.cpp}{{_it->first, _it->second}}); }}({oc}))", TOpt(pt)
                if m in ("first_gt", "gt"):
                    return f"([&](const auto& _m){{ auto _it = _m.upper_bound({k}); return _it == _m.end() ? {ov}() : {ov}({pt.cpp}{{_it->first, _it->second}}); }}({oc}))", TOpt(pt)
                if m in ("last_le", "le"):
                    return f"([&](const auto& _m){{ auto _it = _m.upper_bound({k}); if (_it == _m.begin()) return {ov}(); --_it; return {ov}({pt.cpp}{{_it->first, _it->second}}); }}({oc}))", TOpt(pt)
                return f"([&](const auto& _m){{ auto _it = _m.lower_bound({k}); if (_it == _m.begin()) return {ov}(); --_it; return {ov}({pt.cpp}{{_it->first, _it->second}}); }}({oc}))", TOpt(pt)
            if isinstance(ot, TSet) and m in ("insert", "add"):
                if ot.bag:
                    return f"({oc}.insert({key(args[0])}), true)", BOOL
                return f"{oc}.insert({key(args[0])}).second", BOOL
            if m == "remove":
                if isinstance(ot, TSet) and ot.bag:
                    return f"([&](auto& _s){{ auto _it = _s.find({key(args[0])}); if (_it == _s.end()) return false; _s.erase(_it); return true; }}({oc}))", BOOL
                return f"({oc}.erase({key(args[0])}) > 0)", BOOL
            if m == "clear":
                return f"{oc}.clear()", UNIT
            if isinstance(ot, TSet) and m == "count" and len(args) == 1:
                return f"(ll){oc}.count({key(args[0])})", TInt(0)
            if isinstance(ot, TSet) and not ot.hashed and m in ("min", "max"):
                ov = self.cpp_type(TOpt(ot.elem))
                end = "begin()" if m == "min" else "rbegin()"
                return f"({oc}.empty() ? {ov}() : {ov}(*{oc}.{end}))", TOpt(ot.elem)
            if isinstance(ot, TSet) and not ot.hashed and m in ("pop_first", "pop_last") and not args:
                self.require_mut(obj, e)
                ov = self.cpp_type(TOpt(ot.elem))
                if m == "pop_first":
                    return f"([&](auto& _s) -> {ov} {{ if (_s.empty()) return {ov}(); auto _v = *_s.begin(); _s.erase(_s.begin()); return {ov}(_v); }}({oc}))", TOpt(ot.elem)
                return f"([&](auto& _s) -> {ov} {{ if (_s.empty()) return {ov}(); auto _it = prev(_s.end()); auto _v = *_it; _s.erase(_it); return {ov}(_v); }}({oc}))", TOpt(ot.elem)
            if isinstance(ot, TSet) and not ot.hashed and m in ("first_ge", "first_gt", "last_le", "last_lt", "ge", "gt", "le", "lt") and len(args) == 1:
                ov = self.cpp_type(TOpt(ot.elem))
                k = key(args[0])
                if m in ("first_ge", "ge"):
                    return f"([&](const auto& _s) -> {ov} {{ auto _it = _s.lower_bound({k}); return _it == _s.end() ? {ov}() : {ov}(*_it); }}({oc}))", TOpt(ot.elem)
                if m in ("first_gt", "gt"):
                    return f"([&](const auto& _s) -> {ov} {{ auto _it = _s.upper_bound({k}); return _it == _s.end() ? {ov}() : {ov}(*_it); }}({oc}))", TOpt(ot.elem)
                if m in ("last_le", "le"):
                    return f"([&](const auto& _s) -> {ov} {{ auto _it = _s.upper_bound({k}); if (_it == _s.begin()) return {ov}(); --_it; return {ov}(*_it); }}({oc}))", TOpt(ot.elem)
                return f"([&](const auto& _s) -> {ov} {{ auto _it = _s.lower_bound({k}); if (_it == _s.begin()) return {ov}(); --_it; return {ov}(*_it); }}({oc}))", TOpt(ot.elem)
            if isinstance(ot, TSet) and not ot.hashed and m == "rank" and len(args) == 1:
                return f"cp_set_rank({oc}, {key(args[0])})", TInt(0)
            if isinstance(ot, TSet) and not ot.hashed and m == "nth" and len(args) == 1:
                ov = self.cpp_type(TOpt(ot.elem))
                kc, _ = self.expr(args[0])
                return f"cp_set_nth<decltype({oc}), {ov}>({oc}, {kc})", TOpt(ot.elem)
            self.error(e, f"no method {m} on {ot!r}")
        if isinstance(ot, TList):
            if m == "push":
                c, t = self.expr(args[0], ot.elem)
                return f"{oc}.push_back({self.coerce(c, t, ot.elem, args[0])})", UNIT
            if m == "push_id" and len(args) == 1:
                self.require_mut(obj, e)
                c, t = self.expr(args[0], ot.elem)
                return f"([&](auto& _v){{ ll _id = (ll)_v.size(); _v.push_back({self.coerce(c, t, ot.elem, args[0])}); return _id; }}({oc}))", TInt(0)
            if m == "pop":
                # same shape as heap.pop(): ?T, so `while let u = stack.pop()` drains a list
                oc_t = self.cpp_type(TOpt(ot.elem))
                return f"([&]{{ if ({oc}.empty()) return {oc_t}(); auto _v = {oc}.back(); {oc}.pop_back(); return {oc_t}(_v); }}())", TOpt(ot.elem)
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
        if isinstance(ot, TArr) and getattr(ot, "compressed", False) and m == "at" and len(args) == 1:
            # xs.at(val): the position of a value that was compressed (a value that was not is a bug: it traps)
            c, t = self.expr(args[0], ot.elem)
            return (f"([&](const auto& _x) {{ auto _it = lower_bound({oc}.begin(), {oc}.end(), _x); "
                    f"if (_it == {oc}.end() || !(*_it == _x)) cp_fail(\"compress: `.at` of a value that was not compressed\", {e.line}); "
                    f"return (int)(_it - {oc}.begin()); }}({self.coerce(c, t, ot.elem, args[0])}))"), ot.dims[0]
        if isinstance(ot, TArr) and len(ot.dims) == 1 and m == "find" and len(args) == 1:
            # a.find(x): the first position holding x (linear), or none
            c, t = self.expr(args[0], ot.elem)
            return (f"([&](const auto& _x){{ ll _k = find({oc}.begin(), {oc}.end(), _x) - {oc}.begin(); "
                    f"return _k < (ll){oc}.size() ? Opt<int>((int)_k) : Opt<int>(); }}({self.coerce(c, t, ot.elem, args[0])}))"), TOpt(ot.dims[0])
        if isinstance(ot, TArr) and len(ot.dims) == 1 and m in ("last_le", "first_ge"):
            c, t = self.expr(args[0])
            if m == "last_le":
                return (f"([&](auto _x){{ ll _k = upper_bound({oc}.begin(), {oc}.end(), _x) - {oc}.begin() - 1; "
                        f"return _k >= 0 ? Opt<int>((int)_k) : Opt<int>(); }}({c}))"), TOpt(ot.dims[0])
            return (f"([&](auto _x){{ ll _k = lower_bound({oc}.begin(), {oc}.end(), _x) - {oc}.begin(); "
                    f"return _k < (ll){oc}.size() ? Opt<int>((int)_k) : Opt<int>(); }}({c}))"), TOpt(ot.dims[0])
        if isinstance(ot, TTree) and m in ("ancestor", "step") and len(args) == (2 if m == "ancestor" else 3):
            # ancestor(v, k): k steps up from v; step(a, b, k): k steps from a along the path to b (stdlib/tree.cp)
            cs = []
            for a in args[:-1]:
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"nodes of this tree are {ot.dom!r}, got {t!r}")
                cs.append(c)
            kc, kt = self.expr(args[-1])
            if not is_int(kt):
                self.error(args[-1], f"{m} needs an Int number of steps, got {kt!r}")
            return (f"([&](int _p){{ return _p < 0 ? Opt<int>() : Opt<int>(_p); }}({oc}.{m}({', '.join(cs)}, {kc})))",
                    TOpt(ot.dom))
        if isinstance(ot, TTree):
            cs = []
            for a in args:
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"nodes of this tree are {ot.dom!r}, got {t!r}")
                cs.append(c)
            if m == "lca" and len(cs) == 2:
                return f"{oc}.lca({cs[0]}, {cs[1]})", ot.dom
            if m == "parent" and len(cs) == 1:
                return f"([&](int _p){{ return _p < 0 ? Opt<int>() : Opt<int>(_p); }}({oc}.par[{cs[0]}]))", TOpt(ot.dom)
            if m == "pos" and len(cs) == 1:
                return f"(ll){oc}.tin[{cs[0]}]", TInt(0, ot.dom.info.len_iv[1])
            if m == "depth" and len(cs) == 1:
                return f"(ll){oc}.dep[{cs[0]}]", TInt(0, ot.dom.info.len_iv[1])
            if m in ("sub", "chains") and len(cs) == (1 if m == "sub" else 2):
                # positions in the heavy-first preorder: a subtree is one range of them, a path a few (stdlib/tree.cp)
                pre = self.tree_pre(ot)
                span = self.tuple_type(["l", "r"], [pre, pre])
                if m == "sub":
                    return f"{oc}.sub<{span.cpp}>({cs[0]})", span
                return f"{oc}.chains<{span.cpp}>({cs[0]}, {cs[1]})", TList(span)
            if m == "is_anc" and len(cs) == 2:
                return f"{oc}.is_anc({cs[0]}, {cs[1]})", BOOL
            if m == "dist" and len(cs) == 2:     # the length of the path: its edges, or their `weight:` if given
                return f"{oc}.length({cs[0]}, {cs[1]})", (TInt() if ot.weighted else TInt(0, ot.dom.info.len_iv[1]))
            if m == "children" and len(cs) == 1:
                return f"{oc}.kids[{cs[0]}]", TList(ot.dom)
            if m == "size" and len(cs) == 1:
                return f"(ll){oc}.siz[{cs[0]}]", TInt(1, ot.dom.info.len_iv[1])
            self.error(e, f"no method {m} on {ot!r} (lca, dist, is_anc, parent, ancestor, step, children, size, pos, depth, .kids, .root, .preorder)")
        if isinstance(ot, TPrefix) and m == "sum" and len(ot.dims) == 1 and len(args) == 2 and \
                all(a.kind == "kwarg" for a in args) and [a.name for a in args] == ["from", "len"]:
            # pre.sum(from: v, len: k): k cells starting at v, going round past the end (a circle)
            d = ot.dims[0]
            fc, ft = self.expr(args[0].value, d)
            if ft != d:
                self.error(args[0], f"`from:` is a {d!r}, got {ft!r}")
            kc, kt = self.expr(args[1].value)
            if not is_int(kt):
                self.error(args[1], f"`len:` is an Int, got {kt!r}")
            return f"{oc}.wrap({fc}, {kc}, {d.info.len_cpp}, {e.line})", TInt()
        if isinstance(ot, TPrefix) and m == "sum":
            if len(args) != len(ot.dims):
                self.error(e, f"`sum` on {ot!r} takes {len(ot.dims)} range(s), one per dimension")
            cs = []
            for a, d in zip(args, ot.dims):
                c, t = self.expr(a)
                if not (isinstance(t, TRange) and t.elem == d):
                    self.error(a, f"expected a range of {d!r}, got {t!r}")
                cs.append(c)
            if len(cs) == 1:
                cs.append("Rng{0, 1}")
            return f"{oc}.sum({cs[0]}, {cs[1]})", TInt()
        if isinstance(ot, THash) and m == "of" and len(args) == 1:
            # hash.of(range): the hash of a window, the same value as hash.of(first, last)
            c, t = self.expr(args[0])
            if not (isinstance(t, TRange) and t.elem == ot.dom):
                self.error(args[0], f"hash.of takes a range of {ot.dom!r} (or two positions), got {t!r}")
            self.note_check(e, f"`{unparse(args[0])}` is not empty")
            return (f"([&](Rng _r) {{ return {oc}.of((int)cp_rfirst(_r, {e.line}), (int)cp_rlast(_r, {e.line})); }}({c}))",
                    TInt(0, (1 << 61) - 1))
        if isinstance(ot, THash) and m == "of" and len(args) == 2:
            cs = []
            for a in args:
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"positions of this string are {ot.dom!r}, got {t!r}")
                cs.append(c)
            return f"{oc}.of({cs[0]}, {cs[1]})", TInt(0, (1 << 61) - 1)
        if isinstance(ot, TAssignSum):
            def at(a):
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"a {ot!r} is indexed by {ot.dom!r}, got {t!r}")
                return c
            if m == "assign" and len(args) == 3:
                vc, vt = self.expr(args[2])
                if not is_int(vt):
                    self.error(args[2], f"assign(l, r, x) needs an Int x, got {vt!r}")
                return f"{oc}.assign({at(args[0])}, {at(args[1])}, {vc})", UNIT
            if m == "sum" and len(args) == 2:
                return f"{oc}.sum({at(args[0])}, {at(args[1])})", INT
            self.error(e, f"no method {m} on {ot!r} (assign(l, r, x), sum(l, r))")
        if isinstance(ot, TAddMax):
            def at(a):
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"a {ot!r} is indexed by {ot.dom!r}, got {t!r}")
                return c
            if m == "add" and len(args) == 3:
                vc, vt = self.expr(args[2])
                if not is_int(vt):
                    self.error(args[2], f"add(l, r, x) needs an Int x, got {vt!r}")
                return f"{oc}.add({at(args[0])}, {at(args[1])}, {vc})", UNIT
            if m == "max" and len(args) == 2:
                return f"{oc}.query({at(args[0])}, {at(args[1])})", INT
            if m == "max" and not args:
                return f"{oc}.top()", INT
            self.error(e, f"no method {m} on {ot!r} (add(l, r, x), max(l, r), max())")
        if isinstance(ot, TRt) and ot.name in ("Fenwick", "SparseTable", "LazySegTree"):
            d = ot.args[0]
            def at(a):
                c, t = self.expr(a, d)
                if t != d:
                    self.error(a, f"a {ot.name} over {d!r} is indexed by {d!r}, got {t!r}")
                return c
            if ot.name == "Fenwick" and m == "add" and len(args) == 2:
                self.require_mut(obj, e)
                return f"{oc}.add({at(args[0])}, {self.str_pos(args[1])})", UNIT
            if ot.name == "Fenwick" and m == "sum" and len(args) == 1:
                return f"{oc}.sum({self.dom_range(args[0], d)})", TInt()
            if ot.name == "LazySegTree" and m == "apply" and len(args) == 2:
                self.require_mut(obj, e)
                return f"{oc}.apply({self.dom_range(args[0], d)}, {self.str_pos(args[1])})", UNIT
            if ot.name != "Fenwick" and m == "fold" and len(args) == 1:
                if ot.name == "SparseTable":
                    self.note_check(e, "range is non-empty")
                    return f"{oc}.fold({self.dom_range(args[0], d)}, {e.line})", TInt()
                return f"{oc}.fold({self.dom_range(args[0], d)})", TInt()
            self.error(e, f"no method `{m}` on {ot.name} (" + {"Fenwick": "add(i, d), sum(l..=r)", "SparseTable": "fold(l..=r)",
                                                              "LazySegTree": "apply(l..=r, x), fold(l..=r)"}[ot.name] + ")")
        if isinstance(ot, TSeg) and m == "fold" and len(args) == 1:
            # st.fold(l..=r): the fold of a range of positions (the identity when it is empty)
            return f"([&](Rng _r) {{ return _r.lo < _r.hi ? {oc}.query((int)_r.lo, (int)_r.hi - 1) : {oc}.id; }}({self.dom_range(args[0], ot.dom)}))", ot.elem
        if isinstance(ot, TSeg):
            def at(a):
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"a {ot!r} is indexed by {ot.dom!r}, got {t!r}")
                return c
            if m == "set" and len(args) == 2:
                vc, vt = self.expr(args[1], ot.elem)
                return f"{oc}.set({at(args[0])}, {self.coerce(vc, vt, ot.elem, args[1])})", UNIT
            if m == "get" and len(args) == 1:
                return f"{oc}.get({at(args[0])})", ot.elem
            if m == "query" and len(args) == 2:
                return f"{oc}.query({at(args[0])}, {at(args[1])})", ot.elem
            if m == "refresh" and len(args) == 1:
                return f"{oc}.refresh({at(args[0])})", UNIT
            if m in ("max_right", "min_left") and len(args) == 2 and args[1].kind == "lambda" and len(args[1].params) == 1:
                pn, bc, bt = self.lambda_key(args[1], ot.elem)
                return f"{oc}.{m}({at(args[0])}, [&]({self.cpp_type(ot.elem)} {pn}) {{ return ({bc}); }})", TOpt(ot.dom)
            self.error(e, f"no method {m} on {ot!r} (set(i, v), get(i), query(l, r), refresh(i), max_right, min_left, .all)")
        if isinstance(ot, TDsu) and m in ("find", "unite", "same", "size", "mark", "undo_to"):
            if m == "mark" and not args:
                return f"(ll){oc}.mark()", TInt(0)
            if m == "undo_to" and len(args) == 1:
                sc, st = self.expr(args[0])
                if not is_int(st):
                    self.error(args[0], f"Dsu.undo_to expects a mark Int, got {st!r}")
                return f"{oc}.undo_to((int)({sc}))", UNIT
            cs = []
            for a in args:
                c, t = self.expr(a, ot.dom)
                if t != ot.dom:
                    self.error(a, f"this Dsu is over {ot.dom!r}, got {t!r}")
                cs.append(c)
            if len(cs) != (1 if m in ("find", "size") else 2):
                self.error(e, f"Dsu.{m}: wrong number of arguments")
            if m == "find": return f"{oc}.find({cs[0]})", ot.dom
            if m == "size": return f"(ll){oc}.size({cs[0]})", TInt(1, ot.dom.info.len_iv[1])
            return f"{oc}.{m}({cs[0]}, {cs[1]})", BOOL
        if isinstance(ot, TList) and m == "swap" and len(args) == 2:
            self.require_mut(obj, e)
            ic, it = self.expr(args[0])
            jc, jt = self.expr(args[1])
            return f"([&](ll _i, ll _j){{ swap(cp_at({oc}, _i, {e.line}), cp_at({oc}, _j, {e.line})); }}({ic}, {jc}))", UNIT
        if m == "take" and not args and isinstance(ot, (TList, TStr, TMap, TSet, TRt)):
            self.require_mut(obj, e)
            return f"cp_take({oc})", ot
        if m == "absorb" and len(args) == 1 and isinstance(ot, (TList, TStr, TMap, TSet)):
            self.require_mut(obj, e)
            self.require_mut(args[0], e)
            bc, bt = self.expr(args[0])
            if bt != ot:
                self.error(args[0], f"absorb expects {ot!r}, got {bt!r}")
            fn = "cp_absorb_vec" if isinstance(ot, (TList, TStr)) else "cp_absorb_map" if isinstance(ot, TMap) else "cp_absorb_set"
            return f"{fn}({oc}, {bc})", UNIT
        if isinstance(ot, TType) and isinstance(ot.t, TEnum):
            k, v = ot.t.variant(m)
            if k is None:
                self.error(e, f"enum {ot.t.name} has no variant `{m}`")
            vn, fns, fts = v
            if len(args) != len(fns):
                self.error(e, f"{ot.t.name}.{vn} takes {len(fns)} argument(s), got {len(args)}")
            inits = []
            for a, fnm, ft in zip(args, fns, fts):
                c, t = self.expr(a, ft)
                inits.append(f"_e.{vn}_{fnm} = {self.coerce(c, t, ft, a)};")
            return f"([&]{{ {ot.t.cpp} _e; _e.tag = {k}; {' '.join(inits)} return _e; }}())", ot.t
        if isinstance(ot, TType) and isinstance(ot.t, TRt) and ot.t.name == "Dir4" and m == "from" and len(args) == 2:
            sc, _ = self.expr(args[0])
            cc, _ = self.expr(args[1])
            return f"CpDir4::from({sc}, {cc}, {e.line})", ot.t
        if isinstance(ot, TTuple) and len(ot.types) == 2 and all(isinstance(ft, TDom) for ft in ot.types) and m == "step" and len(args) == 1:
            dc, dt = self.expr(args[0])
            if not (isinstance(dt, TRt) and dt.name in ("Dir4", "Dir8")):
                self.error(args[0], f".step takes a Dir4 or Dir8, got {dt!r}")
            rinfo, cinfo = ot.types[0].info, ot.types[1].info
            rn, cn = self.field_cpp(ot.names[0]), self.field_cpp(ot.names[1])
            cty = self.cpp_type(TOpt(ot))
            return (f"([&](const {ot.cpp}& _p, auto _d) -> {cty} {{ "
                    f"int _r = _p.{rn} + _d.dr(), _c = _p.{cn} + _d.dc(); "
                    f"if (_r < 0 || _r >= {rinfo.len_cpp} || _c < 0 || _c >= {cinfo.len_cpp}) return {cty}(); "
                    f"return {cty}({ot.cpp}{{_r, _c}}); }}({oc}, {dc}))"), TOpt(ot)
        if isinstance(ot, TRt) and ot.name == "Fenwick2D":
            if m == "add" and len(args) == 3:
                self.require_mut(obj, e)
                ri = self.index_into(args[0], ot.args[0].info, e)
                ci = self.index_into(args[1], ot.args[1].info, e)
                vc, vt = self.expr(args[2], ot.args[2])
                return f"{oc}.add({ri}, {ci}, {self.coerce(vc, vt, ot.args[2], args[2])})", UNIT
            if m == "sum" and len(args) == 2:
                return f"{oc}.sum({self.dom_range(args[0], ot.args[0])}, {self.dom_range(args[1], ot.args[1])})", ot.args[2]
            self.error(e, f"no method {m} on Fenwick2D")
        if isinstance(ot, TRt) and ot.name == "RollingHash":
            if m == "get" and len(args) == 1:
                rc, rt = self.expr(args[0])
                return f"{oc}.get({rc})", TInt(0)
            self.error(e, f"no method {m} on RollingHash")
        if isinstance(ot, TRt) and ot.name == "Pt":
            if m in ("dot", "cross") and len(args) == 1:
                bc, _ = self.expr(args[0])
                return f"{oc}.{m}({bc})", TInt()
            self.error(e, f"no method {m} on Pt")
        if isinstance(ot, TRt) and ot.name == "Matrix":
            if m == "pow" and len(args) == 1:
                ec, _ = self.expr(args[0])
                return f"{oc}.pow({ec})", ot
            self.error(e, f"no method {m} on Matrix")
        if isinstance(ot, TPerm):
            if m == "inv" and not args:
                return f"cp_perm_inv({oc})", ot
            if m == "cycles" and not args:
                return f"cp_perm_cycles({oc})", TList(TList(ot.elem))
        if isinstance(ot, TArr) and len(ot.dims) == 2 and m == "col" and len(args) == 1:
            ci = self.index_into(args[0], ot.dims[1].info, e)
            return f"cp_grid_col({oc}, {ot.dims[0].info.len_cpp}, {ot.dims[1].info.len_cpp}, {ci})", TArr([ot.dims[0]], ot.elem)
        if isinstance(ot, TRt) and ot.name == "Bitset":
            if m == "clear" and not args:
                self.require_mut(obj, e)
                return f"{oc}.clear()", UNIT
            if m in ("add", "remove", "toggle") and len(args) == 1:
                self.require_mut(obj, e)
                ic = self.index_into(args[0], ot.args[0].info, e)
                return f"{oc}.{m}({ic})", (UNIT if m == "toggle" else BOOL)
            if m == "shifted" and len(args) == 1:
                kc, _ = self.expr(args[0])
                return f"{oc}.shifted({kc})", ot
            self.error(e, f"no method {m} on Bitset")
        if isinstance(ot, TRt) and ot.name == "Marks":
            if m == "clear" and not args:
                self.require_mut(obj, e)
                return f"{oc}.clear()", UNIT
            if m in ("add", "remove") and len(args) == 1:
                self.require_mut(obj, e)
                ic = self.index_into(args[0], ot.args[0].info, e)
                return f"{oc}.{m}({ic})", BOOL
            self.error(e, f"no method {m} on Marks")
        if isinstance(ot, TRt) and ot.name == "Deque":
            el = self.cpp_type(TOpt(ot.elem))
            if m in ("push", "push_front") and len(args) == 1:
                self.require_mut(obj, e)
                c, t = self.expr(args[0], ot.elem)
                return f"{oc}.{'push_back' if m == 'push' else 'push_front'}({self.coerce(c, t, ot.elem, args[0])})", UNIT
            if m in ("pop", "pop_front", "pop_back") and not args:
                self.require_mut(obj, e)
                end = "front" if m == "pop_front" else "back"
                return f"([&]{{ if ({oc}.empty()) return {el}(); auto _v = {oc}.{end}(); {oc}.pop_{end}(); return {el}(_v); }}())", TOpt(ot.elem)
            if m == "clear" and not args:
                self.require_mut(obj, e)
                return f"{oc}.clear()", UNIT
            self.error(e, f"no method `{m}` on {ot!r} (push, pop, push_front, pop_front, clear, .first, .last, .len, q[i])")
        if isinstance(ot, TMod) and m == "pow" and len(args) == 1:
            return f"({oc}).pow({self.str_pos(args[0])})", ot
        if isinstance(ot, TRt) and ot.name == "Comb":
            want_n = {"fact": 1, "inv_fact": 1, "choose": 2, "arrange": 2}.get(m)
            if want_n != len(args):
                self.error(e, f"no method `{m}` on {ot!r} (fact(k), inv_fact(k), choose(n, k), arrange(n, k))")
            return f"{oc}.{m}({', '.join(self.str_pos(a) for a in args)}, {e.line})", ot.elem
        if isinstance(ot, TRt) and ot.name == "MonoDeque":
            kw = {a.name: a.value for a in args if a.kind == "kwarg"}
            if m == "push" and set(kw) == {"at", "val"} and len(args) == 2:
                self.require_mut(obj, e)
                c, t = self.expr(kw["val"], ot.elem)
                return f"{oc}.push({self.str_pos(kw['at'])}, {self.coerce(c, t, ot.elem, kw['val'])})", UNIT
            if m == "drop_before" and set(kw) == {"min_at"} and len(args) == 1:
                self.require_mut(obj, e)
                return f"{oc}.drop_before({self.str_pos(kw['min_at'])})", UNIT
            if m == "clear" and not args:
                self.require_mut(obj, e)
                return f"{oc}.clear()", UNIT
            self.error(e, f"no method `{m}` on {ot!r} (push(at: i, val: x), drop_before(min_at: i), clear(), .best)")
        if isinstance(ot, THeap):
            if m == "clear" and not args:
                return f"{oc} = {self.cpp_type(ot)}()", UNIT
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
        if isinstance(ot, TDom) and m == "cyc" and len(args) == 1:
            # v.cyc(k): k steps round the domain, wrapping (plan/19 §3.2). Euclidean, so k may be negative.
            c, t = self.expr(args[0])
            if not is_int(t):
                self.error(e, f"cyc needs an Int number of steps, got {t!r}")
            n = ot.info.len_cpp
            if t.lo is not None and t.lo >= 0:
                return f"cp_cyc_nonneg({oc}, {c}, (int)({n}))", ot
            return f"cp_cyc({oc}, {c}, (int)({n}))", ot
        if isinstance(ot, TOpt) and m == "map":
            self.error(e, ".map on ?T is not in the prototype yet")
        self.error(e, f"{ot!r} has no method `{m}`")

    def windows_of(self, oc, ot, k, e):
        """(range C++, element type, k C++) for `r.windows(k)` / `D.windows(k)`."""
        if isinstance(ot, TType):
            oc, el = f"Rng{{0, (ll){ot.t.info.len_cpp}}}", ot.t
        else:
            el = ot.elem
        kc, kt = self.expr(k)
        if not is_int(kt):
            self.error(e, f"`.windows(k)` takes an Int width, got {kt!r}")
        if kt.lo is None or kt.lo < 1:
            self.note_check(e, f"`{unparse(k)}` is at least 1")
            kc = f"cp_width({kc}, {e.line})"
        return oc, el, kc

    # ---------- iteration
    def bind_pattern(self, pat, c, t, node, mut=False, owned=False):
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
        # a `let` owns its value: the names must not point into a list that a later push may move
        self.emit(f"const auto{'' if owned else '&'} {tmp} = {c};")
        for sub, fn, ft in zip(pat.items, t.names, t.types):
            self.bind_pattern(sub, f"{tmp}.{self.field_cpp(fn)}", ft, node, mut=mut)

    def loop(self, pat, it, node, body):
        # a read() inside a top-level loop: the input ends where the outermost loop ends
        self.loop_nest = getattr(self, "loop_nest", 0) + 1
        mark, rmark = self.last_read, getattr(self, "read_pos", None)
        held = self.hold_iterated(it, node)
        try:
            self._loop(pat, it, node, body)
        finally:
            self.loop_nest -= 1
            if held is not None:
                self.holds.remove(held)
        if self.loop_nest == 0 and self.last_read != mark and self.out is self.init_lines:
            self.last_read = len(self.init_lines)
        if self.loop_nest == 0 and getattr(self, "read_pos", None) != rmark:
            self.read_pos = (id(self.out), len(self.out))       # in a fn too: the input ends where the loop ends

    def _loop(self, pat, it, node, body):
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
        if it.kind == "field" and it.name == "subsets":
            c, t = self.expr(it.obj)
            if not (isinstance(t, TDom) and getattr(t.info, "subset_of", None) is not None):
                self.error(it, f"`.subsets` needs a Subset<D> value, got {t!r}")
            if rev:
                self.error(it, "`.subsets` runs from the whole set down to the empty set; `.rev` is not in the prototype")
            s_, v, done = self.tmpname("s"), self.fresh(pat.name if pat.kind == "pname" else "g"), self.tmpname("done")
            # every subset of s, s itself first and the empty set last (the standard (g - 1) & s walk)
            self.emit(f"for (int {s_} = {c}, {v} = {s_}, {done} = 0; !{done}; {done} = !{v}, {v} = ({v} - 1) & {s_}) {{")
            self.scopes.append({})
            self.loop_trips.append(None)
            self.ind += 1
            if pat.kind != "pname":
                self.error(it, "`for g in s.subsets` binds one name")
            self.scopes[-1][pat.name] = Sym("let", t, v, node=pat)
            body()
            self.ind -= 1
            self.loop_trips.pop()
            self.scopes.pop()
            self.emit("}")
            return
        if it.kind == "call" and it.fn.kind == "field" and it.fn.name == "windows" and len(it.args) == 1 and not rev:
            oc, ot = self.expr(it.fn.obj)
            if isinstance(ot, TRange) or (isinstance(ot, TType) and isinstance(ot.t, TDom)):
                # `for w in D.windows(k)`: no list is built
                rc, el, kc = self.windows_of(oc, ot, it.args[0], it)
                r, k, v = self.tmpname("r"), self.tmpname("k"), self.tmpname("s")
                if pat.kind != "pname":
                    self.error(it, "a window is one Range: `for w in D.windows(k)`")
                self.emit(f"{{ const Rng {r} = {rc}; const ll {k} = {kc};")
                self.emit(f"for (ll {v} = {r}.lo; {v} + {k} <= {r}.hi; ++{v}) {{")
                self.scopes.append({})
                self.loop_trips.append(el.info.len_iv[1] if isinstance(el, TDom) else None)
                self.ind += 1
                w = self.fresh(pat.name)
                self.emit(f"const Rng {w}{{{v}, {v} + {k}}};")
                self.scopes[-1][pat.name] = Sym("let", TRange(el), w, node=pat)
                body()
                self.ind -= 1
                self.loop_trips.pop()
                self.scopes.pop()
                self.emit("} }")
                return
        if it.kind == "call" and it.fn.kind == "name" and it.fn.name == "slide_until" and len(it.args) == 2 \
                and not self.lookup("slide_until"):
            # `for (lo, hi) in slide_until(r, (l, r) => ok)`: for each lo in turn, the first hi (never moving
            # back) at which ok(lo, hi) holds; it stops when no hi is left. Stateless two pointers.
            rc, rt = self.expr(it.args[0])
            lam = it.args[1]
            if not isinstance(rt, TRange) or lam.kind != "lambda" or len(lam.params) != 2 or pat.kind != "ptuple" or len(pat.items) != 2 or rev:
                self.error(it, "`for (lo, hi) in slide_until(range, (l, r) => ok)`")
            el = rt.elem
            cty = "int" if isinstance(el, TDom) else "ll"
            r, lo, hi = self.tmpname("r"), self.tmpname("lo"), self.tmpname("hi")
            self.emit(f"{{ const Rng {r} = {rc}; ll {hi} = {r}.lo;")
            self.emit(f"for (ll {lo} = {r}.lo; {lo} < {r}.hi; ++{lo}) {{")
            self.ind += 1
            self.emit(f"if ({hi} < {lo}) {hi} = {lo};")
            self.scopes.append({})
            pa, pb = self.fresh("l"), self.fresh("r")
            self.bind_lambda_param(lam.params[0], pa, el, lam)
            self.bind_lambda_param(lam.params[1], pb, el, lam)
            lines, (bc, bt) = self.capture(lambda: self.expr(lam.body, BOOL))
            self.scopes.pop()
            if not isinstance(bt, TBool):
                self.error(lam, f"slide_until needs a Bool test, got {bt!r}")
            self.emit(f"auto {r}_ok = [&](const {cty} {pa}, const {cty} {pb}) -> bool {{ {NL.join(lines)} return {bc}; }};")
            self.emit(f"while ({hi} < {r}.hi && !{r}_ok(({cty}){lo}, ({cty}){hi})) ++{hi};")
            self.emit(f"if ({hi} >= {r}.hi) break;")
            self.scopes.append({})
            self.loop_trips.append(None)
            self.bind_pattern(pat.items[0], f"({cty}){lo}", el, node)
            self.bind_pattern(pat.items[1], f"({cty}){hi}", el, node)
            body()
            self.loop_trips.pop()
            self.scopes.pop()
            self.ind -= 1
            self.emit("} }")
            return
        if it.kind == "call" and it.fn.kind == "name" and it.fn.name == "read_until" and len(it.args) == 1 and not self.lookup("read_until"):
            sc, st = self.expr(it.args[0])
            ru = self.tmpname("ru")
            sen = self.tmpname("sen")
            self.emit(f"{{ const auto {sen} = {sc}; while (true) {{ {self.cpp_type(st)} {ru};")
            self.ind += 1
            self.emit_read(ru, st, node)
            self.emit(f"if ({ru} == {sen}) break;")
            self.scopes.append({})
            self.loop_trips.append(None)
            self.bind_pattern(pat, ru, st, node)
            body()
            self.loop_trips.pop()
            self.scopes.pop()
            self.ind -= 1
            self.emit("} }")
            return
        if it.kind == "call" and it.fn.kind == "field" and it.fn.name == "ray" and len(it.args) == 1:
            pc, pt = self.expr(it.fn.obj)
            dc, dt = self.expr(it.args[0])
            if isinstance(pt, TTuple) and len(pt.types) == 2 and all(isinstance(ft, TDom) for ft in pt.types):
                rinfo, cinfo = pt.types[0].info, pt.types[1].info
                rn, cn = self.field_cpp(pt.names[0]), self.field_cpp(pt.names[1])
                pr, dr_, rr, cc = self.tmpname("p"), self.tmpname("d"), self.tmpname("r"), self.tmpname("c")
                self.emit(f"{{ auto {pr} = {pc}; auto {dr_} = {dc};")
                self.emit(f"for (int {rr} = {pr}.{rn} + {dr_}.dr(), {cc} = {pr}.{cn} + {dr_}.dc(); "
                          f"{rr} >= 0 && {rr} < {rinfo.len_cpp} && {cc} >= 0 && {cc} < {cinfo.len_cpp}; "
                          f"{rr} += {dr_}.dr(), {cc} += {dr_}.dc()) {{")
                self.scopes.append({})
                self.loop_trips.append(None)
                self.ind += 1
                self.bind_pattern(pat, f"{pt.cpp}{{{rr}, {cc}}}", pt, node)
                body()
                self.ind -= 1
                self.loop_trips.pop()
                self.scopes.pop()
                self.emit("} }")
                return
        if it.kind == "field" and it.name in ("adj4", "adj8"):
            pc, pt = self.expr(it.obj)
            if isinstance(pt, TTuple) and len(pt.types) == 2 and all(isinstance(ft, TDom) for ft in pt.types):
                rinfo, cinfo = pt.types[0].info, pt.types[1].info
                rn, cn = self.field_cpp(pt.names[0]), self.field_cpp(pt.names[1])
                n_dir = 4 if it.name == "adj4" else 8
                dcls = "CpDir4" if it.name == "adj4" else "CpDir8"
                pr, dk, rr, cc = self.tmpname("p"), self.tmpname("k"), self.tmpname("r"), self.tmpname("c")
                self.emit(f"{{ auto {pr} = {pc}; for (int {dk} = 0; {dk} < {n_dir}; ++{dk}) {{ "
                          f"int {rr} = {pr}.{rn} + {dcls}::DR[{dk}], {cc} = {pr}.{cn} + {dcls}::DC[{dk}]; "
                          f"if ({rr} < 0 || {rr} >= {rinfo.len_cpp} || {cc} < 0 || {cc} >= {cinfo.len_cpp}) continue;")
                self.scopes.append({})
                self.loop_trips.append(n_dir)
                self.ind += 1
                self.bind_pattern(pat, f"{pt.cpp}{{{rr}, {cc}}}", pt, node)
                body()
                self.ind -= 1
                self.loop_trips.pop()
                self.scopes.pop()
                self.emit("} }")
                return
        if it.kind == "field" and it.name == "adjacent" and pat.kind == "ptuple" and len(pat.items) == 2:
            oc, ot = self.expr(it.obj)
            if isinstance(ot, (TList, TArr)) and not (isinstance(ot, TArr) and len(ot.dims) != 1):
                a, k = self.tmpname("a"), self.tmpname("k")
                self.emit(f"{{ const auto& {a} = {oc};")
                self.emit(f"for (ll {k} = 0; {k} + 1 < (ll){a}.size(); ++{k}) {{")
                self.scopes.append({})
                self.loop_trips.append(None)
                self.ind += 1
                self.bind_pattern(pat.items[0], f"{a}[{k}]", ot.elem, node)
                self.bind_pattern(pat.items[1], f"{a}[{k} + 1]", ot.elem, node)
                body()
                self.ind -= 1
                self.loop_trips.pop()
                self.scopes.pop()
                self.emit("} }")
                return
        if it.kind == "field" and it.name == "subsets":
            sc, st = self.expr(it.obj)
            if isinstance(st, TDom) and getattr(st.info, "subset_of", None) is not None:
                m_ = self.tmpname("m")
                v = self.fresh(pat.name if pat.kind == "pname" else "s")
                self.emit(f"for (int {m_} = {sc}, {v} = {m_}, {m_}_go = 1; {m_}_go; {m_}_go = ({v} != 0), {v} = ({v} - 1) & {m_}) {{")
                self.scopes[-1][pat.name] = Sym("let", st, v, node=pat)
                self.ind += 1
                body()
                self.ind -= 1
                self.emit("}")
                self.scopes.pop()
                return
        if it.kind == "call" and it.fn.kind == "field" and it.fn.name == "step" and len(it.args) == 1:
            # `for i in (0..n).step(2)` and `.step(2).rev`: every k-th value, counted from the low end
            rc, rt = self.expr(it.fn.obj)
            if not isinstance(rt, TRange) or pat.kind != "pname":
                self.error(it, f"`.step(k)` steps through a range and binds one name, got {rt!r}")
            kc, kt = self.expr(it.args[0])
            if not is_int(kt) or kt.lo is None or kt.lo < 1:
                self.error(it, "the step of `.step(k)` must be an Int known to be at least 1")
            r, k = self.tmpname("r"), self.tmpname("k")
            v = self.fresh(pat.name)
            cty = "int" if isinstance(rt.elem, TDom) else "ll"
            self.emit(f"{{ const Rng {r} = {rc}; const ll {k} = {kc};")
            if rev:
                self.emit(f"if ({r}.lo < {r}.hi) for (ll {v}_ = {r}.lo + ({r}.hi - 1 - {r}.lo) / {k} * {k}; {v}_ >= {r}.lo; {v}_ -= {k}) {{ const {cty} {v} = ({cty}){v}_;")
            else:
                self.emit(f"for (ll {v}_ = {r}.lo; {v}_ < {r}.hi; {v}_ += {k}) {{ const {cty} {v} = ({cty}){v}_;")
            self.scopes.append({})
            self.scopes[-1][pat.name] = Sym("let", rt.elem, v, node=pat)
            self.loop_trips.append(None)
            self.ind += 1
            body()
            self.ind -= 1
            self.loop_trips.pop()
            self.scopes.pop()
            self.emit("} }")
            return
        if it.kind == "field" and it.name == "items" and pat.kind == "ptuple" and len(pat.items) == 2:
            # `for (i, x) in a.items`: (index, value) of an array or list, (key, value) of a map
            ot = self.capture(lambda: self.expr(it.obj))[1][1]
            if isinstance(ot, TMap):
                it = it.obj
            elif isinstance(ot, TList) or (isinstance(ot, TArr) and len(ot.dims) == 1):
                oc, ot = self.expr(it.obj)
                a, k = self.tmpname("a"), self.tmpname("k")
                self.emit(f"{{ const auto& {a} = {oc};")
                self.emit(f"for (ll {k} = (ll){a}.size() - 1; {k} >= 0; --{k}) {{" if rev else
                          f"for (ll {k} = 0; {k} < (ll){a}.size(); ++{k}) {{")
                self.scopes.append({})
                self.loop_trips.append(ot.dims[0].info.len_iv[1] if isinstance(ot, TArr) else None)
                self.ind += 1
                if isinstance(ot, TArr) and not ot.dims[0].info.anon:
                    self.bind_pattern(pat.items[0], f"(int){k}", ot.dims[0], node)
                else:
                    lo = ot.dims[0].info.lo_cpp if isinstance(ot, TArr) else "0"
                    self.bind_pattern(pat.items[0], f"(ll)({k} + {lo})", TInt(), node)
                self.bind_pattern(pat.items[1], f"{a}[{k}]", ot.elem, node)
                body()
                self.ind -= 1
                self.loop_trips.pop()
                self.scopes.pop()
                self.emit("} }")
                return
        self.scopes.append({})
        trips = None
        closed = it.kind == "range" and it.lo is not None and it.hi is not None
        if closed:      # a boundary at either end, or an open end, makes it a Range value: iterate that
            ends = [self.capture(lambda x=x: self.expr(x))[1][1] for x in (it.lo, it.hi)]
            closed = not any(isinstance(t, TDom) and t.info.of is not None for t in ends)
        if closed:
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
                # a range is a value: its end is read once, before the first step (decisions 13b), so a body
                # that grows the list the end was computed from does not move the end
                if re.fullmatch(r"\(ll\)\((\w+|-?\d+LL)\)( \+ 1)?", hi_x):
                    self.emit(f"for ({cty} {v} = {lo}; {v} < {hi_x}; ++{v}) {{")
                else:
                    self.emit(f"for ({cty} {v} = {lo}, {v}_end = {hi_x}; {v} < {v}_end; ++{v}) {{")
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
            elif isinstance(t, TType) and isinstance(t.t, TRt) and t.t.name in ("Dir4", "Dir8"):
                nd = 4 if t.t.name == "Dir4" else 8
                trips = nd
                k = self.tmpname("k")
                v = self.fresh(pat.name if pat.kind == "pname" else "d")
                self.emit(f"for (int {k} = 0; {k} < {nd}; ++{k}) {{ Cp{t.t.name} {v}({k});")
                self.scopes[-1][pat.name] = Sym("let", t.t, v, node=pat)
            elif isinstance(t, TType) and isinstance(t.t, TBool):
                trips = 2
                v = self.fresh(pat.name if pat.kind == "pname" else "b")
                self.emit(f"for (bool {v} : {{{'true, false' if rev else 'false, true'}}}) {{")
                self.scopes[-1][pat.name] = Sym("let", BOOL, v, node=pat)
            elif isinstance(t, TType) and isinstance(t.t, TTuple) and t.t.cpp in self.index_impls:
                # `for c in Cell`: every value of a user Index type, in to_idx order
                info = self.index_impls[t.t.cpp]
                trips = info.len_iv[1]
                k, v = self.tmpname("k"), self.tmpname("x")
                self.emit(f"for (ll {k} = {info.len_cpp} - 1; {k} >= 0; --{k}) {{" if rev else
                          f"for (ll {k} = 0; {k} < {info.len_cpp}; ++{k}) {{")
                self.ind += 1
                self.emit(f"const auto {v} = {self.lookup(info.from_idx).cpp}({k});")
                self.bind_pattern(pat, v, t.t, node)
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
            elif isinstance(t, TDom) and getattr(t.info, "subset_of", None) is not None:
                v = self.fresh(pat.name if pat.kind == "pname" else "i")
                m_ = self.tmpname("m")
                self.emit(f"for (int {m_} = {c}; {m_}; {m_} &= {m_} - 1) {{ int {v} = __builtin_ctz({m_});")
                self.scopes[-1][pat.name] = Sym("let", TDom(t.info.subset_of), v, node=pat)
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
            elif isinstance(t, TStr):
                v = self.tmpname("x")
                r = self.tmpname("r")
                self.emit(f"const auto& {r} = {c};")
                self.emit(f"for (auto {r}i = {r}.rbegin(); {r}i != {r}.rend(); ++{r}i) {{ const char {v} = *{r}i;" if rev else
                          f"for (const char {v} : {r}) {{")
                self.ind += 1
                self.bind_pattern(pat, v, CHAR, node)
                self.ind -= 1
            elif isinstance(t, (TArr, TList)):
                if isinstance(t, TArr) and len(t.dims) == 1:
                    trips = t.dims[0].info.len_iv[1]
                el = t.elem
                v = self.tmpname("x")
                fmut = getattr(node, "mut", False)
                if fmut:
                    self.require_mut(it, node)
                    ref = "auto&"
                else:
                    ref = "const auto&"
                if rev:
                    # lists have no domain to reverse, so `for x in xs.rev` walks back to front
                    r = self.tmpname("r")
                    self.emit(f"{ref} {r} = {c};")
                    self.emit(f"for (auto {r}i = {r}.rbegin(); {r}i != {r}.rend(); ++{r}i) {{ {ref} {v} = *{r}i;")
                else:
                    self.emit(f"for ({ref} {v} : {c}) {{")
                self.ind += 1
                if fmut and pat.kind == "pname":
                    self.scopes[-1][pat.name] = Sym("var", el, v, mut=True, node=pat)
                else:
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
        elif rt is None and ctx[1] in getattr(self, "top_returns", ()):     # a top-level `return` in the body
            self.emit(f"if ({ctx[1]}) goto {self.program_return()};")
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
            if it.kind == "field" and it.name in ("subsets", "adjacent", "adj4", "adj8"):
                return None
            if it.kind == "call" and it.fn.kind == "field" and it.fn.name == "ray":
                return None
            if it.kind == "call" and it.fn.kind == "field" and it.fn.name == "step":
                return None
            if it.kind == "range" and (it.lo is None or it.hi is None):
                rt = self.capture(lambda: self.expr(it))[1][1]
                t = rt.elem.info.len_iv[1] if isinstance(rt.elem, TDom) else None
            elif it.kind == "range":
                lo, lt = self.capture(lambda: self.expr(it.lo))[1]
                hi, ht = self.capture(lambda: self.expr(it.hi))[1]
                if isinstance(lt, TDom):
                    t = (lt.info.of or lt.info).len_iv[1]
                elif isinstance(ht, TDom):
                    t = (ht.info.of or ht.info).len_iv[1]
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
        if name == "prod":
            name = "product"
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
                if isinstance(t, TOpt) and isinstance(t.inner, (TInt, TF64)):
                    box["opt_sum"] = t.inner
                    ct = self.cpp_type(TOpt(TInt() if is_int(t.inner) else t.inner))
                    self.emit(f"{{ auto _o = {c}; if (!_o.has()) {acc} = {ct}(); else if ({acc}.has()) {acc} = {ct}({acc}.get() + _o.get()); }}")
                else:
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
                cmp = "<" if name == "min" else ">"
                if isinstance(t, TOpt):
                    # min/max skip `none` elements (none = "no value here"), result is ?T
                    box["t"], box["opt"] = t.inner, True
                    self.emit(f"{{ auto _o = {c}; if (_o.has() && (!{acc}.has() || _o.get() {cmp} {acc}.get())) {acc} = _o; }}")
                    return
                op_fn = "opt_min" if name == "min" else "opt_max"
                self.emit(f"{op_fn}({acc}, {c});")
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
            res = ""
            if not has_filter:
                # Pre-reserve when every `for` clause has an independent cheap size expression
                factors, bound = [], set()
                def pat_names(p):
                    if p.kind == "pname": bound.add(p.name)
                    elif p.kind == "ptuple":
                        for x in p.items: pat_names(x)
                def uses_bound(nd):
                    if nd is None: return False
                    if getattr(nd, "kind", None) == "name": return nd.name in bound
                    for v in nd.__dict__.values():
                        if isinstance(v, list):
                            if any(uses_bound(x) for x in v): return True
                        elif hasattr(v, "__dict__") and uses_bound(v): return True
                    return False
                ok_res = True
                for cl in clauses:
                    it = cl[2]
                    if it.kind == "field" and it.name == "rev":
                        it = it.obj
                    if uses_bound(it):
                        ok_res = False; break
                    if it.kind == "name":
                        sym = self.lookup(it.name)
                        if sym is not None and isinstance(sym.ty, (TList, TArr)):
                            factors.append(f"(size_t){sym.cpp}.size()")
                        elif sym is not None and isinstance(sym.ty, TType) and isinstance(sym.ty.t, TDom):
                            factors.append(f"(size_t)({sym.ty.t.info.len_cpp})")
                        else:
                            ok_res = False; break
                    elif it.kind == "range" and it.lo is not None and it.hi is not None:
                        _, (lc, lt0) = self.capture(lambda: self.expr(it.lo))
                        _, (hc, ht0) = self.capture(lambda: self.expr(it.hi))
                        if is_int(lt0) and is_int(ht0):
                            end = f"((ll)({hc}) + 1)" if it.incl else f"(ll)({hc})"
                            factors.append(f"(size_t)max(0LL, {end} - (ll)({lc}))")
                        else:
                            ok_res = False; break
                    else:
                        ok_res = False; break
                    pat_names(cl[1])
                if ok_res and factors:
                    res = f" {acc}.reserve({' * '.join(factors)});"
            return f"([&]() {{ {self.cpp_type(lt)} {acc};{res}\n{body}\n return {acc}; }}())", lt
        if name in ("any", "all"):
            return f"([&]() -> bool {{\n{body}\n return {'false' if name == 'any' else 'true'}; }}())", BOOL
        if name == "count":
            return f"([&]() -> ll {{ ll {acc} = 0;\n{body}\n return {acc}; }}())", TInt(0, trips)
        if name in ("sum", "product"):
            if box.get("opt_sum") is not None:
                bi = TInt() if is_int(box["opt_sum"]) else box["opt_sum"]
                rt = TOpt(bi)
                ct = self.cpp_type(rt)
                return f"([&]() {{ {ct} {acc}(0);\n{body}\n return {acc}; }}())", rt
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
        nonempty = not has_filter and not box.get("opt") and self.nonempty(clauses)
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
            if it.kind == "range" and (it.lo is None or it.hi is None):
                return False
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
            if getattr(t, "view", None):
                return False    # a slice keeps its parent's index type, but `a[l..l]` is empty
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
            box["t"] = t if want_el is not None else TInt() if is_int(t) else t
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
        code = (f"([&]() {{ {self.cpp_type(TArr(dims, et))} {tab}({size});\n" + "\n".join(lines) + f"\n return {tab}; }}())")
        return code, TArr(dims, et)

    # ---------- statements
    def block(self, stmts, new_scope=True):
        if new_scope:
            self.scopes.append({})
        saved_rest, nfacts, nholds = self.cur_rest, len(self.facts), len(self.holds)
        for k, s in enumerate(stmts):
            self.cur_rest = stmts[k + 1:]
            self.stmt(s)
            if self.holds:      # a let-bound lend ends after the last statement that mentions it
                self.holds = [h for h in self.holds if h.until is not s]
        self.cur_rest = saved_rest
        del self.facts[nfacts:]
        del self.holds[nholds:]
        if new_scope:
            self.scopes.pop()

    def stmt(self, s):
        m = getattr(self, "s_" + s.kind, None)
        if m is None:
            self.error(s, f"unsupported statement {s.kind}")
        saved = getattr(self, "cur_stmt", None), getattr(self, "after_stmt", [])
        self.cur_stmt, self.after_stmt = s.kind, []
        start = len(self.out)
        try:
            # `let x = e?` is the whole-statement form and has its own code in s_let
            lift = {"let": "value", "return": "value", "exprstmt": "expr", "assign": "value", "doassign": "value"}.get(s.kind)
            if lift and not (s.kind == "let" and s.value is not None and s.value.kind == "try"):
                self.with_tries(s, lift, lambda: m(s))
            else:
                m(s)
            for mark, line in self.after_stmt:
                at = [k for k in range(start, len(self.out)) if self.out[k].strip() == mark]
                if not at:
                    continue
                # the write-back goes where the statement's code ends: the end of the C++ block the mark is in
                k, depth = at[0] + 1, 0
                while k < len(self.out):
                    depth += self.out[k].count("{") - self.out[k].count("}")
                    if depth < 0:
                        break
                    k += 1
                pad = self.out[at[0]][:len(self.out[at[0]]) - len(self.out[at[0]].lstrip())]
                self.out.insert(k, pad + line)
        finally:
            self.cur_stmt, self.after_stmt = saved

    def declare(self, name, ty, cpp, mut, node, kind=None):
        if isinstance(ty, (TSet, TMap, TArr, TOpt, TList)) and node is not None:
            try:
                self.__dict__.setdefault("reprs", []).append((getattr(node, "line", 0), name, repr(ty), self.cpp_type(ty)))
            except Exception:
                pass
        sym = Sym(kind or ("var" if mut else "let"), ty, cpp, mut=mut, node=node)
        self.drop_facts({name})     # a new binding of this name: facts about the old one no longer apply
        self.scopes[-1][name] = sym
        self.decl_depth[id(node)] = len(self.loop_trips)
        return sym

    def is_read(self, v):
        return v is not None and v.kind == "call" and v.fn.kind == "name" and v.fn.name == "read" \
            and all(a.kind == "kwarg" or self.is_columns(a) for a in v.args) and not self.lookup("read")

    def is_columns(self, a):
        """`read(columns)`: an array of records laid out one field after another in the input."""
        return a.kind == "name" and a.name == "columns" and not self.lookup("columns")

    def s_compress(self, s):
        """`let xs = compress(all)`: the distinct values of `all`, sorted, indexed by a fresh domain `xs.D`."""
        c, t = self.expr(s.value.args[0])
        if not isinstance(t, (TList, TArr)) or not self.orderable(t.elem) or s.pat.kind != "pname" or s.mut:
            self.error(s, "`let xs = compress(values)` takes a list or array of ordered values")
        name = s.pat.name
        el = TInt() if is_int(t.elem) else t.elem
        info, _, _ = self.new_domain(f"{name}_D", "0", "", (0, 0), (0, None))
        info.name = f"{name}.D"
        ty = TArr([TDom(info)], el)
        ty.compressed = True
        cpp = self.fresh(name)
        self.decl_global_or_local(cpp, ty)
        self.emit(f"{{ const auto& _a = {c}; {cpp} = {self.cpp_type(ty)}(_a.begin(), _a.end()); }}")
        self.emit(f"sort({cpp}.begin(), {cpp}.end()); {cpp}.erase(unique({cpp}.begin(), {cpp}.end()), {cpp}.end());")
        info.lo_src, info.hi_src = "0", f"(ll){cpp}.size()"
        self.emit(f"{info.cpp}_lo = 0; {info.cpp}_len = (ll){cpp}.size();")
        self.declare(name, ty, cpp, False, s)

    def s_let(self, s):
        v = s.value
        if v is not None and v.kind == "call" and v.fn.kind == "field" and v.fn.name == "freeze" and not v.args \
                and s.pat.kind == "ptuple" and len(s.pat.items) == 2 and all(p.kind == "pname" for p in s.pat.items):
            # `let (D, arr) = nodes.freeze()`: seal a list arena into a domain D and [D]T array
            oc, ot = self.expr(v.fn.obj)
            if not isinstance(ot, TList):
                self.error(s, f"`.freeze()` needs a List, got {ot!r}")
            dname, aname = s.pat.items[0].name, s.pat.items[1].name
            acpp = self.fresh(aname)
            dcpp = self.fresh(dname)
            info = DomInfo(dname, dcpp, "0", f"(ll){acpp}.size()", (0, 0), (0, None))
            info.lo_src, info.hi_src = "0", f"(ll){acpp}.size()"
            at_arr = TArr([TDom(info)], ot.elem)
            self.decl_global_or_local(acpp, ot)
            self.emit(f"{acpp} = std::move({oc});")
            self.scopes[-1][dname] = Sym("type", TDom(info), dcpp, node=s)
            self.scopes[-1][aname] = Sym("var" if s.mut else "let", at_arr, acpp, mut=s.mut, node=s)
            return
        if v is not None and v.kind == "call" and v.fn.kind == "name" and v.fn.name == "compress" and len(v.args) == 1 \
                and s.ty is None and s.pat.kind == "pname" and not self.lookup("compress"):
            return self.s_compress(s)
        if s.pat.kind == "ptuple" and s.ty is not None and s.ty.kind == "ttuple" and self.is_read(v) \
                and len(s.ty.fields) == len(s.pat.items) and not getattr(s, "orelse", None):
            # `let (n, m): (1..=2e5, 1..=n) = read()`: read left to right, so a later bound may name an earlier value
            for k, (pat, (_, ft)) in enumerate(zip(s.pat.items, s.ty.fields)):
                last = k == len(s.pat.items) - 1
                self.s_let(N("let", s.line, pat=pat, ty=ft, value=v, mut=s.mut, const=False,
                             where=getattr(s, "where", None) if last else None, orelse=None, more=[]))
            return
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
            self.emit(f"{{ auto&& _src = {c}; {vals}.assign(_src.begin(), _src.end()); }} sort({vals}.begin(), {vals}.end()); "
                      f"{vals}.erase(unique({vals}.begin(), {vals}.end()), {vals}.end());")
            info, _, _ = self.new_domain(dn, "0", f"{vals}.size()", (0, 0), (0, None))
            info.lo_src, info.hi_src = "0", f"{vals}.size()"
            self.emit(f"{info.cpp}_lo = 0; {info.cpp}_len = (ll){vals}.size();")
            self.scopes[-1][dn] = Sym("type", TDom(info), info.cpp, node=s)
            self.declare(vn, TArr([TDom(info)], t.elem), vals, False, s)
            return
        if v is not None and v.kind == "call" and v.fn.kind == "name" and v.fn.name in ("scc", "components") \
                and not self.lookup(v.fn.name) and s.pat.kind == "ptuple" and len(s.pat.items) == 2 and len(v.args) == 1:
            # let (C, comp) = scc(adj): C is a fresh domain of components, comp: [V]C (stdlib/graph.cp).
            # scc numbers C so that every arc goes to an equal or larger component.
            c, t = self.expr(v.args[0])
            vt, to = self.adjacency(t, v, v.fn.name)
            dn, vn = s.pat.items[0].name, s.pat.items[1].name
            comp = self.fresh(vn)
            info, _, _ = self.new_domain(dn, "0", "", (0, 0), (0, vt.info.len_iv[1]))
            ty = TArr([vt], TDom(info))
            self.decl_global_or_local(comp, ty)
            self.emit(f"{info.cpp}_lo = 0; {info.cpp}_len = cp_{v.fn.name}({c}, {comp}, {to});")
            info.lo_src, info.hi_src = "0", f"{info.cpp}_len"
            self.scopes[-1][dn] = Sym("type", TDom(info), info.cpp, node=s)
            self.declare(vn, ty, comp, False, s)
            return
        if v is not None and getattr(s, "orelse", None):
            # `let (a, b) = (x, y) else:` and `let a = x and let b = y else:` take what `if let` takes:
            # every binding must be there, or the else block runs (and has to leave)
            parts = [(s.pat, v)] + list(getattr(s, "more", None) or [])
            if len(parts) == 1 and s.pat.kind == "ptuple" and v.kind == "tuple" and len(v.items) == len(s.pat.items) \
                    and not any(v.names):
                parts = list(zip(s.pat.items, v.items))
            if len(parts) > 1:
                for pat, value in parts:
                    c, t = self.expr(value)
                    if not isinstance(t, TOpt):
                        self.error(s, f"let-else needs a ?T for `{unparse(value)}`, got {t!r}")
                    o = self.tmpname("o")
                    self.emit(f"auto {o} = {c};")
                    self.emit(f"if (!{o}.has()) {{")
                    self.ind += 1
                    self.block(s.orelse)
                    self.ind -= 1
                    self.emit("}")
                    self.bind_pattern(pat, f"{o}.get()", t.inner, s, mut=s.mut)
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
            if s.value is None:
                if s.ty is not None and s.ty.kind == "ttuple" and len(s.ty.fields) == len(s.pat.items):
                    for pat, (_, ft) in zip(s.pat.items, s.ty.fields):
                        self.s_let(N("let", s.line, pat=pat, ty=ft, value=None, mut=s.mut, const=False, where=None, orelse=None, more=[]))
                    return
                self.error(s, "tuple declaration needs a value or tuple type")
            if s.ty is not None:
                want_ty = self.resolve_type(s.ty)
                if not isinstance(want_ty, TTuple) or len(want_ty.types) != len(s.pat.items):
                    self.error(s, f"expected a {len(s.pat.items)}-tuple type for `{unparse(s.pat)}`, got {want_ty!r}")
                if s.value.kind == "tuple" and not any(s.value.names) and len(s.value.items) == len(s.pat.items) and s.ty.kind == "ttuple":
                    for k, (pat, (_, ft), iv) in enumerate(zip(s.pat.items, s.ty.fields, s.value.items)):
                        last = k == len(s.pat.items) - 1
                        self.s_let(N("let", s.line, pat=pat, ty=ft, value=iv, mut=s.mut, const=False,
                                     where=getattr(s, "where", None) if last else None, orelse=None, more=[]))
                    return
                c, t = self.expr(s.value, want_ty)
                if not isinstance(t, TTuple) or len(t.types) != len(want_ty.types):
                    self.error(s, f"expected {want_ty!r}, got {t!r} (no scalar broadcast on tuple bindings)")
                c = self.coerce(c, t, want_ty, s.value)
                self.bind_pattern(s.pat, c, want_ty, s, mut=s.mut, owned=True)
                return
            c, t = self.expr(s.value)
            self.bind_pattern(s.pat, c, t, s, mut=s.mut, owned=True)
            return
        name = s.pat.name
        v = s.value
        if (v is not None and v.kind == "call" and v.fn.kind == "name" and v.fn.name == "ask"
                and not self.lookup("ask")):
            # interactive: `let x: T = ask("?", i, j, limit: 60)` prints the question, flushes, and reads the judge's reply
            if s.ty is None:
                self.error(s, "ask() needs a declared type: `let x: T = ask(...)`")
            ty = self.resolve_type(s.ty)
            pargs = [a for a in v.args if not (a.kind == "kwarg" and a.name == "limit")]
            self.emit_print(pargs, newline=True, node=s)
            self.emit("out_flush(); fflush(stdout);")
            cpp = self.fresh(name)
            self.emit(f"{self.cpp_type(ty)} {cpp};")
            self.emit_read(cpp, ty, s)
            self.declare(name, ty, cpp, s.mut, s)
            return
        if (s.ty is not None and s.ty.kind == "tarray" and len(s.ty.dims) == 1 and s.ty.dims[0].kind == "tfresh"
                and self.is_read(s.value)):
            return self.read_fresh(s, name)
        ty = self.resolve_type(s.ty) if s.ty is not None else None
        if self.is_read(s.value) and len(self.fn_ret) > getattr(self, "sol_depth", 0) and len(self.fn_ret) > (1 if getattr(self, "in_multitest", False) else 0):
            self.reads_in_fns = True
        if self.is_read(s.value):
            if ty is None:
                self.error(s, "read() needs a declared type: `let x: T = read()`")
            cpp = self.fresh(name)
            self.decl_global_or_local(cpp, ty)
            gen = [a for a in getattr(s.value, "args", []) if a.kind == "kwarg" and a.name == "gen"]
            none_kw = [a for a in getattr(s.value, "args", []) if a.kind == "kwarg" and a.name == "none"]
            saved_rnone = getattr(self, "read_none", None)
            if none_kw and none_kw[0].value.kind == "str":
                self.read_none = none_kw[0].value.value
            try:
                if gen:
                    gc, gt = self.expr(N("call", s.line, fn=gen[0].value, args=[]), ty)
                    self.emit("if (CP_GEN) {")
                    self.ind += 1
                    self.emit(f"{cpp} = {self.coerce(gc, gt, ty, s)};")
                    _emit_input(self, cpp, ty, s)
                    self.ind -= 1
                    self.emit("} else {")
                    self.ind += 1
                    self.emit_read(cpp, ty, s)
                    self.ind -= 1
                    self.emit("}")
                elif s.ty.kind == "trange" and s.ty.range.hi is not None and isinstance(ty, TInt):
                    # `let k: 1..=n = read()`: the bounds are the ones written, evaluated now, not just their static
                    # envelope: the input is checked against them and generated inputs are drawn from them
                    r = s.ty.range
                    (lo, _), (hi, _) = self.expr(r.lo), self.expr(r.hi)
                    if not r.incl:
                        hi = f"({hi}) - 1"
                    self.emit(f"{cpp} = CP_GEN ? gen_int({lo}, {hi}) : rd_range({lo}, {hi}, {s.line});")
                    self.emit(f"if (CP_GEN) {{ out_int({cpp}); out_char(' '); }}")
                elif any(self.is_columns(a) for a in s.value.args):
                    self.emit_read_columns(cpp, ty, s)
                else:
                    self.emit_read(cpp, ty, s)
            finally:
                self.read_none = saved_rnone
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
        fill_assign = None
        if s.value is None:
            if ty is None:
                self.error(s, "var without a value needs a type")
            c, vt = self.default_value(ty, s), ty
        elif isinstance(ty, TArr) and s.value.kind == "list" and len(s.value.items) == 1 and s.value.items[0].kind == "gen":
            c, vt = self.expr(s.value, ty)
            c, vt = self.coerce(c, vt, ty, s.value), ty
        elif (isinstance(ty, TArr) and len(ty.dims) == 1 and s.value.kind == "list" and s.value.items
              and not isinstance(ty.elem, (TList, TMap, TSet, TCounter))):
            # `let a: [D]Int = [2, 3, 5]`: one value per position, in order (never the list in every cell)
            cs = []
            for x in s.value.items:
                xc, xt = self.expr(x, ty.elem)
                cs.append(self.coerce(xc, xt, ty.elem, x))
            n = len(cs)
            c = (f"([&]{{ if ((ll){ty.dims[0].info.len_cpp} != {n}) cp_fail(\"an array literal needs one value per position\", {s.line}); "
                 f"return {self.cpp_type(ty)}{{{', '.join(cs)}}}; }}())")
            vt = ty
        elif isinstance(ty, TArr) and s.value.kind != "table":
            fc, ft = self.expr(s.value, ty.elem)
            if isinstance(ft, TArr):
                c, vt = self.coerce(fc, ft, ty, s.value), ty
            else:
                fc = self.coerce(fc, ft, ty.elem, s.value)
                size = " * ".join(f"(size_t){d.info.len_cpp}" for d in ty.dims)
                c, vt = f"{self.cpp_type(ty)}({size}, {fc})", ty
                fill_assign = (size, fc)
        else:
            c, vt = self.expr(s.value, ty)
            if ty is not None:
                c = self.coerce(c, vt, ty, s.value)
        lent_view = s.value is not None and self.lent_type(s.value, ty)
        if lent_view:
            if s.mut:
                self.no_escape(lent_view, s.value, f"`var {name}`")
            vt = lent_view
        decl_t = ty if ty is not None and not lent_view else vt
        if isinstance(decl_t, TInt) and ty is None and s.mut:
            decl_t = TInt()
        if isinstance(decl_t, TUnit):
            self.error(s, "cannot bind a value of type ()")
        cpp = self.fresh(name)
        if self.at_top and len(self.fn_ret) <= (1 if getattr(self, "in_multitest", False) else 0):
            self.globals.append(f"{self.cpp_type(decl_t)} {cpp};")
            if fill_assign is not None and not getattr(decl_t, "view", None):
                self.emit(f"{cpp}.assign({fill_assign[0]}, {fill_assign[1]});")
            else:
                self.emit(f"{cpp} = {c};")
        else:
            self.emit(f"{'' if s.mut else 'const '}{self.cpp_type(decl_t)} {cpp} = {c};")
        sym = self.declare(name, decl_t, cpp, s.mut, s)
        if getattr(s, "const", False) and s.value is not None:
            sym.kind = "const"
            try:
                sym.const_val = self.eval_const(s.value)
            except CompileError:
                sym.const_val = None
        if lent_view:
            self.begin_hold(s, name, sym, s.value)      # the receiver stays lent until `name` is last used
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

    def emit_read_columns(self, target, ty, node):
        """`let a: [I](x: A, y: B) = read(columns)`: every x, then every y. The input gives one field for all
        the records before the next field, and the program still gets one record per position."""
        if not (isinstance(ty, TArr) and isinstance(ty.elem, TTuple)):
            self.error(node, f"read(columns) fills an array of records one field at a time, got {ty!r}")
        size = " * ".join(f"(size_t){d.info.len_cpp}" for d in ty.dims)
        self.emit(f"{target}.assign({size}, {self.cpp_type(ty.elem)}());")
        for fn, ft in zip(ty.elem.names, ty.elem.types):
            v = self.tmpname("k")
            self.emit(f"for (size_t {v} = 0; {v} < {target}.size(); ++{v}) {{")
            self.ind += 1
            self.emit_read(f"{target}[{v}].{self.field_cpp(fn)}", ft, node, packed=True)
            self.ind -= 1
            self.emit("}")
            self.emit("if (CP_GEN) out_char('\\n');")

    def default_value(self, ty, s):
        if isinstance(ty, TArr):
            size = " * ".join(f"(size_t){d.info.len_cpp}" for d in ty.dims)
            return f"{self.cpp_type(ty)}({size})"
        return f"{self.cpp_type(ty)}()"

    def read_fresh(self, s, name):
        """`let s: [domain S]Char = read()`: a whitespace-delimited word whose positions get the fresh domain S.
        The element may also be a char-labelled domain (`domain Digit = '0'..='9'`): every character is checked
        to be in it, and generated words draw from it."""
        d = s.ty.dims[0]
        et = self.resolve_type(s.ty.elem)
        # the elements are Chars, or a domain of Chars such as `domain Letter = 'a'..='z'`
        letters = et.info if isinstance(et, TDom) and et.info.labels == "char" else None
        if not (isinstance(et, TChar) or letters) or d.range is not None:
            self.error(s, "only `[domain S]Char = read()` (a word; or a domain of Chars as the element) is supported for a fresh domain")
        w = self.tmpname("w")
        lo, span = (letters.lo_cpp, f"{letters.len_cpp} - 1") if letters else ("'a'", "25")
        self.emit(f"string {w};")
        # `read(alphabet: "01")`: the characters a generated word of Chars is drawn from (default a-z)
        alpha = next((a.value for a in getattr(s.value, "args", []) if a.kind == "kwarg" and a.name == "alphabet"), None)
        if alpha is not None and (alpha.kind != "str" or not alpha.value or letters):
            self.error(s, "`alphabet:` takes a non-empty string literal, on a word of Chars")
        table, pick = "", f"(char)({lo} + gen_int(0, {span}))"
        if alpha is not None:
            table, pick = f"static const char _al[] = {json.dumps(alpha.value)}; ", f"_al[gen_int(0, {len(alpha.value) - 1})]"
        self.emit(f"if (CP_GEN) {{ {table}int _n = gen_int(1, 1000000); for (int _i = 0; _i < _n; ++_i) {w} += {pick}; "
                  f"out_str({w}); out_char('\\n'); }} else {w} = rd_word();")
        info, _, _ = self.new_domain(d.name, "0", "", (0, 0), (0, None))
        info.lo_src, info.hi_src = "0", f"(ll){w}.size()"
        self.emit(f"{info.cpp}_lo = 0; {info.cpp}_len = (ll){w}.size();")
        self.scopes[-1][d.name] = Sym("type", TDom(info), info.cpp, node=s)
        ty = TArr([TDom(info)], et)
        cpp = self.fresh(name)
        self.decl_global_or_local(cpp, ty)
        if letters:
            self.emit(f"{cpp} = {self.cpp_type(ty)}({w}.size());")
            self.emit(f"for (size_t _i = 0; _i < {w}.size(); ++_i) "
                      f"{cpp}[_i] = chk_dom((ll){w}[_i] - {letters.lo_cpp}, {letters.len_cpp}, {s.line});")
        else:
            self.emit(f"{cpp} = {self.cpp_type(ty)}({w}.begin(), {w}.end());")
        self.declare(name, ty, cpp, s.mut, s)
        if getattr(s, "where", None) is not None:
            self.emit_where(s.where, s)
        if self.out is self.init_lines:
            self.last_read = len(self.init_lines)
        self.read_pos = (id(self.out), len(self.out))

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

    def x_keepassign(self, e, want=None):
        """`x min= y` / `x max= y` as a Bool: true when the place changed. The update itself is the statement
        form, so `none` is skipped and a missing map key is inserted exactly as there."""
        tgt = e.target
        if self.has_effects(tgt):
            self.error(e, f"`{unparse(tgt)} {e.op} ...` would evaluate `{unparse(tgt)}` twice, and it calls a fn\n"
                          f"  help: bind the position first: `let i = ...`")
        if e.value.kind == "none":      # `none` on the right is skipped: nothing changes
            return "false", TBool()
        stmt = N("assign", e.line, target=tgt, op=e.op, value=e.value)
        changed = self.tmpname("ch")

        def body():
            mt = self.capture(lambda: self.expr(tgt.obj))[1][1] if tgt.kind == "index" else None
            if isinstance(mt, TMap) and not isinstance(mt.val, TOpt):
                oc, _ = self.expr(tgt.obj)
                kc, kt = self.expr(tgt.idx[0], mt.key)
                kc = self.coerce(kc, kt, mt.key, tgt.idx[0])
                key, old = self.tmpname("k"), self.tmpname("old")
                self.emit(f"const auto {key} = {kc};")
                vt_c = self.cpp_type(mt.val)
                self.emit(f"const auto {old}_it = {oc}.find({key});")
                self.emit(f"const bool {old}_had = {old}_it != {oc}.end();")
                self.emit(f"const {vt_c} {old} = {old}_had ? {vt_c}({old}_it->second) : {vt_c}();")
                stmt.target = N("index", tgt.line, obj=tgt.obj, idx=[N("ctemp", tgt.line, code=key, ty=mt.key, src=tgt.idx[0])])
                self.s_assign(stmt)
                self.emit(f"const bool {changed} = !{old}_had || !({vt_c}({oc}.find({key})->second) == {old});")
            else:
                tc, _ = self.expr(tgt)
                old = self.tmpname("old")
                self.emit(f"const auto {old} = {tc};")
                self.s_assign(stmt)
                tc, _ = self.expr(tgt)
                self.emit(f"const bool {changed} = !({tc} == {old});")
        # one expression, so it runs where it is written (`if ok and best min= d:` leaves best alone when not ok)
        lines = self.capture(body)[0]
        return f"([&]() -> bool {{ {NL.join(lines)} return {changed}; }})()", TBool()

    def s_assert(self, s):
        c, t = self.expr(s.cond)
        if not isinstance(t, TBool):
            self.error(s, "assert needs a Bool")
        # `assert` is checked everywhere, judge build included: a wrong answer is worse than a runtime error
        self.emit(f"if (!({c})) cp_fail(\"assertion failed\", {s.line});")
        self.add_facts(s.cond)

    def s_assume(self, s):
        c, t = self.expr(s.cond)
        if not isinstance(t, TBool):
            self.error(s, "assume needs a Bool")
        self.emit(f"if (CP_GEN && !({c})) exit(4);")
        self.emit(f"CP_ASSERT({c}, \"assumption does not hold\", {s.line});")
        self.add_facts(s.cond)

    def s_defer(self, s):
        body = self.capture(lambda: self.stmt(s.stmt))[0]
        dn = self.tmpname("def")
        joined = " ".join(ln.strip() for ln in body)
        self.emit(f"auto {dn} = cp_defer([&]() {{ {joined} }});")

    def x_coalesce_assign(self, e, want=None):
        tgt = e.target
        if tgt.kind == "index":
            oc, ot = self.expr(tgt.obj)
            if isinstance(ot, TMap):
                self.require_mut(tgt.obj, e)
                if len(tgt.idx) != 1:
                    self.error(e, "Map takes one key")
                kc, kt = self.expr(tgt.idx[0], ot.key)
                kc = self.coerce(kc, kt, ot.key, tgt.idx[0])
                vc, vt = self.expr(e.value, ot.val)
                vc = self.coerce(vc, vt, ot.val, e.value)
                vcpp = self.cpp_type(ot.val)
                return (f"([&]() -> {vcpp}& {{ auto _k = {kc}; auto _it = {oc}.find(_k); "
                        f"if (_it == {oc}.end()) _it = {oc}.emplace(_k, {vc}).first; return _it->second; }}())"), ot.val
        tc, tt = self.expr(tgt)
        self.require_mut(tgt, e)
        if isinstance(tt, TOpt):
            vc, vt = self.expr(e.value, tt.inner)
            vc = self.coerce(vc, vt, tt.inner, e.value)
            icpp = self.cpp_type(tt.inner)
            return f"([&]() -> auto& {{ auto& _p = {tc}; if (!_p.has()) _p = {vc}; return _p; }}())", tt
        self.error(e, f"`??=` needs a Map entry or optional place, got {tt!r}")

    def s_coalesce_assign(self, s):
        c, _ = self.x_coalesce_assign(s)
        self.emit(f"(void)({c});")

    def in_gen_body(self):
        ctx = getattr(self, "loop_ctx", [])
        return ctx and ctx[-1][0] == "g"

    def gen_ctx(self):
        for c in reversed(getattr(self, "loop_ctx", [])):
            if c[0] == "g":
                return c
        return None

    def s_break(self, s):
        if getattr(s, "label", None):
            return self.emit(f"goto {self.loop_label(s)}_break;")
        self.emit("return false;" if self.in_gen_body() else "break;")

    def s_continue(self, s):
        if getattr(s, "label", None):
            return self.emit(f"goto {self.loop_label(s)}_continue;")
        self.emit("return true;" if self.in_gen_body() else "continue;")

    def loop_label(self, s):
        """`break 'outer` / `continue 'outer` leave through a C++ label on the loop named `'outer`."""
        for name, cpp, depth in reversed(getattr(self, "labels", [])):
            if name == s.label:
                if depth != len(self.fn_ret) or self.in_gen_body():
                    self.error(s, f"`'{s.label}` names a loop outside this fn or generator body")
                return cpp
        self.error(s, f"no enclosing loop is labeled `'{s.label}`")

    def labeled(self, s, run):
        """Run the loop statement `s`, whose body is compiled by `body()`, under its label (if it has one)."""
        if not getattr(s, "label", None):
            return run(lambda: self.block(s.body))
        cpp = self.tmpname("L_" + s.label)
        self.__dict__.setdefault("labels", []).append((s.label, cpp, len(self.fn_ret)))
        def body():
            self.emit("{")
            self.ind += 1
            self.block(s.body)
            self.ind -= 1
            self.emit("}")
            self.emit(f"{cpp}_continue:;")
        try:
            run(body)
        finally:
            self.labels.pop()
        self.emit(f"{cpp}_break:;")

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
            c, at = self.expr(self.typed(a, pt), pt)
            out.append(self.coerce(c, at, pt, a))
        return out

    def program_return(self):
        """The C++ label a `return` outside every fn jumps to: the end of the program in a single-test file
        and above `@!multitest`, the end of the current test case below it (decision 6.3)."""
        pr = getattr(self, "prog_ret", None)
        if pr is None or len(self.fn_ret) != pr[0]:
            return None
        pr[2] = True
        return pr[1]

    def s_directive(self, s):
        self.error(s, f"`@!{s.name}` stands at the top level of the file, on a line of its own")

    def s_return(self, s):
        if getattr(self, "in_do", 0) and not getattr(s, "synthetic", False):
            self.error(s, "`return` inside `do:` (the block's value is its last expression)")
        label = self.program_return()
        if label is not None:
            if s.value is not None:
                self.error(s, "a top-level `return` takes no value: it ends the program, or the current test case below `@!multitest`")
            g = self.gen_ctx()
            if g is not None:       # inside a `for` over a generator: stop it, and jump once it has returned
                if not hasattr(self, "top_returns"):
                    self.top_returns = set()
                self.top_returns.add(g[1])
                self.emit(f"{{ {g[1]} = true; return false; }}")
            else:
                self.emit(f"goto {label};")
            return
        if not self.fn_ret:
            self.error(s, "return outside a function")
        rt = self.fn_ret[-1]
        g = self.gen_ctx()
        if g is not None:
            if isinstance(rt, TGen):
                self.emit(f"{{ {g[1]} = true; {g[2]} = true; return false; }}")
            elif s.value is None:
                self.emit(f"{{ {g[1]} = true; return false; }}")
            else:
                c, t = self.expr(self.typed(s.value, rt), rt)
                self.emit(f"{{ {g[2]} = {self.coerce(c, t, rt, s.value)}; {g[1]} = true; return false; }}")
            return
        if isinstance(rt, TGen):
            self.emit("return true;")
            return
        if s.value is None:
            self.emit("return;")
            return
        self.escapes[id(s.value)] = "`return`"
        c, t = self.expr(self.typed(s.value, rt), rt)
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
        if getattr(s, "is_const", False):
            # `if const` is decided at compile time, and only the taken branch is type-checked
            cv = self.eval_const(s.cond)
            if not isinstance(cv, bool):
                self.error(s, f"`if const` condition must be Bool, got {type(cv).__name__}")
            branch = s.body if cv else (s.orelse or [])
            if branch:
                self.block(branch, new_scope=False)
            return
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
        saved_opt, saved_ne = dict(self.opt_narrow), dict(self.nonempty_narrow)
        then_facts = self.extract_narrow(s.cond, truthy=True)
        else_facts = self.extract_narrow(s.cond, truthy=False)
        self.scopes.append(dict(self.refine_facts(s.cond)))
        nfacts = len(self.facts)
        self.add_facts(s.cond)
        self.apply_narrow(then_facts)
        self.block(s.body, new_scope=False)
        then_after = (dict(self.opt_narrow), dict(self.nonempty_narrow))
        del self.facts[nfacts:]
        self.scopes.pop()
        self.ind -= 1
        self.opt_narrow, self.nonempty_narrow = dict(saved_opt), dict(saved_ne)
        self.apply_narrow(else_facts)
        if s.orelse:
            self.emit("} else {")
            self.ind += 1
            self.add_facts(s.cond, negate=True)
            self.block(s.orelse)
            del self.facts[nfacts:]
            self.ind -= 1
        else_after = (dict(self.opt_narrow), dict(self.nonempty_narrow))
        self.emit("}")
        # after the `if`: a branch that always leaves hands its complement to the rest of the block;
        # otherwise only what held before and survived both branches is kept (no fact is born at a join)
        then_div, else_div = self.always_diverges(s.body), self.always_diverges(s.orelse)
        if then_div and not else_div:
            self.opt_narrow, self.nonempty_narrow = else_after
        elif else_div and not then_div:
            self.opt_narrow, self.nonempty_narrow = then_after
        else:
            self.opt_narrow = {k: v for k, v in saved_opt.items()
                               if then_after[0].get(k) == v and else_after[0].get(k) == v}
            self.nonempty_narrow = {k: v for k, v in saved_ne.items()
                                    if then_after[1].get(k) is v and else_after[1].get(k) is v}
        if not s.orelse and s.body and s.body[-1].kind in ("return", "break", "continue"):
            self.add_facts(s.cond, negate=True)      # `if i == j: return` leaves `i != j` for the rest of the block
        sym = getattr(self, "after_if_nonzero", None)
        if sym is not None:
            sym.nonzero = True
            self.after_if_nonzero = None

    def s_iflet_chain(self, s):
        """`if let a = x and let b = y [and cond]: ... else: ...`: the body runs when every binding is there."""
        done = self.tmpname("f")
        self.emit(f"{{ bool {done} = false;")
        depth = 0
        self.scopes.append({})
        for pat, value in [(s.pat, s.value)] + s.more:
            c, t = self.expr(value)
            if not isinstance(t, TOpt):
                self.error(s, f"`if let` needs a ?T, got {t!r}")
            o = self.tmpname("o")
            self.emit(f"{{ auto {o} = {c}; if ({o}.has()) {{")
            self.ind += 1
            depth += 1
            self.bind_pattern(pat, f"{o}.get()", t.inner, s)
        if s.extra is not None:
            self.emit(f"if ({self.cond(s.extra)}) {{")
        self.emit(f"{done} = true;")
        self.block(s.body)
        if s.extra is not None:
            self.emit("}")
        for _ in range(depth):
            self.ind -= 1
            self.emit("} }")
        self.scopes.pop()
        if s.orelse:
            self.emit(f"if (!{done}) {{")
            self.ind += 1
            self.block(s.orelse)
            self.ind -= 1
            self.emit("}")
        self.emit("}")

    def s_iflet(self, s):
        if getattr(s, "more", None):
            return self.s_iflet_chain(s)
        c, t = self.expr(s.value)
        if not isinstance(t, TOpt):
            self.error(s, f"`if let` needs a ?T, got {t!r}")
        o = self.tmpname("o")
        self.emit(f"{{ auto {o} = {c};")
        self.emit(f"if ({o}.has()) {{")
        self.ind += 1
        self.scopes.append({})
        self.bind_pattern(s.pat, f"{o}.get()", t.inner, s)
        nfacts = len(self.facts)
        if (s.pat.kind == "pname" and s.value.kind == "binop" and s.value.op in ("+", "-")
                and s.value.b.kind == "num" and s.value.b.value != 0):
            aname = unparse(s.value.a)
            self.facts.append((s.pat.name, "!=", aname, {s.pat.name} | names_in(s.value.a)))
            self.facts.append((aname, "!=", s.pat.name, {s.pat.name} | names_in(s.value.a)))
        if s.extra is not None:
            ec = self.cond(s.extra)
            self.emit(f"if ({ec}) {{")
            self.ind += 1
            self.block(s.body)
            self.ind -= 1
            self.emit("} else goto " + o + "_else;" if s.orelse else "}")
        else:
            self.block(s.body)
        del self.facts[nfacts:]
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
        self.drop_narrow(self.block_writes(s.body) | (self.block_writes(s.orelse) if s.orelse else set()))

    def s_for(self, s):
        stack = self.__dict__.setdefault("trip_stack", [1])
        try:
            t = self.trips_of([("for", s.pat, s.iter)])
        except CompileError:
            t = None
        stack.append(stack[-1] * (t if t is not None else 1))
        self.check_cost(s.line, stack[-1])
        written = self.block_writes(s.body) | {p.name for p in walk(s.pat) if p.kind == "pname"}
        self.drop_facts(written)
        self.drop_narrow(written)
        saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
        try:
            self.labeled(s, lambda body: self.loop(s.pat, s.iter, s, body))
        finally:
            self.opt_narrow, self.nonempty_narrow = saved_n
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
        if getattr(s, "tparams", None):
            # `struct Rooted<V, const C: TreeCfg = DEFAULT>`: compiled once per set of arguments it is used with
            self.scopes[-1][s.name] = Sym("genstruct", None, s.name, node=s)
            return
        self.struct_inst(s, s.name)

    def struct_inst(self, s, inst_name, binds=None):
        saved_scopes = self.scopes
        if binds is not None:
            self.scopes = [self.scopes[0], binds]
        try:
            raw = [(f, self.resolve_type(t)) for f, t in s.fields]
            if getattr(s, "is_const", False):
                for fname, ft in raw:
                    if not self.is_pod_type(ft):
                        self.error(s, f"`const struct {s.name}` holds plain scalars only (Bool, Int, Char, F64 or another "
                                      f"const struct); field `{fname}` is a {ft!r}")
            # a field of type `()` takes no space: it is left out of the emitted struct
            names = [f for f, ft in raw if not isinstance(ft, TUnit)]
            types = [ft for _, ft in raw if not isinstance(ft, TUnit)]
            t = self.tuple_type(names, types, nominal=inst_name if (binds is not None or getattr(s, "is_const", False) or len(names) != len(raw)) else None)
            t.erased_fields = {f for f, ft in raw if isinstance(ft, TUnit)}
            t.all_field_names = [f for f, _ in raw]
            t.is_const_struct = bool(getattr(s, "is_const", False))
            if t.is_const_struct:
                self.const_structs[s.name] = (t, s)
            sym = Sym("type", t, t.cpp, node=s)
            self.scopes[-1][s.name] = sym
            if binds is None:
                saved_scopes[-1][s.name] = sym
            if not s.methods:
                return t
            from .parser import Parser
            self.proj_names |= {m.name for m in s.methods if getattr(m, "lends", None)}
            table = self.__dict__.setdefault("struct_methods", {}).setdefault(t.cpp, {})
            getters = self.struct_getters.setdefault(t.cpp, {})
            for m in s.methods:
                if getattr(m, "lends", None):
                    self.s_proj(s.name, t, m)
                    continue
                mdecos = {d.name: d for d in (m.decos or [])}
                for d in (m.decos or []):
                    # `@law some_when(not self.is_empty)`: this member is there whenever the receiver is not empty (N2)
                    la = d.args[0] if d.name == "law" and d.args else None
                    if la is not None and la.kind == "call" and la.fn.kind == "name" and la.fn.name == "some_when" \
                            and len(la.args) == 1 and unparse(la.args[0]) == "not self.is_empty":
                        self.struct_laws[(t.cpp, m.name)] = "not_empty"
                ps = list(m.params)
                has_self = bool(ps) and ps[0].name == "self"
                if has_self:
                    ps[0] = N("param", m.line, name="self", ty=Parser(s.name).type_(), mode=ps[0].mode)
                    ps[0].ty_fixed = t
                fn = N("fn", m.line, name=f"{inst_name}__{m.name}", tparams=m.tparams, params=ps, ret=m.ret,
                       body=m.body, decos=[d for d in (m.decos or []) if d.name not in ("get", "law")], pure=m.pure)
                if has_self:
                    fn.self_struct = t
                table[m.name] = (fn.name, has_self, has_self and ps[0].mode == "inout")
                if "get" in mdecos and has_self and len(ps) == 1:
                    getters[m.name] = fn.name
                self.method_nodes.setdefault(t.cpp, {}).setdefault(m.name, []).append(fn)
                if binds is None:
                    self.s_fn(fn)
                else:
                    saved = (self.at_top, self.fn_ret, getattr(self, "loop_ctx", []), getattr(self, "in_multitest", False))
                    self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = True, [], [], False
                    try:
                        self.capture(lambda: self.s_fn(fn, hoist=True))
                    finally:
                        self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = saved
                    self.scopes[0][fn.name] = binds[fn.name]
            return t
        finally:
            self.scopes = saved_scopes

    # ---------- plan/19: projections, lent views, user Index types
    def lent_type(self, value, ty):
        """The view type if `value` is a lent slice or row, else None (re-evaluates nothing)."""
        t = self.last_type.get(id(value))
        return t if getattr(t, "view", None) else None

    def find_proj(self, name):
        for tbl in self.projs.values():
            if name in tbl:
                return tbl[name]
        return None

    def require_mut(self, tgt, node):
        """A write through `tgt` needs a var at the root and `lends mut` at every projection on the way."""
        root = tgt
        while root.kind in ("index", "field", "paren", "unwrap", "coalesce_assign") or self.is_proj_call(root):
            if root.kind == "call":
                p = self.find_proj(root.fn.name)
                if p is not None and not p.mut:
                    self.error(node, f"`{show(root)}` is lent read-only: `{p.owner}.{p.name}` is declared `lends {p.ret!r}`; "
                                     f"declare it `lends mut {p.ret!r}` to allow writes through it")
                root = root.fn.obj
            elif root.kind == "coalesce_assign":
                root = root.target
            else:
                root = root.a if root.kind in ("paren", "unwrap") else root.obj
        if root.kind == "name":
            sym = self.lookup(root.name)
            if sym is None or not (sym.mut or getattr(sym, "lent_mut", False)):
                lent = getattr(sym, "lent", None)
                if lent is not None:
                    self.error(node, f"`{root.name}` was lent from a value that is not a var, so it is read-only")
                self.error(node, f"`{root.name}` is not a var")

    def builtin_swap(self, e):
        """swap(mut x, mut y): two places lent mutably at once, so they must be provably disjoint."""
        a, b = e.args
        if not (getattr(a, "mut", False) and getattr(b, "mut", False)):
            self.error(e, "swap changes both arguments; write `swap(mut x, mut y)`")
        cs = []
        for x in (a, b):
            self.require_mut(x, x) if x.kind != "name" else None
            if x.kind == "name" and not getattr(self.lookup(x.name), "mut", False):
                self.error(x, f"`{x.name}` is not a var")
            self.note_write(x, e.line)
            self.lvalue = (x, "=")
            try:
                cs.append(self.expr(x))
            finally:
                self.lvalue = None
        (ac, at), (bc, bt) = cs
        if at != bt:
            self.error(e, f"swap of {at!r} and {bt!r}")
        self.check_args(e, [(a, True), (b, True)])
        if getattr(at, "view", None) or getattr(bt, "view", None):
            if not (getattr(at, "view", None) == "row" and getattr(bt, "view", None) == "row"):
                self.error(e, "swap of two lent parts needs both to have one fixed length; the prototype swaps rows only")
            return f"cp_swap_view({ac}, {bc})", UNIT
        return f"swap({ac}, {bc})", UNIT

    def view_spec(self, sym, ats, e):
        g = getattr(sym, "node", None)
        if sym is None or g is None or g.kind != "fn" or getattr(sym, "local", False):
            self.error(e, "a lent slice or row can only be passed to a top-level fn (prototype)")
        key = (sym.cpp,) + tuple(getattr(t, "view", None) or "-" for t in ats)
        if key not in self.view_specs:
            clone = copy.copy(g)
            clone.name = f"{g.name}__view{len(self.view_specs)}"
            clone.decos = []
            clone.params = []
            for p, t in zip(g.params, ats):
                q = copy.copy(p)
                if getattr(t, "view", None):
                    q.ty_fixed = t
                clone.params.append(q)
            holder = {}
            saved = (self.scopes, self.at_top, self.fn_ret, getattr(self, "loop_ctx", []), getattr(self, "in_multitest", False))
            self.scopes, self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = [self.scopes[0], holder], True, [], [], False
            self.view_specs[key] = None
            try:
                self.capture(lambda: self.s_fn(clone, hoist=True))
            finally:
                self.scopes, self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = saved
            self.view_specs[key] = holder[clone.name]
        if self.view_specs[key] is None:
            self.error(e, f"`{g.name}` passes a lent part to itself recursively (prototype)")
        return self.view_specs[key]

    def s_proj(self, sname, st, m):
        """`fn at(self, i: I) lends mut T: ... lend mut place`.

        Checked as a fn with one inline callback called at the `lend`; emitted as a C++ function that
        returns the lent place by reference (or a view by value), which is what inlining the callback gives."""
        ps = list(m.params)
        if not ps or ps[0].name != "self":
            self.error(m, f"a projection lends part of its receiver, so `{m.name}` needs `self` as its first parameter")
        at = [k for k, x in enumerate(m.body) if x.kind == "lend"]
        if len(at) != 1:
            self.error(m, f"`{m.name}` is declared `lends`, so every path through it must reach one `lend place`; "
                          f"the prototype wants it as a statement of the body itself, not inside an `if` or a loop")
        # statements after the `lend` run when the caller is done with the place: the write-back (plan/19 §3.4)
        pre, lend, post = m.body[:at[0]], m.body[at[0]], m.body[at[0] + 1:]
        laws = [x for x in pre if x.kind == "law"]
        pre = [x for x in pre if x.kind != "law"]
        for x in walk(pre + post):
            if x.kind in ("lend", "return", "law"):
                self.error(x, f"`{m.name}` must reach exactly one `lend`, and can't `return` (prototype)")
        law = laws[0] if laws else None
        if len(laws) > 1 or (law is not None and law.name != "disjoint"):
            self.error(laws[-1], "a projection states one law, `law disjoint(...)`")
        if law is not None and len(law.params) != 2 * (len(ps) - 1):
            self.error(law, f"`law disjoint` relates two lends of `{m.name}`, so it takes {2 * (len(ps) - 1)} "
                            f"parameters: the arguments of the first lend, then of the second")
        if m.lends == "read" and lend.mut:
            self.error(lend, f"`{m.name}` is declared `lends` without `mut`, so it can't `lend mut`; declare it `lends mut`")
        if m.lends == "mut" and not lend.mut:
            self.error(lend, f"`{m.name}` is declared `lends mut`, so say `lend mut {show(lend.value)}`")
        fields = set(st.names)
        if root_name(lend.value) not in fields | {"self"}:
            self.error(lend, f"a projection may only lend part of `self`; `{show(lend.value)}` is not reached through a field")
        mut_self = ps[0].mode == "inout"
        declared = self.resolve_type(m.ret)
        cpp = self.fresh(f"{sname}__{m.name}")
        self.scopes.append({})
        saved_top, self.at_top = self.at_top, False
        saved_lend = (self.facts, self.holds, self.cur_rest)
        self.facts, self.holds, self.cur_rest = [], [], []
        saved_ctx, self.loop_ctx = getattr(self, "loop_ctx", []), []
        self.fn_ret.append(UNIT)
        params, ptypes, pnames = [], [], []
        try:
            self.declare("self", st, "self_", mut_self, None)
            for fname, ftype in zip(st.names, st.types):
                self.declare(fname, ftype, f"self_.{self.field_cpp(fname)}", mut_self, None)
            for p in ps[1:]:
                if p.ty is None:
                    self.error(p, f"parameter `{p.name}` needs a type")
                pt = self.resolve_type(p.ty)
                pc = self.fresh(p.name)
                params.append(f"{self.cpp_type(pt)} {'' if isinstance(pt, SCALARS) else 'const &'}{pc}")
                self.declare(p.name, pt, pc, False, p)
                ptypes.append(pt)
                pnames.append(p.name)
            def body():
                self.block(pre, new_scope=False)
                self.lvalue = (lend.value, "=")
                try:
                    return self.expr(lend.value)
                finally:
                    self.lvalue = None
            lines, (c, t) = self.capture(body)
            # the prototype emits the write-back as a second function, so it sees `self` and the
            # parameters but not locals declared before the `lend`
            if post and not mut_self:
                self.error(post[0], f"code after the `lend` changes `self`, so `{m.name}` takes `mut self`")
            after = self.capture(lambda: self.block(post, new_scope=True))[0] if post else None
        finally:
            self.fn_ret.pop()
            self.loop_ctx = saved_ctx
            self.facts, self.holds, self.cur_rest = saved_lend
            self.at_top = saved_top
            self.scopes.pop()
        if t != declared:
            self.error(lend, f"`{m.name}` is declared `lends {declared!r}` but lends a {t!r}")
        sig = "".join(", " + x for x in params)
        self.funcs.append(f"template <class S_> decltype(auto) {cpp}(S_& self_{sig}) {{\n" + "\n".join(lines) +
                          f"\n    return ({c});\n}}")
        proj = Proj(m.name, cpp, pnames, ptypes, t, m.lends == "mut", law, mut_self, sname)
        proj.after = None
        if after is not None:
            proj.after = cpp + "_after"
            self.funcs.append(f"template <class S_> void {proj.after}(S_& self_{sig}) {{\n" + "\n".join(after) + "\n}")
        self.projs.setdefault(st.cpp, {})[m.name] = proj

    def call_proj(self, p, e, oc, ot):
        if isinstance(ot, TType):
            self.error(e, f"`{p.name}` lends part of a value; call it on a {p.owner}, not on the type")
        if len(e.args) != len(p.ptypes):
            self.error(e, f"`{p.owner}.{p.name}` takes {len(p.ptypes)} arguments, got {len(e.args)}")
        lv, self.lvalue = getattr(self, "lvalue", None), None
        args = []
        for a, pt in zip(e.args, p.ptypes):       # index arguments are evaluated before the lend begins
            c, at = self.expr(self.typed(a, pt), pt)
            args.append(self.coerce(c, at, pt, a))
        self.lvalue = lv
        if p.mut_self:
            self.require_mut(e.fn.obj, e) if e.fn.obj.kind != "name" else None
            if e.fn.obj.kind == "name" and not getattr(self.lookup(e.fn.obj.name), "mut", False):
                self.error(e, f"`{p.owner}.{p.name}` takes `mut self`, and `{e.fn.obj.name}` is not a var")
            self.note_write(e.fn.obj, e.line)
        if p.after:
            # the write-back runs when the statement that used the place is over
            if getattr(self, "cur_stmt", None) not in ("assign", "exprstmt", "let"):
                self.error(e, f"`{p.owner}.{p.name}` runs code after its lend; the prototype allows it in an "
                              f"assignment, a `let` or a call statement, not in a condition or a loop header")
            held = []
            for c, pt in zip(args, p.ptypes):
                t_ = self.tmpname("a")
                self.emit(f"const {self.cpp_type(pt)} {t_} = {c};")
                held.append(t_)
            args = held
            # an expression may be compiled more than once (to find its type); the mark says which copy was kept
            mark = f"// lent by {p.owner}.{p.name} until this statement ends ({self.tmpname('w')})"
            self.emit(mark)
            self.after_stmt.append((mark, f"{p.after}({', '.join([oc] + args)});"))
        code = f"{p.cpp}({', '.join([oc] + args)})"
        if self.narrow(p.ret) and not (lv is not None and lv[0] is e):
            code = f"(ll){code}"
        return code, p.ret

    def to_idx(self, info, oc, e):
        f = self.lookup(info.to_idx)
        return f"{f.cpp}({oc})", f.ty.ret

    def s_lend(self, s):
        self.error(s, "`lend` is the last statement of a projection (`fn name(self, ...) lends T`), and only there")

    def s_law(self, s):
        self.error(s, "`law disjoint(...)` belongs at the top of a projection's body")

    def s_impl(self, s):
        """`impl Index for Cell:` makes a struct a bounded index: `[Cell]T`, `for c in Cell`, `Cell.K`."""
        sym = self.lookup(s.target)
        if s.trait != "Index":
            if not s.body:
                return          # `impl Seq for V`: a marker; domains already have the Seq methods
            self.error(s, f"the prototype implements `impl Index for <struct>` only, not `{s.trait}`")
        if sym is None or sym.kind != "type" or not isinstance(sym.ty, TTuple):
            self.error(s, f"`impl Index for {s.target}` needs a struct; a `domain` is already an Index")
        st = sym.ty
        ks = [x for x in s.body if x.kind == "let" and x.pat.kind == "pname" and x.pat.name == "K"]
        fns = {x.name: x for x in s.body if x.kind == "fn"}
        if len(ks) != 1 or set(fns) != {"to_idx", "from_idx"} or len(s.body) != 3:
            self.error(s, f"`impl Index for {s.target}` is exactly `let K = <size>`, `fn to_idx(self) -> Int` and "
                          f"`fn from_idx(i: Int) -> {s.target}`")
        k = ks[0]
        if k.mut:
            self.error(k, "the size of an index type is fixed where it is declared; write `let K = ...`, not `var K`")
        if k.const:
            try:
                self.capture(lambda: self.const_value(k.value))
            except CompileError:
                raise CompileError(
                    f"line {k.line}: `const K = {show(k.value)}`: `const` is compile-time only, and `{show(k.value)}` is known "
                    f"only at run time\n  help: write `let K = {show(k.value)}`; an associated `let` is evaluated once, where "
                    f"the impl is declared, and is frozen from then on")
        for nm in sorted(names_in(k.value)):
            v = self.lookup(nm)
            if v is not None and v.kind == "var":
                self.error(k, f"`K` can't be sized by the var `{nm}`: it could change after arrays are made\n"
                              f"  help: freeze it first: `let n = {nm}`")
        kc, kt = self.expr(k.value)
        if not is_int(kt):
            self.error(k, f"`K` is a size, got {kt!r}")
        kcpp = self.fresh(f"K_{s.target}")
        self.globals.append(f"ll {kcpp};")
        self.emit(f"{kcpp} = {kc};")
        ksym = Sym("let", TInt(kt.lo, kt.hi), kcpp, node=None)
        from .parser import Parser
        table = self.__dict__.setdefault("struct_methods", {}).setdefault(st.cpp, {})
        had = self.scopes[-1].get("K")
        self.scopes[-1]["K"] = ksym
        try:
            for name in ("to_idx", "from_idx"):
                m = fns[name]
                ps = list(m.params)
                has_self = bool(ps) and ps[0].name == "self"
                if has_self != (name == "to_idx") or len(ps) != 1 or m.ret is None:
                    self.error(m, f"expected `fn to_idx(self) -> Int` and `fn from_idx(i: Int) -> {s.target}`")
                if has_self:
                    ps[0] = N("param", m.line, name="self", ty=Parser(s.target).type_(), mode=ps[0].mode)
                fn = N("fn", m.line, name=f"{s.target}__{name}", tparams=m.tparams, params=ps, ret=m.ret,
                       body=m.body, decos=m.decos, pure=m.pure, lends=None)
                if has_self:
                    fn.self_struct = st
                table[name] = (fn.name, has_self, False)
                self.s_fn(fn)
        finally:
            if had is None:
                self.scopes[-1].pop("K", None)
            else:
                self.scopes[-1]["K"] = had
        if not is_int(self.lookup(f"{s.target}__to_idx").ty.ret) or self.lookup(f"{s.target}__from_idx").ty.ret != st:
            self.error(s, f"expected `fn to_idx(self) -> Int` and `fn from_idx(i: Int) -> {s.target}`")
        info = DomInfo(s.target, f"I_{s.target}", "0", kcpp, (0, 0), (kt.lo, kt.hi), anon=False)
        info.user, info.to_idx, info.from_idx = st, f"{s.target}__to_idx", f"{s.target}__from_idx"
        self.index_impls[st.cpp] = info

    def lambda_key(self, lam, et):
        if lam.kind == "name" and getattr(self.lookup(lam.name), "kind", None) in ("fn", "genfn"):
            # a fn passed by name: `first_where(ok)` is `first_where(x => ok(x))`
            lam = N("lambda", lam.line, params=[N("pname", lam.line, name="_it")],
                    body=N("call", lam.line, fn=lam, args=[N("name", lam.line, name="_it")]))
        if lam.kind != "lambda" or len(lam.params) != 1:
            self.error(lam, "expected a one-parameter lambda `x => ...`")
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
                if len(d.args) != len(s.variants):
                    self.error(s, "@tags needs one tag per variant")
                tags = []
                for a in d.args:
                    if a.kind == "str":
                        tags.append(a.value)
                    elif a.kind == "char":
                        tags.append(("char", a.value))
                    elif a.kind == "range" and a.lo and a.hi and a.lo.kind == "char" and a.hi.kind == "char":
                        tags.append(("charrange", a.lo.value, a.hi.value, a.incl))
                    else:
                        self.error(s, "@tags entries must be strings, chars, or char ranges")
                et.tags = tags
        self.scopes[-1][s.name] = Sym("type", et, cpp, node=s)
        fields = []
        for k, (vn, fl) in enumerate(s.variants):
            fns = [f for f, _ in fl]
            fts = [self.resolve_type(t) for _, t in fl]
            et.variants.append((vn, fns, fts))
            for f, t in zip(fns, fts):
                fields.append(f"{self.cpp_type(t)} {vn}_{f}{{}};")
            self.scopes[-1][vn] = Sym("ctor", et, k, node=s)
        # Compute finite Index cardinality & to_idx() if every variant field is finite (unit enum, bounded Int, Bool, Domain)
        def field_card(ft):
            if isinstance(ft, TDom):
                return ft.info.len_cpp, ft.info.len_iv, lambda expr: f"(int)({expr})"
            if isinstance(ft, TBool):
                return "2", (2, 2), lambda expr: f"(int)({expr})"
            if isinstance(ft, TInt) and ft.lo is not None and ft.hi is not None:
                sz = ft.hi - ft.lo + 1
                return str(sz), (sz, sz), (lambda lo: lambda expr: f"(int)(({expr}) - ({lo}LL))")(ft.lo)
            return None
        off_parts, lo_tot, hi_tot, idx_cases, ok_idx = [], 0, 0, [], True
        for k, (vn, fns, fts) in enumerate(et.variants):
            base_expr = " + ".join(off_parts) if off_parts else "0"
            if not fns:
                idx_cases.append(f"if (_e.tag == {k}) return (int)({base_expr});")
                off_parts.append("1")
                lo_tot = None if lo_tot is None else lo_tot + 1
                hi_tot = None if hi_tot is None else hi_tot + 1
            else:
                cs = [field_card(ft) for ft in fts]
                if any(c is None for c in cs):
                    ok_idx = False
                    break
                cur_expr = None
                sz_exprs = []
                vlo, vhi = 1, 1
                for fnm, (sc, siv, to_i) in zip(fns, cs):
                    fi = to_i(f"_e.{vn}_{fnm}")
                    cur_expr = fi if cur_expr is None else f"(({cur_expr}) * (int)({sc}) + ({fi}))"
                    sz_exprs.append(f"(int)({sc})")
                    vlo = None if (vlo is None or siv[0] is None) else vlo * siv[0]
                    vhi = None if (vhi is None or siv[1] is None) else vhi * siv[1]
                idx_cases.append(f"if (_e.tag == {k}) return (int)(({base_expr}) + ({cur_expr}));")
                off_parts.append(" * ".join(sz_exprs))
                lo_tot = None if (lo_tot is None or vlo is None) else lo_tot + vlo
                hi_tot = None if (hi_tot is None or vhi is None) else hi_tot + vhi
        if ok_idx and off_parts:
            len_cpp = " + ".join(off_parts)
            info = DomInfo(s.name, f"enum_{cpp}", "0", len_cpp, (0, 0), (lo_tot, hi_tot), anon=False)
            info.enum_type = et
            et.index_info = info
            self.funcs.append(f"static inline int cp_eidx_{cpp}(const {cpp}& _e) {{ {' '.join(idx_cases)} return 0; }}")
        trivial = all(isinstance(t, (TInt, TDom, TBool, TChar)) for _, _, fts in et.variants for t in fts)
        if trivial and sum(1 for _, fns, _ in et.variants if fns) >= 2:
            # variants share storage; anonymous structs in a union are a GNU extension every judge's g++ accepts
            arms = " ".join(f"struct {{ {' '.join(f'{self.cpp_type(t)} {vn}_{f};' for f, t in zip(fns, fts))} }};"
                            for vn, fns, fts in et.variants if fns)
            cases = " : ".join(f"tag == {k} ? ({' && '.join(f'{vn}_{f} == o.{vn}_{f}' for f in fns)})"
                               for k, (vn, fns, _) in enumerate(et.variants) if fns)
            self.struct_defs.append(f"struct {cpp} {{ int tag = 0; union {{ {arms} }};\n"
                                    f"    {cpp}() {{}}\n"
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

    def x_matchexpr(self, e, want):
        sc, st = self.expr(e.subject)
        if not isinstance(st, TEnum):
            self.error(e, f"match needs an enum value, got {st!r}")
        m = self.tmpname("m")
        rt = want
        arm_code = []
        for arm in e.arms:
            self.scopes.append({})
            lines = []
            if arm.name == "_":
                vc, vt = self.expr(arm.value, rt)
                rt = rt or (TInt() if is_int(vt) else vt)
                lines.append(f"return {self.coerce(vc, vt, rt, arm.value)};")
            else:
                k, v = st.variant(arm.name)
                if k is None:
                    self.error(arm, f"{st.name} has no variant {arm.name}")
                vn, fns, fts = v
                binds_c = []
                for b, fnm, ft in zip(arm.binds, fns, fts):
                    cpp = self.fresh(b)
                    self.scopes[-1][b] = Sym("let", ft, cpp, node=arm)
                    binds_c.append(f"const auto& {cpp} = {m}.{vn}_{fnm};")
                vc, vt = self.expr(arm.value, rt)
                rt = rt or (TInt() if is_int(vt) else vt)
                g_cond = f"if ({self.expr(arm.guard)[0]}) " if arm.guard is not None else ""
                lines.append(f"if ({m}.tag == {k}) {{ {' '.join(binds_c)} {g_cond}return {self.coerce(vc, vt, rt, arm.value)}; }}")
            self.scopes.pop()
            arm_code.extend(lines)
        cty = self.cpp_type(rt)
        return f"([&](const auto {m}) -> {cty} {{ {' '.join(arm_code)} cp_fail(\"non-exhaustive match\", {e.line}); return {cty}(); }}({sc}))", rt

    def s_match(self, s):
        c, t = self.expr(s.subject)
        if not isinstance(t, TEnum):
            self.error(s, f"match needs an enum value, got {t!r}")
        m = self.tmpname("m")
        done = f"{m}_done"
        # the subject is owned, as a `let` destructuring is: an arm may grow the list it was read from
        self.emit(f"{{ const auto {m} = {c};")
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
        for arm in s.arms:
            self.drop_narrow(self.block_writes(arm.body))

    def s_whilelet(self, s):
        written = self.block_writes(s.body)
        self.drop_narrow(written)
        saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
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
        if getattr(s, "extra", None) is not None:
            self.emit(f"if (!({self.cond(s.extra)})) break;")
        self.block(s.body)
        self.scopes.pop()
        self.ind -= 1
        self.loop_trips.pop()
        self.emit("}")
        self.opt_narrow, self.nonempty_narrow = saved_n

    def s_while(self, s):
        written = self.block_writes(s.body)
        self.drop_facts(written)   # a fact about a var the loop assigns holds only on the first trip
        self.drop_narrow(written)
        saved_n = (dict(self.opt_narrow), dict(self.nonempty_narrow))
        c = self.cond(s.cond)
        self.emit(f"while ({c}) {{")
        self.loop_trips.append(None)
        self.ind += 1
        if not hasattr(self, "loop_ctx"):
            self.loop_ctx = []
        self.loop_ctx.append(("c",))
        nfacts = len(self.facts)
        self.add_facts(s.cond)
        self.apply_narrow(self.extract_narrow(s.cond, truthy=True))
        cpp = None
        if getattr(s, "label", None):
            cpp = self.tmpname("L_" + s.label)
            self.__dict__.setdefault("labels", []).append((s.label, cpp, len(self.fn_ret)))
            self.emit("{")
        self.block(s.body)
        if cpp:
            self.emit("}")
            self.emit(f"{cpp}_continue:;")
            self.labels.pop()
        del self.facts[nfacts:]
        self.loop_ctx.pop()
        self.ind -= 1
        self.loop_trips.pop()
        self.emit("}")
        if cpp:
            self.emit(f"{cpp}_break:;")
        self.opt_narrow, self.nonempty_narrow = saved_n

    def s_assign(self, s):
        try:
            return self.s_assign_inner(s)
        finally:
            for t in (s.target.items if s.target.kind == "tuple" else [s.target]):
                self.invalidate_narrow(t)

    def s_assign_inner(self, s):
        tgt, op = s.target, s.op
        fold = getattr(s, "fold", None)         # `x f= y`, parsed as `x = f(x, y)`
        if tgt.kind == "index" and (fold or op in ("min=", "max=")):
            mt = self.capture(lambda: self.expr(tgt.obj))[1][1]
            if isinstance(mt, TMap) and not isinstance(mt.val, TOpt):
                if not fold:
                    return self.s_map_keep(s, mt)
                if not self.lookup(fold):
                    return self.s_map_fold(s, mt)
                self.no_identity(tgt, fold)
        if tgt.kind == "tuple":
            return self.s_parallel_assign(s)
        if tgt.kind == "unwrap" and not fold:
            # `x! += d` / `x! = v`: `x!` is a place. The value must be there; the write goes to x itself.
            self.require_mut(tgt, s)
            if self.has_effects(tgt.a):
                self.error(s, f"`{unparse(tgt)} {op} ...` would evaluate `{unparse(tgt.a)}` twice, and it calls a fn\n"
                              f"  help: bind the position first: `let i = ...`")
            cur = N("unwrap", tgt.line, a=tgt.a)
            cc, ct = self.expr(cur)
            if op == "=":
                self.emit(f"(void)({cc});")
                value = s.value
            elif op in ("min=", "max="):
                value = N("call", s.line, fn=N("name", s.line, name=op[:3]), args=[cur, s.value])
            else:
                value = N("binop", s.line, op=op[:-1], a=cur, b=s.value)
            return self.s_assign(N("assign", s.line, target=tgt.a, op="=", value=value))
        if op in ("min=", "max=") and self.has_effects(tgt):
            self.error(s, f"`{unparse(tgt)} {op} ...` would evaluate `{unparse(tgt)}` twice, and it calls a fn\n"
                          f"  help: bind the position first: `let i = ...`")
        # Indexed assignment `a[idx].f = rhs` (v2.1 U4): evaluate effectful RHS first (matching RHS-before-LHS
        # assignment semantics), then effectful indices left-to-right, and only then capture the mutable slot
        # in `a`, so growing/reallocating `a` during `rhs` or `idx` never dangles or reads stale size.
        rhs_node = s.value
        tgt_eval = tgt
        if op == "=" and self.io_call(rhs_node) is not None:
            tt0 = self.capture(lambda: self.expr(tgt))[1][1]
            rc, rt = self.expr(self.typed(rhs_node, tt0))
            rhs_node = N("ctemp", rhs_node.line, code=rc, ty=rt, src=rhs_node)
        has_idx = tgt.kind in ("index", "field") and any(x.kind == "index" for x in walk([tgt]))
        if has_idx and self.has_effects(rhs_node):
            tt0 = self.capture(lambda: self.expr(tgt))[1][1]
            want0 = tt0.inner if isinstance(tt0, TOpt) and (op == "=" or op in ("min=", "max=")) and rhs_node.kind != "none" else tt0
            self.escapes[id(rhs_node)] = "this assignment"
            rvc, rvt = self.expr(rhs_node, want0 if not isinstance(tt0, TInt) else None)
            self.check_assign_alias(s)
            rvtmp = self.tmpname("rv")
            self.emit(f"auto {rvtmp} = {rvc};")
            rhs_node = N("ctemp", rhs_node.line, code=rvtmp, ty=rvt, src=rhs_node)
        if has_idx:
            def _hoist_idx(n):
                if n.kind == "field":
                    return N("field", n.line, obj=_hoist_idx(n.obj), name=n.name)
                if n.kind == "index":
                    hoisted_obj = _hoist_idx(n.obj)
                    new_idx = []
                    for ix in n.idx:
                        if self.has_effects(ix):
                            ic, it = self.expr(ix)
                            tmp = self.tmpname("ix")
                            self.emit(f"auto {tmp} = {ic};")
                            new_idx.append(N("ctemp", ix.line, code=tmp, ty=it, src=ix))
                        else:
                            new_idx.append(ix)
                    return N("index", n.line, obj=hoisted_obj, idx=new_idx)
                return n
            tgt_eval = _hoist_idx(tgt)
        self.lvalue = (tgt_eval, op)
        tc, tt = self.expr(tgt_eval)
        self.lvalue = None
        self.note_write(tgt, s.line)
        if tgt.kind == "name":
            sym = self.lookup(tgt.name)
            if sym is None or not sym.mut:
                self.error(s, f"`{tgt.name}` is not a var")
        else:
            self.require_mut(tgt, s)
            sym = None
        if op != "=" and any(self.is_proj_call(x) for x in walk([tgt])):
            # `t.at(v) += 1` reads and writes one lent place: the projection runs once
            place = self.tmpname("p")
            self.emit(f"auto&& {place} = {tc};")
            tc = place
        if op in ("min=", "max="):
            # `none` is "no value yet" on either side and is skipped: `best min= d` keeps best when d is none,
            # and takes d when best is none (v2 O2). The value itself must have an order: a Mod<P> has none.
            base = tt.inner if isinstance(tt, TOpt) else tt
            if not self.orderable(base):
                self.error(s, f"`{op}` needs an ordered value, and {base!r} has no `<`")
            vt0 = self.capture(lambda: self.expr(rhs_node, base))[1][1]
            if isinstance(vt0, TOpt) or rhs_node.kind == "none":
                if rhs_node.kind == "none":
                    return
                vc, vt = self.expr(rhs_node)
                if rhs_node is s.value:
                    self.check_assign_alias(s)
                o = self.tmpname("o")
                x = self.coerce(f"{o}.get()", vt.inner, base, rhs_node)
                if isinstance(tt, TOpt):
                    self.emit(f"{{ const auto {o} = {vc}; if ({o}.has()) opt_{op[:3]}({tc}, {x}); }}")
                else:
                    cmp = "<" if op == "min=" else ">"
                    self.emit(f"{{ const auto {o} = {vc}; if ({o}.has() && {x} {cmp} {tc}) {tc} = {x}; }}")
                    if isinstance(tt, TInt):
                        self.track(sym, tgt, TInt(), "=", s)
                return
        if op in ("min=", "max=") and isinstance(tt, TOpt):
            vc, vt = self.expr(rhs_node, tt.inner)
            if rhs_node is s.value:
                self.check_assign_alias(s)
            self.emit(f"opt_{op[:3]}({tc}, {self.coerce(vc, vt, tt.inner, rhs_node)});")
            return
        want = tt.inner if isinstance(tt, TOpt) and op == "=" and rhs_node.kind != "none" else tt
        self.escapes[id(rhs_node)] = "this assignment"
        vc, vt = self.expr(rhs_node, want if not isinstance(tt, TInt) else None)
        if rhs_node is s.value:
            self.check_assign_alias(s)
        if op == "=":
            c = self.coerce(vc, vt, tt, rhs_node)
            self.emit(f"{tc} = {c};")
            self.track(sym, tgt, vt, "=", s)
            return
        if op in ("min=", "max="):
            if isinstance(tt, TInt) and isinstance(vt, TInt):
                self.emit(f"{tc} = {op[:3]}<ll>({tc}, {vc});")
                self.track(sym, tgt, vt, "=", s)
                return
            self.emit(f"{tc} = {op[:3]}({tc}, {self.coerce(vc, vt, tt, rhs_node)});")
            return
        bop = op[:-1]
        if isinstance(tt, TOpt) and op in ("+=", "-=") and isinstance(tt.inner, (TInt, TF64)):
            bi = TInt() if is_int(tt.inner) else tt.inner
            octy = self.cpp_type(tt)
            if isinstance(vt, TOpt):
                self.emit(f"{{ auto _b = {vc}; if (!_b.has()) {tc} = {octy}(); else if ({tc}.has()) {tc} = {octy}({tc}.get() {bop} _b.get()); }}")
            else:
                self.emit(f"{{ auto _b = {self.coerce(vc, vt, bi, rhs_node)}; if ({tc}.has()) {tc} = {octy}({tc}.get() {bop} _b); }}")
            return
        if isinstance(tt, TStr) and op == "+=" and isinstance(vt, (TStr, TChar)):
            self.emit(f"{tc} += {vc};")
            return
        if isinstance(tt, TMod):
            self.emit(f"{tc} {op} {self.coerce(vc, vt, tt, rhs_node)};")
            return
        if isinstance(tt, TDom):
            self.error(s, f"`{op}` on a domain value; use `i = i + k` (checked)")
        if isinstance(tt, (TF64, TU64, TI128)):
            c, _ = self.x_binop(N("binop", s.line, op=bop, a=N("ctemp", s.line, code=tc, ty=tt, src=tgt), b=rhs_node), tt)
            self.emit(f"{tc} = {c};")
            return
        if not (isinstance(tt, TInt) and isinstance(vt, TInt)):
            self.error(s, f"`{op}` between {tt!r} and {vt!r}")
        if bop in ("+", "-"):
            self.track(sym, tgt, vt if bop == "+" else TInt(*iv_neg(vt)), "+=", s)
        elif bop == "*":
            if tt.lo is not None:
                self.int_result(s, *iv_mul(tt, vt))
            self.track(sym, tgt, None, "*=", s)
        elif bop in ("/", "%"):
            # same Euclidean rule as the binary operators: native only when nothing can be negative
            if vt.lo is not None and vt.hi is not None and vt.lo <= 0 <= vt.hi:
                self.error(s, f"`{unparse(s.value)}` may be 0 in `{op}`")
            if not (tt.lo is not None and tt.lo >= 0 and vt.lo is not None and vt.lo > 0):
                fn = "cp_ediv" if bop == "/" else "cp_emod"
                if self.has_effects(tgt_eval):   # evaluate the target once
                    self.emit(f"{{ auto& _t = {tc}; _t = {fn}(_t, {vc}, {s.line}); }}")
                else:
                    self.emit(f"{tc} = {fn}({tc}, {vc}, {s.line});")
                return
        unproven = bop in ("+", "-", "*") and None in (tt.lo, tt.hi, vt.lo, vt.hi) and not self.has_effects(tgt_eval)
        if unproven:
            self.emit(f"{tc} = cp_{ {'+': 'add', '-': 'sub', '*': 'mul'}[bop] }({tc}, {vc}, {s.line});")
        else:
            self.emit(f"{tc} {op} {vc};")

    def s_parallel_assign(self, s):
        """`(a, b) = (b, a + b)`: every right-hand value is computed before any target is written (v2 R4)."""
        tgt = s.target
        if s.op != "=":
            self.error(s, f"`{s.op}` has no tuple form; parallel assignment is `(a, b) = (x, y)`")
        seen = set()
        for t in tgt.items:
            if t.kind not in ("name", "index", "field"):
                self.error(s, f"`{unparse(t)}` is not a place to assign to")
            key = unparse(t)
            if key in seen:
                self.error(s, f"`{key}` is assigned twice in one parallel assignment")
            seen.add(key)
        v = s.value.a if s.value.kind == "paren" else s.value
        temps = []
        if self.io_call(v) is v and v.fn.kind != "tapp":
            # `(a[d], b[d]) = read()`: each place's type says what to read, left to right
            rest = [N("call", v.line, fn=N("name", v.line, name="read"), args=[]) for _ in tgt.items[1:]]
            v = N("tuple", v.line, items=[v] + rest, names=[None] * len(tgt.items))
            for t, x in zip(tgt.items, v.items):
                self.typed(x, self.capture(lambda: self.expr(t))[1][1])
        if v.kind == "tuple" and len(v.items) == len(tgt.items) and not any(v.names):
            for t, x in zip(tgt.items, v.items):
                want = self.capture(lambda: self.expr(t))[1][1]
                c, ty = self.expr(x, None if isinstance(want, TInt) else want)
                tmp = self.tmpname("pa")
                self.emit(f"auto {tmp} = {c};")
                temps.append(N("ctemp", x.line, code=tmp, ty=ty, src=x))
        else:
            c, ty = self.expr(v)
            if not isinstance(ty, TTuple) or len(ty.types) != len(tgt.items):
                self.error(s, f"cannot assign {ty!r} to {len(tgt.items)} places")
            tmp = self.tmpname("pa")
            self.emit(f"auto {tmp} = {c};")
            temps = [N("ctemp", v.line, code=f"{tmp}.{self.field_cpp(fn)}", ty=ft, src=v) for fn, ft in zip(ty.names, ty.types)]
        for t, x in zip(tgt.items, temps):
            self.s_assign(N("assign", s.line, target=t, op="=", value=x))

    def no_identity(self, tgt, f):
        old = f"({unparse(tgt.obj)}.get({unparse(tgt.idx[0])}) ?? start)"
        self.error(tgt, f"`{unparse(tgt)} {f}= x` may meet a key that is not in the map yet, and `{f}` has no identity "
                        f"to start it from\n  help: say what a missing key means: `{unparse(tgt)} = "
                        + (f"{f}({old}, x)`" if f[:1].isalpha() else f"{old} {f} x`"))

    def s_map_keep(self, s, mt):
        """`m[k] min= x` / `m[k] max= x`: a key that is not there yet takes x, as a `none` variable does."""
        tgt, f = s.target, s.op[:3]
        self.require_mut(tgt, s)
        self.note_write(tgt, s.line)
        oc, _ = self.expr(tgt.obj)
        kc, kt = self.expr(tgt.idx[0], mt.key)
        kc = self.coerce(kc, kt, mt.key, tgt.idx[0])
        vc, vt = self.expr(s.value, mt.val)
        vc = self.coerce(vc, vt, mt.val, s.value)
        vt_c = self.cpp_type(mt.val)
        self.emit(f"cp_mkeep({oc}, {kc}, {vt_c}({vc}), [](const {vt_c}& a, const {vt_c}& b) {{ return {f}(a, b); }});")

    def s_map_fold(self, s, mt):
        """`m[k] f= x` for a named operation: the entry is found once, starting from f's identity if it is new."""
        tgt = s.target
        self.require_mut(tgt, s)
        self.note_write(tgt, s.line)
        self.lvalue = (tgt, s.fold + "=")
        tc, tt = self.expr(tgt)
        self.lvalue = None
        place = self.tmpname("p")
        self.emit(f"auto& {place} = {tc};")
        call = N("call", s.line, fn=s.value.fn, args=[N("ctemp", tgt.line, code=place, ty=tt, src=tgt), s.value.args[1]])
        vc, vt = self.expr(call)
        self.emit(f"{place} = {self.coerce(vc, vt, tt, s.value)};")

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

    def s_fn(self, s, hoist=False):
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
        if "grader" in decos or s.body is None:
            ptypes = [self.resolve_type(p.ty) for p in s.params]
            rt = self.resolve_type(s.ret) if s.ret is not None else UNIT
            cpp = self.fresh(s.name)
            self.scopes[-1][s.name] = Sym("fn", TFn(ptypes, rt), cpp, node=s)
            pdecl = ", ".join(f"{self.cpp_type(pt)} {self.fresh(p.name)}" for p, pt in zip(s.params, ptypes))
            self.funcs.append(f"{self.cpp_type(rt)} {cpp}({pdecl}) {{ return {self.cpp_type(rt)}(); }}")
            return
        params, ptypes, names = [], [], []
        # a fn declared inside a top-level loop or `if` sees that block's variables, so it is a closure too
        # (a specialization of a generic fn is compiled where it is first called, but belongs to the top level)
        in_block = len(self.scopes) > getattr(self, "mt_scopes", 1) and not self.fn_ret and not hoist
        self.scopes.append({})
        saved_top = self.at_top
        self.at_top = False
        for p in s.params:
            if p.ty is None:
                self.error(p, f"parameter `{p.name}` needs a type")
            pt = getattr(p, "ty_fixed", None) or self.resolve_type(p.ty)
            ptypes.append(pt)
        if getattr(s, "lends", None):
            self.error(s, f"`{s.name}` is declared `lends`: a projection lends part of its receiver, so it must be a "
                          f"method with `self` (prototype: methods of a struct only)")
        rt = self.resolve_type(s.ret) if s.ret is not None else UNIT
        local = len(self.fn_ret) > (1 if getattr(self, "in_multitest", False) else 0) or not saved_top or in_block
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
            if getattr(pt, "view", None):
                mode = ""       # a view is a pointer and a window: passed by value
            params.append(f"{self.cpp_type(pt)} {mode}{pc}".replace("  ", " "))
            self.declare(p.name, pt, pc, p.mode == "inout", p)
        st = getattr(s, "self_struct", None)
        if st is not None:
            me = self.lookup("self")
            for fname, ftype in zip(st.names, st.types):
                self.declare(fname, ftype, f"{me.cpp}.{self.field_cpp(fname)}", me.mut, None)
        self.fn_ret.append(rt)
        saved_ctx, self.loop_ctx = getattr(self, "loop_ctx", []), []
        saved_lend = (self.facts, self.holds, self.cur_rest)     # a fn body knows none of its definition site's facts
        self.facts, self.holds, self.cur_rest = [], [], []
        try:
            self.fn_rest(s, decos, params, ptypes, rt, cpp, local, saved_top)
        finally:
            self.loop_ctx = saved_ctx
            self.facts, self.holds, self.cur_rest = saved_lend

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
            def memo_dim(t):
                if isinstance(t, TDom):
                    return ("0", t.info.len_cpp, t.info.len_iv[1], False, lambda pn: pn)
                if isinstance(t, TBool):
                    return ("0", "2", 2, False, lambda pn: f"(int)({pn})")
                if isinstance(t, TEnum) and all(not fns for _, fns, _ in t.variants):
                    return ("0", str(len(t.variants)), len(t.variants), False, lambda pn: f"(int)({pn}.tag)")
                if isinstance(t, TInt) and t.lo is not None and t.hi is not None and t.hi >= t.lo:
                    max_n = t.hi - t.lo + 1
                    lo_c = getattr(t, "lo_cpp", None) or str(t.lo)
                    hi_c = getattr(t, "hi_cpp", None) or str(t.hi)
                    len_c = str(max_n) if lo_c == str(t.lo) and hi_c == str(t.hi) else f"max(0LL, (ll)({hi_c}) - (ll)({lo_c}) + 1)"
                    idx_f = (lambda pn, lc=lo_c: f"(ll)({pn})" if lc == "0" else f"((ll)({pn}) - (ll)({lc}))")
                    return (lo_c, len_c, max_n, True, idx_f)
                return None
            unit_enum = lambda t: isinstance(t, TEnum) and all(not fns for _, fns, _ in t.variants)
            if local or not all(isinstance(t, (TDom, TInt, TBool)) or unit_enum(t) for t in ptypes) or isinstance(rt, TUnit):
                self.error(s, "@memo (prototype) needs a fn at top level or directly in a @multitest fn, with domain parameters")
            dims_info = [memo_dim(t) for t in ptypes]
            has_into_order = any(a.kind == "kwarg" and a.name in ("into", "order") for a in decos["memo"].args)
            hashed = any(a.kind == "name" and a.name == "hash" for a in decos["memo"].args) or any(d is None for d in dims_info)
            if not hashed and has_into_order and not all(isinstance(t, TDom) for t in ptypes):
                hashed = True
            if not hashed:
                # a dense table has one slot per point of the dimensions' product; past MEMO_DENSE_MAX slots (at the
                # dimensions' declared maximum sizes) only the points actually asked for are stored
                slots = 1
                for _, _, n, _, _ in dims_info:
                    slots = None if slots is None or n is None else slots * n
                if slots is not None and slots > MEMO_DENSE_MAX:
                    if has_into_order:
                        self.error(s, f"@memo(into/order) fills a table of up to {slots} slots, one per point of "
                                      f"{' x '.join(repr(t) for t in ptypes)}; that is past the {MEMO_DENSE_MAX} it allows")
                    hashed = True
            self.check_memo_reads(s)
            tab = f"_memo_{cpp}"
            if hashed:
                if has_into_order:
                    self.error(s, "@memo(hash) does not support into/order yet (prototype)")
                k = len(ptypes)
                self.globals.append(f"CpHashMap<array<ll, {k}>, {self.cpp_type(rt)}, CpArrHash> {tab};")
                pnames = [self.lookup(p.name).cpp for p in s.params]
                inner = self.fresh(s.name + "_body")
                body_lines, _ = self.capture(lambda: self.fn_body(s.body, rt))
                self.fn_ret.pop()
                self.scopes.pop()
                self.at_top = saved_top
                rcpp = self.cpp_type(rt)
                self.funcs.append(f"{rcpp} {inner}({', '.join(params)}) {{\n" + "\n".join(body_lines) + "\n}")
                key = ", ".join(f"(ll)({pn}.tag)" if isinstance(t, TEnum) else f"(ll){pn}" for pn, t in zip(pnames, ptypes))
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
            size = " * ".join(f"(size_t)({d[1]})" for d in dims_info)
            pnames = [self.lookup(p.name).cpp for p in s.params]
            guard_lines = []
            idx_exprs = []
            for k, (pn, d) in enumerate(zip(pnames, dims_info)):
                _, len_c, _, is_range, idx_f = d
                if is_range:
                    iv = f"_mi{k}"
                    guard_lines.append(f"    ll {iv} = {idx_f(pn)};\n    if ((unsigned long long){iv} >= (unsigned long long)({len_c})) return {cpp}_body_placeholder;")
                    idx_exprs.append(iv)
                else:
                    idx_exprs.append(idx_f(pn))
            flat = idx_exprs[0]
            for ix, d in zip(idx_exprs[1:], dims_info[1:]):
                flat = f"({flat}) * (size_t)({d[1]}) + (size_t)({ix})"
            inner = self.fresh(s.name + "_body")
            guard = "".join(g.replace(f"{cpp}_body_placeholder", f"{inner}({', '.join(pnames)})") + "\n" for g in guard_lines)
            body_lines, _ = self.capture(lambda: self.fn_body(s.body, rt))
            self.fn_ret.pop()
            self.scopes.pop()
            self.at_top = saved_top
            rcpp = self.cpp_type(rt)
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
            init_chk = ""
            self.funcs.append(f"static __attribute__((noinline)) {rcpp} {inner}({', '.join(params)});")
            if order != "ascending":
                self.funcs.append(f"static __attribute__((noinline)) void {tab}_init() {{ {tab}.assign({size}, {{}}); }}")
                init_chk = f"    if (__builtin_expect({tab}.empty(), 0)) {tab}_init();\n"
            self.funcs.append(
                f"inline {rcpp} {cpp}({', '.join(params)}) {{\n" + fill + init_chk + guard +
                f"    auto& _m = {tab}[{flat}];\n"
                f"    if (__builtin_expect(!_m.has(), 0)) _m = {inner}({', '.join(pnames)});\n"
                f"    return _m.get();\n}}")
            self.funcs.append(f"static __attribute__((noinline)) {rcpp} {inner}({', '.join(params)}) {{\n" + "\n".join(body_lines) + "\n}")
            self.funcs.insert(0, f"inline {rcpp} {cpp}({', '.join(params)});")
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
            elif into is not None:
                if into.kind != "name":
                    self.error(s, "@memo(into: name) needs a plain name")
                self.emit(f"{tab}.assign({size}, {{}});")
                self.declare(into.name, TArr(list(ptypes), TOpt(rt)), tab, True, s)
            else:
                self.emit(f"{tab}.assign({size}, {{}});")
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
                                  orelse=self.do_tail(last.orelse), is_const=getattr(last, "is_const", False))]
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
                                  orelse=self.tail_returns(last.orelse), is_const=getattr(last, "is_const", False))]
        return body

    def fn_body(self, body, rt):
        saved_n = (self.opt_narrow, self.nonempty_narrow)
        self.opt_narrow, self.nonempty_narrow = {}, {}      # a guard says nothing inside another fn's body
        try:
            self.fn_body_inner(body, rt)
        finally:
            self.opt_narrow, self.nonempty_narrow = saved_n

    def fn_body_inner(self, body, rt):
        if body and not isinstance(rt, (TUnit, TGen)) and body[-1].kind == "if" and body[-1].orelse:
            body = self.tail_returns(body)
        if body and not isinstance(rt, TUnit) and body[-1].kind == "exprstmt":
            self.block(body[:-1], new_scope=False)
            def tail():
                self.escapes[id(body[-1].expr)] = "returning it"
                c, t = self.expr(self.typed(body[-1].expr, rt), rt)
                self.emit(f"return {self.coerce(c, t, rt, body[-1])};")
            self.with_tries(body[-1], "expr", tail)
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

    def specialize(self, g, bind, const_bind=None):
        const_bind = const_bind or {}
        const_tps = getattr(g, "const_tparams", None) or {}
        def _part(tp):
            if tp in const_tps:
                v = const_bind[tp]
                return re.sub(r"[^A-Za-z0-9_]", "_", repr(v))
            t = bind[tp]
            return t.info.cpp if isinstance(t, TDom) else getattr(t, "cpp", re.sub(r"[^A-Za-z0-9_]", "_", repr(t)))
        key = (g.name,) + tuple(_part(tp) for tp in g.tparams)
        specs = self.__dict__.setdefault("specs", {})
        if key not in specs:
            import copy
            clone = copy.copy(g)
            clone.name = "_".join(key)
            clone.tparams = []
            clone.const_tparams = {}
            clone.decos = [d for d in g.decos if d.name != "brute"]
            binds = {}
            for tp in g.tparams:
                if tp in const_tps:
                    v = const_bind[tp]
                    pty = self.resolve_type(const_tps[tp]) if const_tps[tp] is not None else self.const_value_type(v)
                    binds[tp] = Sym("const", pty, self.const_to_cpp(v, pty), const_val=v)
                else:
                    t = bind[tp]
                    binds[tp] = Sym("type", t, t.info.cpp if isinstance(t, TDom) else getattr(t, "cpp", ""))
            saved = (self.scopes, self.at_top, self.fn_ret, getattr(self, "loop_ctx", []), getattr(self, "in_multitest", False))
            self.scopes, self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = [self.scopes[0], binds], True, [], [], False
            specs[key] = clone.name
            try:
                self.capture(lambda: self.s_fn(clone, hoist=True))
            finally:
                self.scopes, self.at_top, self.fn_ret, self.loop_ctx, self.in_multitest = saved
            specs[key] = binds[clone.name]
        sym = specs[key]
        return self.lookup(sym) if isinstance(sym, str) else sym

    def call_generic(self, g, e, explicit_targs=None):
        """Monomorphize a generic fn for the domains and/or const params its arguments use."""
        if len(e.args) != len(g.params):
            self.error(e, f"`{g.name}` takes {len(g.params)} arguments")
        const_tps = getattr(g, "const_tparams", None) or {}
        tdefaults = getattr(g, "tparam_defaults", None) or {}
        bind, const_bind = {}, {}
        if explicit_targs is not None:
            for tp, ta in zip(g.tparams, explicit_targs):
                if tp in const_tps:
                    const_bind[tp] = self.eval_const_typearg(ta)
                else:
                    bind[tp] = self.resolve_type(ta)
        args = [self.expr(a) for a in e.args]
        type_tps = {tp for tp in g.tparams if tp not in const_tps}
        for (_, at), p in zip(args, g.params):
            self.unify(p.ty, at, type_tps, bind, e)
        for tp in g.tparams:
            if tp in const_tps and tp not in const_bind and tp in tdefaults:
                const_bind[tp] = self.eval_const(tdefaults[tp])
            elif tp not in const_tps and tp not in bind and tp in tdefaults:
                bind[tp] = self.resolve_type(tdefaults[tp])
        missing = [tp for tp in g.tparams if tp not in bind and tp not in const_bind]
        if missing:
            self.error(e, f"cannot infer {', '.join(missing)} for `{g.name}` from its arguments")
        sym = self.specialize(g, bind, const_bind)
        if not const_tps:
            for h in [x.node for x in self.scopes[0].values() if getattr(x, "kind", None) == "genfn"]:
                bd = {d.name: d for d in h.decos}.get("brute")
                if bd and bd.args and getattr(bd.args[0], "name", None) == g.name and h.tparams == g.tparams:
                    key = ("brute", h.name) + tuple(bind[tp].info.cpp for tp in g.tparams)
                    done = self.__dict__.setdefault("brute_specs", set())
                    if key not in done:
                        done.add(key)
                        self.fn_brutes.append((sym, self.specialize(h, bind), h))
        return f"{sym.cpp}({', '.join(c for c, _ in args)})", sym.ty.ret

    # ---------- Synthesis #3 helpers: Leap 2 (local narrowing), Leap 3 (.get(?I)), Leap 4 (const POD & if const)
    def array_get(self, e, oc, ot, args):
        """a.get(?I) / a.get(I) on [D]T or [R, C]T: accepts D or ?D (strict nominal domain!), returns ?T."""
        if len(args) != len(ot.dims):
            self.error(e, f"`{unparse(e.fn.obj)}.get(...)` expects {len(ot.dims)} index(es), got {len(args)}")
        parts = []
        checks = []
        rt = TOpt(ot.elem)
        rty = self.cpp_type(rt)
        for idx_node, dim in zip(args, ot.dims):
            info = dim.info
            ic, it = self.expr(idx_node, dim)
            if isinstance(it, TOpt):
                inner = it.inner
                if isinstance(inner, TDom):
                    if inner.info is not info:
                        self.error(idx_node, f"`.get(...)` expects {info.name} or ?{info.name}, got {it!r} (wrong domain)")
                    v = self.tmpname("gi")
                    checks.append(f"Opt<int> {v} = {ic}; if (!{v}.has()) return {rty}();")
                    parts.append(f"{v}.get()")
                elif is_int(inner) and info.anon:
                    v = self.tmpname("gi")
                    checks.append(
                        f"auto {v} = {ic}; if (!{v}.has()) return {rty}(); "
                        f"ll {v}_k = (ll){v}.get() - ({info.lo_cpp}); "
                        f"if ({v}_k < 0 || {v}_k >= (ll){info.len_cpp}) return {rty}();")
                    parts.append(f"(int){v}_k")
                else:
                    self.error(idx_node, f"`.get(...)` expects {info.name} or ?{info.name}, got {it!r}")
            elif isinstance(it, TDom):
                if it.info is not info:
                    self.error(idx_node, f"`.get(...)` expects {info.name} or ?{info.name}, got {it!r} (wrong domain)")
                parts.append(ic)
            elif is_int(it):
                if not info.anon:
                    self.error(idx_node, f"`.get(...)` expects {info.name} or ?{info.name}, got {it!r} (use `{info.name}.try_at(...)` or `{info.name}(...)`)")
                v = self.tmpname("gi")
                checks.append(
                    f"ll {v}_k = (ll)({ic}) - ({info.lo_cpp}); "
                    f"if ({v}_k < 0 || {v}_k >= (ll){info.len_cpp}) return {rty}();")
                parts.append(f"(int){v}_k")
            else:
                self.error(idx_node, f"`.get(...)` expects {info.name} or ?{info.name}, got {it!r}")
        idx_expr = parts[0] if len(parts) == 1 else f"(size_t)({parts[0]}) * (size_t)({ot.dims[1].info.len_cpp}) + (size_t)({parts[1]})"
        if len(parts) > 2:
            acc = f"(size_t)({parts[0]})"
            for p, d in zip(parts[1:], ot.dims[1:]):
                acc = f"({acc} * (size_t)({d.info.len_cpp}) + (size_t)({p}))"
            idx_expr = acc
        elem_expr = f"(ll){oc}[{idx_expr}]" if self.narrow(ot.elem) else f"{oc}[{idx_expr}]"
        return f"([&]() -> {rty} {{ {' '.join(checks)} return {rty}({elem_expr}); }}())", rt

    def list_get(self, e, oc, ot, idx_node):
        """xs.get(i) / xs.get(?i) on List<T>: bounds-checked optional access returning ?T."""
        ic, it = self.expr(idx_node)
        rt = TOpt(ot.elem)
        rty = self.cpp_type(rt)
        if isinstance(it, TOpt) and is_int(it.inner):
            return (f"([&]() -> {rty} {{ auto _o = {ic}; if (!_o.has()) return {rty}(); "
                    f"ll _k = _o.get(); if (_k < 0 || _k >= (ll){oc}.size()) return {rty}(); "
                    f"return {rty}({oc}[(size_t)_k]); }}())"), rt
        if is_int(it):
            return (f"([&]() -> {rty} {{ ll _k = {ic}; if (_k < 0 || _k >= (ll){oc}.size()) return {rty}(); "
                    f"return {rty}({oc}[(size_t)_k]); }}())"), rt
        self.error(idx_node, f"List.get expects Int or ?Int, got {it!r}")

    def is_pod_type(self, t):
        if isinstance(t, (TBool, TInt, TChar, TF64)):
            return True
        if isinstance(t, TTuple) and getattr(t, "is_const_struct", False):
            return True
        return False

    def const_value_type(self, v):
        if isinstance(v, bool): return BOOL
        if isinstance(v, int): return TInt(v, v)
        if isinstance(v, float): return F64
        if isinstance(v, str) and len(v) == 1: return CHAR
        if isinstance(v, ConstStruct) and v.name in self.const_structs:
            return self.const_structs[v.name][0]
        return INT

    def const_to_cpp(self, v, ty=None):
        if isinstance(v, bool): return "true" if v else "false"
        if isinstance(v, int): return f"{v}LL"
        if isinstance(v, float): return repr(v)
        if isinstance(v, str) and len(v) == 1: return repr(v)
        if isinstance(v, ConstStruct) and v.name in self.const_structs:
            st, _ = self.const_structs[v.name]
            vals = [self.const_to_cpp(v.fields[fn], ft) for fn, ft in zip(st.names, st.types)]
            return f"{st.cpp}{{{', '.join(vals)}}}"
        return "0LL"

    def eval_const_typearg(self, ta):
        """Evaluate a const generic argument passed inside `<...>` (parsed as a type AST)."""
        if ta.kind == "texpr":
            return self.eval_const(ta.expr)
        if ta.kind == "tname" and not ta.args:
            if ta.name == "true": return True
            if ta.name == "false": return False
            return self.eval_const(N("name", ta.line, name=ta.name))
        self.error(ta, f"expected a compile-time const argument, got `{ta.kind}`")

    def eval_const(self, e):
        """Evaluate a compile-time constant expression (POD scalars, const structs, field access, boolean/arith ops)."""
        k = e.kind
        if k == "paren": return self.eval_const(e.a)
        if k == "bool": return bool(e.value)
        if k == "num": return e.value
        if k == "char": return e.value
        if k == "name":
            if e.name == "true": return True
            if e.name == "false": return False
            s = self.lookup(e.name)
            if s is not None and s.const_val is not None:
                return s.const_val
            self.error(e, f"`{e.name}` is not a compile-time constant (declare it `const` or pass it as `const` parameter)")
        if k == "field":
            ov = self.eval_const(e.obj)
            if isinstance(ov, ConstStruct):
                if e.name not in ov.fields:
                    self.error(e, f"const struct `{ov.name}` has no field `{e.name}`")
                return ov.fields[e.name]
            self.error(e, f"cannot read `.{e.name}` on non-struct const value {ov!r}")
        if k == "unop":
            a = self.eval_const(e.a)
            if e.op == "not":
                if not isinstance(a, bool): self.error(e, "`not` in const expr requires Bool")
                return not a
            if e.op == "-": return -a
            if e.op == "+": return +a
        if k == "binop":
            if e.op == "and":
                a = self.eval_const(e.a)
                return bool(a) and bool(self.eval_const(e.b))
            if e.op == "or":
                a = self.eval_const(e.a)
                return bool(a) or bool(self.eval_const(e.b))
            a, b = self.eval_const(e.a), self.eval_const(e.b)
            ops = {"+": lambda x, y: x + y, "-": lambda x, y: x - y, "*": lambda x, y: x * y,
                   "/": lambda x, y: x // y, "%": lambda x, y: x % y, "&": lambda x, y: x & y,
                   "|": lambda x, y: x | y, "^": lambda x, y: x ^ y, "<<": lambda x, y: x << y, ">>": lambda x, y: x >> y}
            if e.op in ops:
                return ops[e.op](a, b)
        if k == "cmp":
            cur = self.eval_const(e.first)
            for op, rhs in zip(e.ops, e.rest):
                rv = self.eval_const(rhs)
                ok = {"==": cur == rv, "!=": cur != rv, "<": cur < rv, "<=": cur <= rv, ">": cur > rv, ">=": cur >= rv}.get(op)
                if ok is None:
                    self.error(e, f"unsupported operator `{op}` in const expression")
                if not ok:
                    return False
                cur = rv
            return True
        if k == "call" and e.fn.kind == "name" and e.fn.name in self.const_structs:
            st, snode = self.const_structs[e.fn.name]
            fields = st.names
            vals = {}
            for i, a in enumerate(e.args):
                if a.kind == "kwarg":
                    if a.name not in fields:
                        self.error(a, f"`{e.fn.name}` has no field `{a.name}`")
                    vals[a.name] = self.eval_const(a.value)
                else:
                    if i >= len(fields):
                        self.error(a, f"`{e.fn.name}` takes at most {len(fields)} arguments")
                    vals[fields[i]] = self.eval_const(a)
            defaults = getattr(snode, "defaults", None) or {}
            for f in fields:
                if f not in vals:
                    if f in defaults:
                        vals[f] = self.eval_const(defaults[f])
                    else:
                        self.error(e, f"`{e.fn.name}(...)` is missing field `{f}`")
            return ConstStruct(e.fn.name, vals)
        self.error(e, f"expression `{unparse(e)}` is not a compile-time constant")

    def specialize_struct(self, snode, targs, node):
        """Monomorphize a generic/const-generic struct `struct Rooted<V, const C: TreeCfg = DEFAULT>`."""
        tps = getattr(snode, "tparams", [])
        const_tps = getattr(snode, "const_tparams", None) or {}
        tdefaults = getattr(snode, "tparam_defaults", None) or {}
        if len(targs) > len(tps):
            self.error(node, f"`{snode.name}` takes at most {len(tps)} type/const arguments, got {len(targs)}")
        binds = {}
        key_parts = []
        for i, tp in enumerate(tps):
            if i < len(targs):
                ta = targs[i]
                if tp in const_tps:
                    v = self.eval_const_typearg(ta)
                    pty = self.resolve_type(const_tps[tp]) if const_tps[tp] is not None else self.const_value_type(v)
                    binds[tp] = Sym("const", pty, self.const_to_cpp(v, pty), const_val=v)
                    key_parts.append(v.key() if isinstance(v, ConstStruct) else v)
                else:
                    t = self.resolve_type(ta)
                    binds[tp] = Sym("type", t, t.info.cpp if isinstance(t, TDom) else getattr(t, "cpp", ""))
                    key_parts.append(t.info.cpp if isinstance(t, TDom) else repr(t))
            elif tp in tdefaults:
                if tp in const_tps:
                    v = self.eval_const(tdefaults[tp])
                    pty = self.resolve_type(const_tps[tp]) if const_tps[tp] is not None else self.const_value_type(v)
                    binds[tp] = Sym("const", pty, self.const_to_cpp(v, pty), const_val=v)
                    key_parts.append(v.key() if isinstance(v, ConstStruct) else v)
                else:
                    t = self.resolve_type(tdefaults[tp])
                    binds[tp] = Sym("type", t, t.info.cpp if isinstance(t, TDom) else getattr(t, "cpp", ""))
                    key_parts.append(t.info.cpp if isinstance(t, TDom) else repr(t))
            else:
                self.error(node, f"`{snode.name}` is missing type/const argument `{tp}`")
        key = (snode.name, tuple(key_parts))
        if key in self.gen_struct_insts:
            return self.gen_struct_insts[key]
        inst_name = f"{snode.name}_{len(self.gen_struct_insts) + 1}"
        t = self.struct_inst(snode, inst_name, binds=binds)
        self.gen_struct_insts[key] = t
        return t

    def construct_struct(self, st, snode, e):
        """Construct a struct value, handling defaults and erased `()` fields."""
        all_fields = getattr(st, "all_field_names", st.names)
        erased = getattr(st, "erased_fields", set())
        items, names = [], []
        for i, a in enumerate(e.args):
            if a.kind == "kwarg":
                items.append(a.value); names.append(a.name)
            else:
                if i >= len(all_fields):
                    self.error(a, f"{snode.name} has {len(all_fields)} fields")
                items.append(a); names.append(all_fields[i])
        defaults = getattr(snode, "defaults", None) or {}
        for f in all_fields:
            if f not in names:
                if f in defaults:
                    items.append(defaults[f]); names.append(f)
                elif f in erased:
                    items.append(None); names.append(f)
        if sorted(names) != sorted(all_fields):
            missing = [f for f in all_fields if f not in names]
            extra = [f for f in names if f not in all_fields]
            self.error(e, f"{snode.name}(...) is missing {', '.join(missing) or 'nothing'}"
                          + (f" / unknown {', '.join(extra)}" if extra else ""))
        kept_items, kept_names = [], []
        for f in st.names:
            k = names.index(f)
            kept_items.append(items[k])
            kept_names.append(f)
        if not kept_names:
            return f"{st.cpp}{{}}", st
        lit = N("tuple", e.line, items=kept_items, names=kept_names)
        return self.x_tuple(lit, st)

    # ---------- Synthesis #3 Leap 2: Strict Local Syntactic Narrowing (Rules N1/N2 + 4 Anti-Spookiness Invariants)
    def extract_narrow(self, cond, truthy=True):
        """Extract local syntactic narrowing facts from `cond` when `cond` is `truthy`.
        Returns (opt_facts: [(name, Sym, inner_ty)], nonempty_facts: [(name, Sym)]).
        Strictly syntactic (Invariant 1) and local-binding only (Invariant 2)."""
        opt_f, ne_f = [], []
        if cond is None:
            return opt_f, ne_f
        k = cond.kind
        if k == "paren":
            return self.extract_narrow(cond.a, truthy)
        if k == "unop" and cond.op == "not":
            return self.extract_narrow(cond.a, not truthy)
        if k == "binop" and ((cond.op == "and" and truthy) or (cond.op == "or" and not truthy)):
            o1, n1 = self.extract_narrow(cond.a, truthy)
            o2, n2 = self.extract_narrow(cond.b, truthy)
            return o1 + o2, n1 + n2
        if k == "cmp" and len(cond.ops) == 1:
            op, a, b = cond.ops[0], cond.first, cond.rest[0]
            if op in ("!=", "=="):
                is_not_none = (op == "!=" and truthy) or (op == "==" and not truthy)
                if is_not_none:
                    target = a if b.kind == "none" else (b if a.kind == "none" else None)
                    # Invariant 2: ONLY local bindings (`name`), NEVER array elements `a[i]` or struct fields `s.x`!
                    if target is not None and target.kind == "name":
                        sym = self.lookup(target.name)
                        if sym is not None and sym.kind in ("let", "var") and isinstance(sym.ty, TOpt):
                            opt_f.append((target.name, sym, sym.ty.inner))
            if truthy:
                ne_name = None
                if op == ">" and a.kind == "field" and a.name == "len" and a.obj.kind == "name" and b.kind == "num" and b.value == 0:
                    ne_name = a.obj.name
                elif op == "<" and b.kind == "field" and b.name == "len" and b.obj.kind == "name" and a.kind == "num" and a.value == 0:
                    ne_name = b.obj.name
                elif op == ">=" and a.kind == "field" and a.name == "len" and a.obj.kind == "name" and b.kind == "num" and b.value >= 1:
                    ne_name = a.obj.name
                elif op == "<=" and b.kind == "field" and b.name == "len" and b.obj.kind == "name" and a.kind == "num" and a.value >= 1:
                    ne_name = b.obj.name
                elif op == "!=" and a.kind == "field" and a.name == "len" and a.obj.kind == "name" and b.kind == "num" and b.value == 0:
                    ne_name = a.obj.name
                elif op == "!=" and b.kind == "field" and b.name == "len" and b.obj.kind == "name" and a.kind == "num" and a.value == 0:
                    ne_name = b.obj.name
                if ne_name is not None:
                    sym = self.lookup(ne_name)
                    if sym is not None and sym.kind in ("let", "var"):
                        ne_f.append((ne_name, sym))
            else:
                ne_name = None
                if op == "==" and a.kind == "field" and a.name == "len" and a.obj.kind == "name" and b.kind == "num" and b.value == 0:
                    ne_name = a.obj.name
                elif op == "==" and b.kind == "field" and b.name == "len" and b.obj.kind == "name" and a.kind == "num" and a.value == 0:
                    ne_name = b.obj.name
                if ne_name is not None:
                    sym = self.lookup(ne_name)
                    if sym is not None and sym.kind in ("let", "var"):
                        ne_f.append((ne_name, sym))
        if not truthy and k == "field" and cond.name == "is_empty" and cond.obj.kind == "name":
            sym = self.lookup(cond.obj.name)
            if sym is not None and sym.kind in ("let", "var"):
                ne_f.append((cond.obj.name, sym))
        return opt_f, ne_f

    def apply_narrow(self, facts):
        opt_f, ne_f = facts
        for nm, sym, inner_t in opt_f:
            self.opt_narrow[nm] = (sym, inner_t)
        for nm, sym in ne_f:
            self.nonempty_narrow[nm] = sym

    def drop_narrow(self, names):
        for w in names:
            self.opt_narrow.pop(w, None)
            self.nonempty_narrow.pop(w, None)

    def invalidate_narrow(self, expr_node):
        """Invariant 3: invalidate local narrowing whenever a binding (or root of a mutated place) is mutated."""
        root = expr_node
        while root is not None and root.kind in ("paren", "unwrap", "index", "field"):
            root = root.a if root.kind in ("paren", "unwrap") else root.obj
        if root is not None and root.kind == "name":
            self.opt_narrow.pop(root.name, None)
            self.nonempty_narrow.pop(root.name, None)

    def always_diverges(self, stmts):
        if not stmts:
            return False
        for st in stmts:
            if st.kind in ("return", "break", "continue"):
                return True
            if st.kind == "if" and st.orelse and self.always_diverges(st.body) and self.always_diverges(st.orelse):
                return True
        return False

    def is_local_nonempty(self, obj):
        if obj.kind != "name":
            return False
        sym = self.lookup(obj.name)
        return sym is not None and self.nonempty_narrow.get(obj.name) is sym

    def maybe_n2_field(self, e, ot, f, rc, rt):
        if not isinstance(rt, TOpt) or not self.is_local_nonempty(e.obj):
            return rc, rt
        ok = False
        if isinstance(ot, TList) and f in ("first", "last"): ok = True
        elif isinstance(ot, THeap) and f == "peek": ok = True
        elif isinstance(ot, TRt) and ot.name == "Deque" and f in ("first", "last", "front", "back"): ok = True
        elif isinstance(ot, TRt) and ot.name == "MonoDeque" and f == "best": ok = True
        elif isinstance(ot, (TSet, TMap)) and f in ("first", "last", "min", "max") and not ot.hashed: ok = True
        elif isinstance(ot, TTuple) and self.struct_laws.get((ot.cpp, f)) == "not_empty": ok = True
        if ok:
            self.narrowed_exprs.add(id(e))
            return f"({rc}).get()", rt.inner
        return rc, rt

    def maybe_n2_method(self, e, obj, ot, m, rc, rt, was_nonempty=None):
        if was_nonempty is None:
            was_nonempty = self.is_local_nonempty(obj)
        if not isinstance(rt, TOpt) or not was_nonempty:
            return rc, rt
        ok = False
        if isinstance(ot, TList) and m == "pop": ok = True
        elif isinstance(ot, THeap) and m == "pop": ok = True
        elif isinstance(ot, TRt) and ot.name == "Deque" and m in ("pop", "pop_front", "pop_back"): ok = True
        elif isinstance(ot, TTuple) and self.struct_laws.get((ot.cpp, m)) == "not_empty": ok = True
        if ok:
            self.narrowed_exprs.add(id(e))
            return f"({rc}).get()", rt.inner
        return rc, rt

    def is_mutating_method(self, ot, m):
        if m in ("push", "push_front", "push_back", "pop", "pop_front", "pop_back", "clear", "insert",
                 "remove", "erase", "swap", "sort", "rev", "fill", "resize", "truncate", "set", "add",
                 "unite", "refresh", "update", "assign", "extend", "append"):
            return True
        if isinstance(ot, TTuple):
            tbl = getattr(self, "struct_methods", {}).get(ot.cpp, {})
            if m in tbl and tbl[m][2]:
                return True
        return False

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
        if getattr(t, "hashed", False) or getattr(t, "bag", False):
            return False
        e = t.elem
        return isinstance(e, TDom) and e.info.len_iv[1] is not None and e.info.len_iv[1] <= 64

    def set_ctor(self, t):
        if isinstance(t, TMap) and isinstance(t.key, TDom):
            return f"{self.cpp_type(t)}({t.key.info.len_cpp})"
        if isinstance(t, TSet) and (t.hashed or t.bag):
            return f"{self.cpp_type(t)}()"
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
        while root.kind in ("index", "field") or self.is_proj_call(root):
            root = root.fn.obj if root.kind == "call" else root.obj
        if root.kind == "name":
            sym = self.lookup(root.name)
            if sym is not None:
                self.writes.append((sym, line))
            self.drop_facts({root.name})
            self.check_access(target, True)

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
        saved_ret = getattr(self, "prog_ret", None)
        self.prog_ret = ret = [len(self.fn_ret), self.tmpname("cp_next" if mt else "cp_end"), False]
        def body():
            holder["out"] = self.out
            self.block(s.body, new_scope=False)
        lines, _ = self.capture(body)
        self.prog_ret = saved_ret
        self.fn_ret.pop()
        self.at_top = saved_top
        self.in_multitest = False
        self.scopes.pop()
        if mt:
            pos = self.read_pos[1] if self.read_pos and self.read_pos[0] == id(holder["out"]) else 0
            lines = lines[:pos] + ["    if (!CP_GEN) {"] + ["    " + l for l in lines[pos:]] + ["    }"]
        if ret[2]:      # a `return` in the body: leave the block, so the jump crosses no declaration
            lines = ["    {"] + ["    " + l for l in lines] + ["    }", f"    {ret[1]}:;"]
        if mt:
            lines = (["    ll _T = CP_GEN ? gen_int(1, 3) : rd_int();",
                      "    if (CP_GEN) { out_int(_T); out_char('\\n'); }",
                      "    while (_T--) {"] + ["    " + l for l in lines] + ["    }"])
        when = None
        if "brute" in decos:
            for a in decos["brute"].args:
                if a.kind == "kwarg" and a.name == "when":
                    when, _ = self.capture(lambda: self.expr(a.value))[1]
        self.solutions[name] = (lines, "multitest" in decos, when)
        if "brute" in decos:
            self.__dict__.setdefault("brute_solutions", set()).add(name)


def pat_names(p):
    if p.kind == "pname":
        return [p.name]
    out = []
    for q in p.items:
        out += pat_names(q)
    return out

# ---------- I/O emitters and driver (attached to Compiler)
def _emit_read(self, target, ty, node, packed=False):
    """Emit code reading (or, in gen mode, generating) a value of type ty into target. `packed`: the value is
    one of a run (an array), so a one-character tag is written without a space after it."""
    ln = node.line
    if isinstance(ty, TF64):
        self.emit(f"{target} = CP_GEN ? (double)gen_int(-1000000, 1000000) / 1000.0 : rd_f64({ln});")
        self.emit(f"if (CP_GEN) {{ out_str(cp_fmt_f({target}, 3, 0, false)); out_char(' '); }}")
    elif isinstance(ty, (TU64, TI128)):
        self.emit(f"{target} = ({self.cpp_type(ty)})(CP_GEN ? gen_int(0, 1000000000) : rd_int());")
        self.emit(f"if (CP_GEN) {{ out_str(cp_wide_str({target})); out_char(' '); }}")
    elif isinstance(ty, TInt):
        lo = getattr(ty, "lo_cpp", None) or ("LLONG_MIN" if ty.lo is None else f"{ty.lo}LL")
        hi = getattr(ty, "hi_cpp", None) or ("LLONG_MAX" if ty.hi is None else f"{ty.hi}LL")
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
    elif isinstance(ty, TEnum) and ty.tags and any(isinstance(t, tuple) for t in ty.tags):
        # char / char-range tags: `@tags('.', '*', '1'..='9') enum Cell: Empty | Wall | Coin(val: 1..=9)`
        cvar = self.tmpname("c")
        self.emit(f"{{ char {cvar} = rd_char();")
        self.ind += 1
        first = True
        for k, (tag, (vn, fns, fts)) in enumerate(zip(ty.tags, ty.variants)):
            kw = "if" if first else "else if"
            first = False
            if isinstance(tag, tuple) and tag[0] == "char":
                self.emit(f"{kw} ({cvar} == {repr(tag[1])}) {{ {target}.tag = {k}; }}")
            elif isinstance(tag, tuple) and tag[0] == "charrange":
                _, lo_c, hi_c, incl = tag
                cmp_op = "<=" if incl else "<"
                assign = ""
                if len(fns) == 1:
                    ft = fts[0]
                    if is_int(ft):
                        base = f"({ft.lo}LL)" if ft.lo is not None else "0LL"
                        assign = f" {target}.{vn}_{fns[0]} = (ll)({cvar} - {repr(lo_c)}) + {base};"
                    elif isinstance(ft, TChar):
                        assign = f" {target}.{vn}_{fns[0]} = {cvar};"
                self.emit(f"{kw} ({cvar} >= {repr(lo_c)} && {cvar} {cmp_op} {repr(hi_c)}) {{ {target}.tag = {k};{assign} }}")
        self.emit(f"else cp_fail(\"unknown tag\", {ln});")
        self.ind -= 1
        self.emit("}")
    elif isinstance(ty, TEnum) and ty.tags and all(len(t) == 1 for t in ty.tags) and not any(f for _, f, _ in ty.variants):
        # tags of one character are read a character at a time, so `WLLW` and `W L L W` are the same input
        # (this is also every `.`/`#` grid); a generated array of them is written packed
        tags = "".join(ty.tags)
        chain = " : ".join(f"c == {repr(t)} ? {k}" for k, t in enumerate(ty.tags))
        self.emit(f"if (CP_GEN) {{ {target}.tag = (int)gen_int(0, {len(tags) - 1}); out_char({json.dumps(tags)}[{target}.tag]);"
                  f"{'' if packed else chr(32) + 'out_char(' + repr(' ') + ');'} }}")
        self.emit(f"else {{ char c = rd_char(); {target}.tag = {chain} : (cp_fail(\"unknown tag\", {ln}), 0); }}")
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
        if not any(fns for _, fns, _ in ty.variants):
            names = ", ".join(json.dumps(vn) for vn, _, _ in ty.variants)
            self.error(node, f"reading a `{ty!r}` needs to know how its variants are written in the input: "
                             f"put `@tags({names})` above the enum (it is not read as a number or by guessing)")
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
    elif isinstance(ty, TOpt) and is_int(ty.inner):
        sentinel = getattr(self, "read_none", None)
        if sentinel is not None:
            w = self.tmpname("w")
            self.emit(f"{{ string {w} = rd_word(); {target} = ({w} == {json.dumps(sentinel)}) ? Opt<ll>() : Opt<ll>(stoll({w})); }}")
        else:
            x = self.tmpname("r")
            self.emit(f"{{ ll {x} = rd_int(); {target} = ({x} == -1) ? Opt<ll>() : Opt<ll>({x}); }}")
    elif isinstance(ty, TOpt) and isinstance(ty.inner, TDom):
        info = ty.inner.info
        x = self.tmpname("r")
        self.emit(f"{{ ll {x} = CP_GEN ? (gen_int(0, 1) ? -1 : {info.lo_cpp} + gen_int(0, {info.len_cpp} - 1)) : rd_int();")
        self.emit(f"if (CP_GEN) {{ out_int({x}); out_char(' '); }}")
        self.emit(f"{target} = {x} == -1 ? Opt<int>() : Opt<int>(chk_dom({x} - {info.lo_cpp}, {info.len_cpp}, {ln})); }}")
    elif isinstance(ty, TRt) and ty.name == "Matrix":
        r, c, el = ty.args[0].info, ty.args[1].info, ty.args[2]
        self.emit(f"{target} = CpMat<{self.cpp_type(el)}>({r.len_cpp}, {c.len_cpp});")
        ir, ic = self.tmpname("r"), self.tmpname("c")
        self.emit(f"for (int {ir} = 0; {ir} < {r.len_cpp}; ++{ir}) for (int {ic} = 0; {ic} < {c.len_cpp}; ++{ic}) {{")
        self.ind += 1
        self.emit_read(f"{target}.at({ir}, {ic})", el, node)
        self.ind -= 1
        self.emit("}")
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
        self.emit_read(f"{target}[{v}]", ty.elem, node, packed=True)
        word = isinstance(ty.elem, TEnum) and ty.elem.tags and all(len(t) == 1 for t in ty.elem.tags)
        if word and len(ty.dims) > 1:       # packed tags: one word of the input for each row
            self.emit(f"if (CP_GEN && ({v} + 1) % (size_t){ty.dims[-1].info.len_cpp} == 0) out_char('\\n');")
        self.ind -= 1
        self.emit("}")
        if word:
            self.emit("if (CP_GEN) out_char(' ');")
    else:
        self.error(node, f"cannot read {ty!r}")

def _emit_input(self, target, ty, node, packed=False):
    """Write a value a `gen:` fn made as input text: the format `_emit_read` reads and its generator writes."""
    if isinstance(ty, (TInt, TBool)):
        self.emit(f"out_int({target}); out_char(' ');")
    elif isinstance(ty, TDom):
        self.emit(f"out_int({self.dom_label(ty.info, target)}); out_char(' ');")
    elif isinstance(ty, TChar):     # the characters of a table are one word for each row
        self.emit(f"out_char({target});{'' if packed else ' out_char(' + repr(' ') + ');'}")
    elif isinstance(ty, TStr):
        self.emit(f"out_str({target}); out_char(' ');")
    elif isinstance(ty, TTuple):
        for fn, ft in zip(ty.names, ty.types):
            _emit_input(self, f"{target}.{self.field_cpp(fn)}", ft, node)
    elif isinstance(ty, TEnum) and ty.tags and all(len(t) == 1 for t in ty.tags) and not any(f for _, f, _ in ty.variants):
        self.emit(f"out_char({json.dumps(''.join(ty.tags))}[{target}.tag]);{'' if packed else ' out_char(' + repr(' ') + ');'}")
    elif isinstance(ty, TOpt) and isinstance(ty.inner, TDom):
        self.emit(f"out_int({target}.has() ? {target}.get() + {ty.inner.info.lo_cpp} : -1); out_char(' ');")
    elif isinstance(ty, TPerm) and ty.elem.info.labels != "char":
        self.emit(f"for (auto _x : {target}) {{ out_int(_x + {ty.elem.info.lo_cpp}); out_char(' '); }}")
    elif isinstance(ty, (TList, TArr)) and not isinstance(ty, TPerm):
        if isinstance(ty, TList):
            self.emit(f"out_int((ll){target}.size()); out_char(' ');")
        v = self.tmpname("k")
        self.emit(f"for (size_t {v} = 0; {v} < {target}.size(); ++{v}) {{")
        self.ind += 1
        _emit_input(self, f"{target}[{v}]", ty.elem, node, packed=isinstance(ty, TArr))
        word = isinstance(ty, TArr) and (isinstance(ty.elem, TChar) or (
            isinstance(ty.elem, TEnum) and ty.elem.tags and all(len(t) == 1 for t in ty.elem.tags)))
        if word and len(ty.dims) > 1:
            self.emit(f"if (({v} + 1) % (size_t){ty.dims[-1].info.len_cpp} == 0) out_char('\\n');")
        elif isinstance(ty.elem, (TList, TArr)):
            self.emit("out_char('\\n');")
        self.ind -= 1
        self.emit("}")
        if word:
            self.emit("out_char(' ');")
    elif isinstance(ty, TEnum):
        if ty.tags:
            names = ", ".join(json.dumps(t) for t in ty.tags)
            self.emit(f"{{ static const char* _tn[] = {{{names}}}; out_str(_tn[{target}.tag]); out_char(' '); }}")
        else:
            self.emit(f"out_int({target}.tag); out_char(' ');")
        for k, (vn, fns, fts) in enumerate(ty.variants):
            if not fns:
                continue
            self.emit(f"if ({target}.tag == {k}) {{")
            self.ind += 1
            for f, ft in zip(fns, fts):
                _emit_input(self, f"{target}.{vn}_{f}", ft, node)
            self.ind -= 1
            self.emit("}")
    else:       # no input format of its own: written as `print` writes it
        self.emit_value(target, ty, node)

def _emit_print(self, args, newline, node):
    # in input-generation mode the whole program runs, but only the generated input is printed
    self.emit("if (!CP_GEN) {")
    self.ind += 1
    _emit_print_body(self, args, newline, node)
    self.ind -= 1
    self.emit("}")

def _emit_print_body(self, args, newline, node):
    first = True
    # every argument is evaluated before anything is written: a later argument that asks a question or
    # prints must not find the first words of this line already in the output
    # `print(ans, none: "IMPOSSIBLE")`: what a `none` is written as (the same keyword `read` takes)
    kws = {a.name: a.value for a in args if a.kind == "kwarg"}
    args = [a for a in args if a.kind != "kwarg"]
    for k, v in kws.items():
        if k != "none" or v.kind != "str":
            self.error(node, "`print` takes one keyword, `none: \"text\"`")
    self.print_none = json.dumps(kws["none"].value) if "none" in kws else None
    late = any(w or io for w, _, io in (self.effects_in(a) for a in args[1:]))
    vals = []
    for a in args:
        c, t = self.expr(a)
        if late and a.kind not in ("num", "char", "bool", "str"):
            tmp = self.tmpname("pa")
            self.emit(f"auto {tmp} = {c};")
            c = tmp
        vals.append((c, t))
    for a, (c, t) in zip(args, vals):
        if not first:
            self.emit("out_char(' ');")
        first = False
        self.emit_value(c, t, a)
    if newline:
        self.emit("out_char('\\n');")

def _emit_value(self, c, t, node):
    if isinstance(t, TF64):
        self.error(node, "print of a bare F64 does not say how many digits: format it, `print(\"{x:.6f}\")`")
    if isinstance(t, (TU64, TI128)):
        self.emit(f"out_str(cp_wide_str({c}));")
    elif isinstance(t, TInt):
        self.emit(f"out_int({c});")
    elif isinstance(t, TDom):
        lab = self.dom_label(t.info, c)
        self.emit(f"out_char((char){lab});" if t.info.labels == "char" else f"out_int({lab});")
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
        none = f"out_str({self.print_none})" if getattr(self, "print_none", None) else "out_int(-1)"
        self.emit(f"{{ auto {o} = {c}; if (!{o}.has()) {none}; else {{")
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


DECL_KINDS = ("fn", "struct", "enum", "typedef", "impl", "trait", "use")


def _program(self):
    self.out = self.init_lines
    self.memo_reads, self.writes, self.fn_brutes, self.fn_props = [], [], [], []
    body = self.prog.body
    is_mt = lambda st: st.kind == "directive" or any(d.name == "multitest" for d in getattr(st, "decos", []))
    marks = [i for i, st in enumerate(body) if is_mt(st)]
    mt_idx = marks[0] if marks else None
    for i in marks:
        if body[i].kind == "directive" and body[i].name != "multitest":
            self.error(body[i], f"unknown directive `@!{body[i].name}` (the one file-level directive is `@!multitest`)")
    if len(marks) > 1:
        self.error(body[marks[1]], "a second `@!multitest`: a file has one divider, and everything below it runs once per test case")
    if mt_idx is not None and body[mt_idx].kind == "fn" and not body[mt_idx].params and not any(
            st.kind not in DECL_KINDS for st in body[mt_idx + 1:]):
        mt_idx = None       # `@multitest fn solve():` with only declarations after it: the older wrapper form
    self.prog_ret = done = [0, "_cp_done", False]
    if mt_idx is not None:
        self.block(body[:mt_idx], new_scope=False)
        st = copy.copy(body[mt_idx])
        if st.kind == "directive":
            mt_deco, rest = st, body[mt_idx + 1:]
        else:       # deprecated spelling: a bare `@multitest` on the first statement of the test case
            mt_deco = next(d for d in st.decos if d.name == "multitest")
            st.decos = [d for d in st.decos if d.name != "multitest"]
            rest = [st] + body[mt_idx + 1:]
        args = mt_deco.args
        is_eof = any(a.kind == "name" and a.name == "until_eof" for a in args)
        cnt_kw = [a.value for a in args if a.kind == "kwarg" and a.name == "count"]
        if is_eof:
            self.emit("while (cp_has_more()) {")
        elif cnt_kw:
            cc, _ = self.expr(cnt_kw[0])
            self.emit(f"for (ll _mt = (ll)({cc}), _mi = 0; _mi < _mt; ++_mi) {{")
        else:
            self.emit("ll _mt = CP_GEN ? gen_int(1, 3) : rd_int(); if (CP_GEN) { out_int(_mt); out_char('\\n'); }")
            self.emit("for (ll _mi = 0; _mi < _mt; ++_mi) {")
        self.ind += 1
        saved_mt = getattr(self, "in_multitest", False)
        self.in_multitest = True
        self.mt_scopes = len(self.scopes) + 1       # a fn directly below the divider is a top-level fn of its test case
        self.prog_ret = nxt = [0, "_cp_next", False]
        start = len(self.out)
        self.block(rest, new_scope=True)
        if nxt[2]:      # `return` ends this test case: leave the block, so the jump crosses no declaration
            self.out[start:] = ["    " * self.ind + "{"] + \
                ["    " + l for l in self.out[start:]] + ["    " * self.ind + "}", "    " * self.ind + "_cp_next:;"]
        self.prog_ret = done
        self.mt_scopes = 1
        self.in_multitest = saved_mt
        self.ind -= 1
        self.emit("}")
        self.last_read = len(self.init_lines)
    else:
        self.block(body, new_scope=False)
    self.returns_at_top = done[2]
    self.prog_ret = None
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
    for _ in range(4):
        nxt = re.sub(r"// @part ([\w|]+)\n(.*?)// @end\n", keep, code, flags=re.S)
        if nxt == code:
            break
        code = nxt
    return (code,) + tuple(res[1:]) if isinstance(res, tuple) else code


def _render(comp):
    import os
    rt = open(os.path.join(os.path.dirname(__file__), "runtime.hpp")).read()
    sols = comp.solutions
    init = comp.init_lines
    gen_exit = "    if (CP_GEN && !CP_FNTEST_ON) { out_flush(); return 0; }"
    brutes = getattr(comp, "brute_solutions", set())
    if all(n in brutes for n in sols):
        # the top-level statements are the program; a parameterless @brute is only ever an oracle, picked with
        # CP_SOLUTION=<name> by `cp.py stress`, and never runs in a judge build (backlog W6)
        sols = {"main": (init[comp.last_read:], False, None), **sols}
        init = init[:comp.last_read]
    parts = [rt, "#ifndef CP_GEN_MODE\n#define CP_GEN 0\n#else\n#define CP_GEN 1\n#endif",
             "#ifndef CP_FNTEST\n#define CP_FNTEST_ON 0\n#else\n#define CP_FNTEST_ON 1\n#endif"]
    parts += comp.struct_defs + comp.globals + comp.funcs
    out = []
    multitest = any(m for _, m, _ in sols.values())
    out.append("int main(int argc, char** argv) {")
    out.append("    if (CP_GEN) { if (argc > 1) cp_rng.seed(atoll(argv[1])); if (argc > 2) cp_size = atoll(argv[2]); }")
    out.append("    const char* cp_sol = getenv(\"CP_SOLUTION\");")
    if comp.returns_at_top:     # a top-level `return` leaves these blocks, so the jump crosses no declaration
        init = ["    {"] + ["    " + l for l in init] + ["    }"]
    out += init
    if not any(m for _, m, _ in sols.values()) and not comp.reads_in_fns:
        out.append(gen_exit)
    names = list(sols)
    main_name = next((n for n in names if n == "main"), None) or next(n for n in names if n not in brutes)
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
    if comp.returns_at_top:
        out.append("    _cp_done:;")
    if comp.fntest_lines:
        out.append("#ifdef CP_FNTEST\n    {\n    out_flush(); printf(\"@@FNTEST\\n\"); fflush(stdout);")
        out += ["    " + l for l in comp.fntest_lines]
        out.append("    out_flush(); return 0;\n    }\n#endif")
    out.append("    out_flush();\n    return 0;\n}")
    code = "\n\n".join(parts) + "\n\n" + "\n".join(out) + "\n"
    return code, comp
