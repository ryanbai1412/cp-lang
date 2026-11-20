"""A tree-walking interpreter for library code.

Types are ordinary runtime values here, so generic structs, operation
values and fresh domains need no specialization pass: `Fenwick<V, Int>` is
the struct plus a tuple of two type values. The compiler will do the same
work at compile time; this file is the executable reference for it.

What it checks while running: array indices carry their domain (indexing
`[V]T` with an `E` or an `Int` stops the test), bounds, `Int` overflow,
division by zero, unwrapping `none`, and every `assert`.
"""
import os
import re
import random

from .parser import LibParser, ParseError
from .values import *
from cpc.ast import N
from cpc.lexer import LexError

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OPS = {"+": "add", "-": "sub", "*": "mul", "/": "div", "%": "rem"}
# what `m[k] f= x` starts from when k is new: the identity of f (decisions.md section 12)
FOLD_IDENTITY = {"+": 0, "-": 0, "*": 1, "|": 0, "^": 0, "gcd": 0, "lcm": 1}
ESC = {"n": "\n", "t": "\t", "\\": "\\", '"': '"', "'": "'", "0": "\0", "{": "{", "}": "}"}


def same_fn(a, b):
    """Two fn values are equal when they are the same fn; a bare operator (`+`, `max`) equals itself."""
    if a is b:
        return True
    if type(a) is Closure and type(b) is Closure:
        if a.node is b.node and a.env is b.env:
            return True
        x, y = a.node, b.node
        return x.kind == "lambda" and y.kind == "lambda" and is_bare_op(x) and is_bare_op(y) and x.body.op == y.body.op
    return False


def is_bare_op(lam):
    b = lam.body
    return (b.kind == "binop" and b.a.kind == "name" and b.a.name == "_a" and b.b.kind == "name" and b.b.name == "_b")


def is_gen(fn):
    return fn.kind == "fn" and fn.ret is not None and fn.ret.kind == "tyields"


DECL_KINDS = ("fn", "struct", "enum", "typedef", "impl", "trait", "use")


class Interp:
    def __init__(self, stdin_tokens=None, out=None, stdin_stream=None):
        self.modules = {}
        self.trait_defaults = {}
        self.loading = []
        self.line = 0
        self.mod = None
        self.depth = 0
        self.rng = random.Random(1)
        self.tokens = list(stdin_tokens) if stdin_tokens else []
        self.tok_i = 0
        self.stdin_stream = stdin_stream
        self.out = out if out is not None else []
        self.methods = {}
        self.last_call = None
        self.globals = Env()
        from . import builtins
        builtins.install(self)
        Obj.interp = self

    # ---- errors
    def fail(self, msg, node=None):
        line = node.line if node is not None and getattr(node, "line", 0) else self.line
        where = os.path.relpath(self.mod.path) if self.mod else "?"
        raise CpError(f"{where}:{line}: {msg}")

    # ---- modules
    def load(self, path, name=None):
        path = os.path.realpath(path)
        if path in self.modules:
            return self.modules[path]
        if path in self.loading:
            raise CpError(f"cyclic import through {os.path.relpath(path)}")
        with open(path) as f:
            src = f.read()
        try:
            tree = LibParser(src).program()
        except (ParseError, LexError) as e:
            raise CpError(f"{os.path.relpath(path)}: {e}")
        mod = Module(name or os.path.splitext(os.path.basename(path))[0], path, Env(self.globals))
        mod.src = src.split("\n")
        self.loading.append(path)
        saved = self.mod, self.line
        self.mod = mod
        try:
            self.run_program(tree.body, mod)
        finally:
            self.mod, self.line = saved
            self.loading.pop()
        self.modules[path] = mod
        return mod

    def run_program(self, body, mod):
        """Top-level statements run in order. `@!multitest` on its own line divides the file: what is above
        runs once, what is below runs once per test case (the count is read first), each time in a fresh
        scope, so its `let`s and its fns start over (cpi keeps no `@memo` cache: a `@memo fn` is pure, and
        runs as written). A top-level `return` ends the program, or
        the current test case below the divider (decision 6.3)."""
        is_mt = lambda s: s.kind == "directive" or any(d.name == "multitest" for d in getattr(s, "decos", ()))
        marks = [i for i, s in enumerate(body) if is_mt(s)]
        for i in marks:
            if body[i].kind == "directive" and body[i].name != "multitest":
                self.fail(f"unknown directive `@!{body[i].name}` (the one file-level directive is `@!multitest`)", body[i])
        if len(marks) > 1:
            self.fail("a second `@!multitest`: a file has one divider, and everything below it runs once per test case", body[marks[1]])
        cut = marks[0] if marks else len(body)
        wrapper = None
        if marks and body[cut].kind == "fn" and not body[cut].params and not any(
                s.kind not in DECL_KINDS for s in body[cut + 1:]):
            wrapper, cut = body[cut], len(body)     # `@multitest fn solve():`, the older wrapper form
        try:
            for s in body[:cut]:
                self.declare(s, mod)
        except ReturnEx as r:
            self.top_return(r)
            return
        until_eof = False
        if wrapper is not None:
            rest = wrapper.body
            deco = next(d for d in wrapper.decos if d.name == "multitest")
        elif marks and body[cut].kind == "directive":
            rest, deco = body[cut + 1:], body[cut]
        elif marks:         # deprecated spelling: a bare `@multitest` on the first statement of the test case
            rest = body[cut:]
            deco = next(d for d in rest[0].decos if d.name == "multitest")
        else:
            return
        until_eof = any(a.kind == "name" and a.name == "until_eof" for a in deco.args)
        count = next((a.value for a in deco.args if a.kind == "kwarg" and a.name == "count"), None)
        if until_eof:
            tests = None
        elif count is not None:
            tests = self.ev(count, mod.env)
        else:
            if not self.more_input():
                self.fail("@multitest: the input ended before the number of test cases", deco)
            tests = int(self.tokens[self.tok_i])
            self.tok_i += 1
        done = 0
        while (self.more_input() if tests is None else done < tests):
            done += 1
            env = Env(mod.env)
            try:
                self.block(rest, env)
            except ReturnEx as r:
                self.top_return(r)

    def top_return(self, r):
        if r.value is not None:
            raise CpError(f"{os.path.relpath(self.mod.path)}: a top-level `return` takes no value: it ends the "
                          "program, or the current test case below `@!multitest`")

    def more_input(self):
        while self.tok_i >= len(self.tokens) and self.stdin_stream is not None:
            line = self.stdin_stream.readline()
            if not line:
                break
            self.tokens.extend(line.split())
        return self.tok_i < len(self.tokens)

    def declare(self, s, mod):
        self.ex(s, mod.env)
        name = getattr(s, "name", None)
        if s.kind == "let" and s.pat.kind == "pname":
            name = s.pat.name
        if name and s.kind in ("fn", "struct", "gconst", "let", "typedef", "domain", "trait"):
            if getattr(s, "pub", False):
                mod.pub.add(name)
            mod.decls.append(s)
        if s.kind == "fn" and any(d.name == "test" for d in s.decos):
            mod.tests.append(s)

    def resolve(self, path, node):
        """`use std.segtree` -> stdlib/segtree.cp; `use lib.x` -> the library named lib."""
        root, rest = path[0], path[1:]
        if root == "std":
            base = os.path.join(ROOT, "stdlib")
        else:
            base = None
            d = os.path.dirname(self.mod.path)
            while True:
                for cfg in ("lib.toml", "cp.toml"):
                    p = os.path.join(d, cfg)
                    if os.path.exists(p):
                        text = open(p).read()
                        m = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M)
                        if cfg == "lib.toml" and m and m.group(1) == root:
                            base = d
                        m = re.search(r'^%s\s*=\s*"([^"]+)"' % re.escape(root), text, re.M)
                        if base is None and m:
                            base = os.path.normpath(os.path.join(d, m.group(1)))
                if base or os.path.dirname(d) == d:
                    break
                d = os.path.dirname(d)
            if base is None:
                self.fail(f"no library named `{root}`: add it under [libs] in cp.toml, or name it in the library's lib.toml", node)
        f = os.path.join(base, *rest) + ".cp"
        if not os.path.exists(f):
            self.fail(f"no module `{'.'.join(path)}` (looked for {os.path.relpath(f)})", node)
        return f

    def x_use(self, s, env):
        m = self.load(self.resolve(s.path, s), s.path[-1])
        if s.names is not None or s.glob:
            for nm in (sorted(m.pub) if s.glob else s.names):
                if nm not in m.pub:
                    self.fail(f"`{nm}` is not `pub` in {'.'.join(s.path)}" if nm in m.env.vars
                              else f"{'.'.join(s.path)} has no `{nm}`", s)
                env.vars[nm] = m.env.vars[nm]
        else:
            env.vars[s.alias or s.path[-1]] = m

    # ---- statements
    def block(self, body, env):
        v = None
        ex = self.ex
        for s in body:
            v = ex(s, env)
        return v

    def ex(self, s, env):
        self.line = s.line
        m = self.methods.get(s.kind)
        if m is None:
            m = getattr(self, "x_" + s.kind, None)
            if m is None:
                self.fail(f"`{s.kind}` statements are not supported by the reference interpreter", s)
            self.methods[s.kind] = m
        return m(s, env)

    def x_exprstmt(self, s, env):
        if s.expr.kind in ("call", "paren") and self.is_io(s.expr, env):
            ret = env.declared_type("$ret")     # `fn next() -> 0..=9 = read()`: the tail value is the return type
            if ret is not None and ret[0].kind != "tyields" and ret[2] is s:
                return self.io(s.expr, ret[0], ret[1], env)
        return self.ev(s.expr, env)

    def x_let(self, s, env):
        if s.pat.kind == "ptuple" and s.ty is not None and s.ty.kind == "ttuple" and s.value is not None \
                and self.is_io(s.value, env) and len(s.ty.fields) == len(s.pat.items) and not getattr(s, "orelse", None):
            # `let n, m: T = read()` / `let (n, m): (1..=10, 1..=n) = read()`: read left to right
            for k, (pat, (_, ft)) in enumerate(zip(s.pat.items, s.ty.fields)):
                last = k == len(s.pat.items) - 1
                self.x_let(N("let", s.line, pat=pat, ty=ft, value=s.value, mut=s.mut, const=False,
                             where=getattr(s, "where", None) if last else None, orelse=None, more=[]), env)
            return
        if s.value is None:
            v = self.default(self.ty(s.ty, env), s) if s.ty is not None else None
        elif s.ty is not None and s.ty.kind == "tarray" and s.value.kind not in ("table", "call", "name", "field", "index"):
            v = self.table_fill(s.ty, s.value, env)
        elif s.ty is not None and self.is_io(s.value, env):
            v = self.io(s.value, s.ty, env, env)
        else:
            v = self.ev(s.value, env)
            if s.value.kind in ("name", "field", "index"):
                v = self.own(v)
            if s.ty is not None and s.ty.kind == "tname" and s.value.kind == "list" and not s.value.items:
                # `var q: Deque<T> = []`: the empty value of a library container
                d = env.get(s.ty.name) if env.has(s.ty.name) else None
                if type(d) is StructDecl and "new" in d.methods:
                    v = self.construct(self.ty(s.ty, env), [], None, s, None)
        if getattr(s, "orelse", None) and v is None:
            self.block(s.orelse, env)
            self.fail("the `else` of a `let` must leave (return, break or continue)", s)
        self.bind(s.pat, v, env)
        if s.pat.kind == "pname":
            if s.ty is not None:
                env.declare_type(s.pat.name, s.ty, env)
            elif env.types is not None:
                env.types.pop(s.pat.name, None)
        if getattr(s, "where", None) is not None and not self.ev(s.where, env):
            self.fail(f"`where` does not hold: {self.src(s)}", s)

    def table_fill(self, ty, value, env):
        dims = tuple(self.dim_ty(d, env) for d in ty.dims)
        n = 1
        for d in dims:
            n *= self.dim_k(d, ty)
        if value.kind == "list" and value.items and len(dims) == 1 and ty.elem.kind != "tlist" and not (
                ty.elem.kind == "tname" and ty.elem.name == "List"):
            # `let a: [D]Int = [2, 3, 5]`: one value per index, in order (not the whole list in every cell)
            if len(value.items) != n:
                self.fail(f"an array literal for this type needs {n} values, got {len(value.items)}", value)
            return CArray(dims, [self.ev(x, env) for x in value.items])
        return CArray(dims, [self.ev(value, env) for _ in range(n)])

    def dim_k(self, d, node=None):
        if isinstance(d, Domain):
            return d.K
        if isinstance(d, StructType) and "K" in d.decl.assoc:
            return self.assoc(d, "K")
        self.fail(f"`{d!r}` is not an index type (a domain, or a struct with `impl Index`)", node)

    def bind(self, pat, v, env):
        if pat.kind == "pname":
            env.vars[pat.name] = v
        else:
            if type(v) is not Tup or len(v.items) != len(pat.items):
                self.fail(f"cannot destructure {self.show(v)} into {len(pat.items)} names")
            for p, x in zip(pat.items, v.items):
                self.bind(p, x, env)

    def x_assign(self, s, env):
        t = s.target
        if t.kind == "tuple":
            if self.is_io(s.value, env):
                # `(a[d], b[d]) = read()`: each place's declared type says what to read, left to right
                self.ask(s.value, env)
                v = Tup([self.read_into(tt, self.place(tt, env)[0], None, env) for tt in t.items])
            else:
                v = self.ev(s.value, env)
            if type(v) is not Tup or len(v.items) != len(t.items):
                self.fail("tuple assignment needs a tuple of the same size", s)
            for tt, x in zip(t.items, list(v.items)):
                self.place(tt, env)[1](x)
            return
        op = s.op
        fold = getattr(s, "fold", None)
        if t.kind == "index" and (op != "=" or fold):
            # `m[k] f= x` on a key that is not there yet starts from the identity of f; `min=` / `max=` take x
            o = self.ev(t.obj, env)
            key = self.ev(t.idx[0], env) if type(o) is CMap else None
            if type(o) is CMap and key not in o.items:
                f = fold or op[:-1]
                if f in ("min", "max"):
                    o.items[key] = self.ev(s.value, env)
                    return
                if f not in FOLD_IDENTITY:
                    self.fail(f"`{self.src(t)} {f}= ...`: the key is not in the map, and `{f}` has no identity to start from", s)
                o.items[key] = FOLD_IDENTITY[f]
        get, put = self.place(t, env)
        if op == "=" and self.is_io(s.value, env):
            v = self.read_into(t, get, s.value, env)
        else:
            v = self.ev(s.value, env)
        if op != "=":
            cur = get()
            if op in ("min=", "max="):
                # `none` on either side is skipped
                if v is None or (cur is not None and ((cur < v) if op == "min=" else (v < cur))):
                    v = cur
            else:
                v = self.binop(op[:-1], cur, v, s)
        elif s.value.kind in ("name", "field", "index"):
            v = self.own(v)
        put(v)

    def own(self, v):
        """A binding or assignment from a place copies: `var t = s` never aliases `s` (value semantics, X4)."""
        if type(v) in (CList, CArray, CMap, CSet):
            from .builtins import copyv
            return copyv(v)
        if type(v) is Tup:
            return Tup([self.own(x) for x in v.items], v.names)
        return v

    def place(self, t, env):
        """(get, put) for an assignable expression."""
        k = t.kind
        if k == "name":
            nm = t.name
            def put(v):
                if not env.set(nm, v):
                    self.fail(f"unknown name `{nm}`", t)
            return (lambda: self.e_name(t, env)), put
        if k == "field":
            nm = t.name
            if t.obj.kind == "index":
                oget, _ = self.place(t.obj, env)
                def fget():
                    o = oget()
                    if type(o) is Obj:
                        if nm not in o.f:
                            self.fail(f"{o.stype!r} has no field `{nm}`", t)
                        return o.f[nm]
                    if type(o) is Tup:
                        return o.items[self.tup_index(o, nm, t)]
                    self.fail(f"cannot assign to `.{nm}` of {self.show(o)}", t)
                def fput(v):
                    o = oget()
                    if type(o) is Obj:
                        if nm not in o.f:
                            self.fail(f"{o.stype!r} has no field `{nm}`", t)
                        o.f[nm] = v
                        return
                    if type(o) is Tup:
                        o.items[self.tup_index(o, nm, t)] = v
                        return
                    self.fail(f"cannot assign to `.{nm}` of {self.show(o)}", t)
                return fget, fput
            o = self.ev(t.obj, env)
            if type(o) is Obj:
                if nm not in o.f:
                    self.fail(f"{o.stype!r} has no field `{nm}`", t)
                return (lambda: o.f[nm]), (lambda v: o.f.__setitem__(nm, v))
            if type(o) is Tup:
                i = self.tup_index(o, nm, t)
                return (lambda: o.items[i]), (lambda v: o.items.__setitem__(i, v))
            self.fail(f"cannot assign to `.{nm}` of {self.show(o)}", t)
        if k == "index":
            o = self.ev(t.obj, env)
            idx = [self.ev(i, env) for i in t.idx]
            if type(o) is CMap:
                key = idx[0]
                return (lambda: o.items[key]), (lambda v: o.items.__setitem__(key, v))
            return (lambda: o.items[self.offset(o, idx, t)]), (lambda v: o.items.__setitem__(self.offset(o, idx, t), v))
        if k == "call":
            r = self.e_call(t, env, raw=True)
            if type(r) is not Lent:
                self.fail("only a `lends` projection can be assigned through", t)
            return self.place(r.node, r.env)
        if k == "paren":
            return self.place(t.a, env)
        if k == "unwrap":
            get, put = self.place(t.a, env)
            return (lambda: self.unwrap(get(), t)), put
        self.fail("this expression cannot be assigned to", t)

    def x_if(self, s, env):
        if self.ev(s.cond, env):
            return self.block(s.body, env)
        return self.block(s.orelse, env)

    def x_iflet(self, s, env):
        v = self.ev(s.value, env)
        if v is not None:
            self.bind(s.pat, v, env)
            if s.extra is None or self.ev(s.extra, env):
                return self.block(s.body, env)
        return self.block(s.orelse, env)

    def x_while(self, s, env):
        try:
            while self.ev(s.cond, env):
                try:
                    self.block(s.body, env)
                except ContinueEx:
                    pass
        except BreakEx:
            pass

    def x_whilelet(self, s, env):
        try:
            while True:
                v = self.ev(s.value, env)
                if v is None:
                    break
                self.bind(s.pat, v, env)
                try:
                    self.block(s.body, env)
                except ContinueEx:
                    pass
        except BreakEx:
            pass

    def x_for(self, s, env):
        it = self.ev(s.iter, env)
        pat, body = s.pat, s.body
        simple = pat.name if pat.kind == "pname" else None

        def step(x):
            e = Env(env)
            if simple:
                e.vars[simple] = x
            else:
                self.bind(pat, x, e)
            try:
                self.block(body, e)
            except ContinueEx:
                pass
        try:
            self.iterate(it, step, s)
        except BreakEx:
            pass

    def iterate(self, it, step, node=None):
        t = type(it)
        if t is GenVal:
            token = object()

            def emit(x):
                try:
                    step(x)
                except (BreakEx, ReturnEx) as ex:
                    raise ConsumerExit(token, ex)
            try:
                it.run(emit)
            except ConsumerExit as ce:
                if ce.token is token:
                    raise ce.inner
                raise
            return
        for x in self.values(it, node):
            step(x)

    def values(self, it, node=None):
        t = type(it)
        if t is CList or t is CArray:
            return list(it.items) if t is CList else it.items
        if t is RangeVal:
            return it.values()
        if t is Domain:
            return (DomVal(it, i) for i in range(it.K))
        if t is CSet:
            return sorted(it.items)
        if t is CMap:
            return [Tup([k, v]) for k, v in sorted(it.items.items())]
        if t is GenVal:
            out = []
            self.iterate(it, out.append, node)
            return out
        self.fail(f"cannot iterate over {self.show(it)}", node)

    def x_return(self, s, env):
        if s.value is not None and self.is_io(s.value, env):
            ret = env.declared_type("$ret")
            if ret is not None:
                raise ReturnEx(self.io(s.value, ret[0], ret[1], env))
        raise ReturnEx(self.ev(s.value, env) if s.value is not None else None)

    # ---- read() / ask(): the type comes from where the value is going (a typed let, a typed place, the
    # fn's return type, a parameter), or from `read::<T>()` when nothing around it says
    def is_io(self, e, env):
        if e.kind == "paren":
            return self.is_io(e.a, env)
        return e.kind == "call" and e.fn.kind == "name" and e.fn.name in ("read", "ask") and not env.has(e.fn.name) \
            and not (e.fn.name == "read" and e.args)

    def ask(self, e, env):
        """`ask("?", i, j)` prints the question, flushes, and the reply is read next."""
        while e.kind == "paren":
            e = e.a
        if e.fn.name != "ask":
            return
        from .builtins import f_print
        f_print(self, [self.ev(a, env) for a in e.args if a.kind != "kwarg"], None, e)

    def io(self, e, ty, tenv, env):
        self.ask(e, env)
        return self.read(ty, tenv)

    def read_into(self, t, get, call, env):
        """`place = read()`: read what the place holds. Its declared type says what that is; a `var` with no
        annotation holds the type of its current value (cpi has no static types to ask)."""
        if call is not None:
            self.ask(call, env)
        r = self.place_type_opt(t, env)
        if r is not None:
            return self.read(*r)
        cur = get()
        ty = type(cur)
        if ty is DomVal and cur.dom.of is None:
            tok = self.token(t)
            return self.dom_new(cur.dom, tok[0] if cur.dom.char else int(tok), t)
        if ty is int:
            return int(self.token(t))
        if ty is float:
            return float(self.token(t))
        if ty is str:
            return self.token(t)[0]
        if ty is CList and cur.is_str:
            return CList(list(self.token(t)), True)
        self.place_type(t, env, t)

    def place_type(self, t, env, node):
        """The declared type of an assignable expression, as (type node, env), for `place = read()`."""
        r = self.place_type_opt(t, env)
        if r is None:
            self.fail(f"read() cannot tell the type of `{self.src(t)}` here: declare it (`var x: T`), or say `read::<T>()`", node)
        return r

    def place_type_opt(self, t, env):
        k = t.kind
        if k == "paren":
            return self.place_type_opt(t.a, env)
        if k == "name":
            return env.declared_type(t.name)
        if k == "unwrap":
            r = self.place_type_opt(t.a, env)
            return (r[0].inner, r[1]) if r is not None and r[0].kind == "topt" else None
        if k == "index":
            r = self.place_type_opt(t.obj, env)
            if r is None:
                return None
            ty, tenv = self.alias(*r)
            if ty.kind == "tarray" and len(ty.dims) == len(t.idx):
                return ty.elem, tenv
            if ty.kind == "tarray" and len(t.idx) < len(ty.dims):
                return N("tarray", ty.line, dims=ty.dims[len(t.idx):], elem=ty.elem), tenv
            if ty.kind == "tlist" and len(t.idx) == 1:
                return ty.elem, tenv
            if ty.kind == "tname" and ty.name in ("List", "Deque") and len(ty.args) == 1 and len(t.idx) == 1:
                return ty.args[0], tenv
            if ty.kind == "tname" and ty.name in ("Map", "HashMap") and len(ty.args) == 2 and len(t.idx) == 1:
                return ty.args[1], tenv
            return None
        if k == "field":
            r = self.place_type_opt(t.obj, env)
            if r is not None:
                ty, tenv = self.alias(*r)
                if ty.kind == "ttuple":
                    for i, (nm, ft) in enumerate(ty.fields):
                        if nm == t.name or str(i) == str(t.name):
                            return ft, tenv
                    return None
            try:
                o = self.ev(t.obj, env)
            except CpError:
                return None
            if type(o) is Obj and t.name in o.stype.decl.ftypes:
                return o.stype.decl.ftypes[t.name], o.stype.env
        return None

    def alias(self, ty, tenv):
        """Follow `type X = ...` names to the type they stand for."""
        while ty.kind == "tname" and not ty.args:
            r = tenv.declared_type("type " + ty.name)
            if r is None:
                break
            ty, tenv = r
        return ty, tenv

    def x_break(self, s, env):
        raise BreakEx()

    def x_continue(self, s, env):
        raise ContinueEx()

    def x_assert(self, s, env):
        if not self.ev(s.cond, env):
            self.fail(f"assert failed: {self.src(s)}{self.explain(s.cond, env)}", s)

    def src(self, s):
        try:
            return self.mod.src[s.line - 1].strip()
        except Exception:
            return ""

    def explain(self, c, env):
        """For `assert a == b`, show both sides."""
        if c.kind == "cmp" and len(c.ops) == 1:
            try:
                return f"\n    left:  {self.show(self.ev(c.first, env))}\n    right: {self.show(self.ev(c.rest[0], env))}"
            except Exception:
                return ""
        return ""

    def x_directive(self, s, env):
        self.fail(f"`@!{s.name}` stands at the top level of the file, on a line of its own", s)

    def x_fn(self, s, env):
        env.vars[s.name] = Closure(s, env, self.mod)

    def x_struct(self, s, env):
        env.vars[s.name] = StructDecl(s, env, self.mod)

    def x_impl(self, s, env):
        decl = env.get(s.target) if env.has(s.target) else None
        if not isinstance(decl, StructDecl):
            self.fail(f"`impl {s.trait} for {s.target}`: {s.target} is not a struct defined here", s)
        for m in s.body:
            if m.kind == "fn":
                if s.trait == "Index" and m.name == "to_idx":
                    m.decos = list(m.decos) + [N("deco", m.line, name="get", args=[])]
                decl.methods[m.name] = m
            elif m.kind == "assoc":
                decl.assoc[m.name] = (m.value, env)

    def assoc(self, st, name):
        if name not in st.consts:
            node, _ = st.decl.assoc[name]
            st.consts[name] = self.ev(node, st.env)
        return st.consts[name]

    def x_trait(self, s, env):
        # a trait method written with a body is the default: any value whose type has no
        # method of that name answers with it (`Reversible.rev` is the identity for plain values)
        env.vars[s.name] = TypeV("trait " + s.name)
        for m in getattr(s, "defaults", ()):
            self.trait_defaults[m.name] = Closure(m, env, self.mod)

    def extension(self, o, name):
        """`p.adj4` for a value with no such member: a fn in scope whose first parameter is `self`
        (an extension method, `fn adj4(self: (R, C))`), else a trait's default method."""
        env = self.mod.env if self.mod is not None else self.globals
        f = env.get(name) if env.has(name) else None
        if not (type(f) is Closure and f.node.kind == "fn" and f.node.params and f.node.params[0].name == "self"):
            f = self.trait_defaults.get(name)
        return f

    def call_extension(self, f, o, args, kwargs, node):
        return self.call_fn(f.node, f.env, f.mod, [o] + list(args), kwargs, node)

    def is_get(self, f):
        return any(d.name == "get" for d in f.node.decos)

    def x_law(self, s, env):
        pass

    def x_gconst(self, s, env):
        env.vars[s.name] = GConst(s, env)

    def x_typedef(self, s, env):
        env.vars[s.name] = self.ty(s.ty, env)
        env.declare_type("type " + s.name, s.ty, env)

    def x_domain(self, s, env):
        r = s.range
        if r.kind != "range" or r.hi is None:
            self.fail("a domain is declared over a range: `domain V = 1..=n`", s)
        lo, hi = self.ev(r.lo, env), self.ev(r.hi, env)
        char = isinstance(lo, str)
        if char:
            hi = ord(hi) if isinstance(hi, str) else hi
            k = hi - ord(lo) + (1 if r.incl else 0)
        else:
            k = hi - lo + (1 if r.incl else 0)
        env.vars[s.name] = Domain(s.name, lo, max(0, k), char)

    def x_yield(self, s, env):
        env.get("$yield")(self.ev(s.value, env))

    def x_yieldfrom(self, s, env):
        self.iterate(self.ev(s.value, env), env.get("$yield"), s)

    def x_lend(self, s, env):
        raise ReturnEx(Lent(s.value, env))

    def dim_ty(self, d, env):
        if d.kind == "texpr":
            k = self.ev(d.expr, env)
            if type(k) is int:
                return Domain(f"0..{k}", 0, max(0, k), anon=True)
            return k
        if d.kind == "trange":
            r = d.range
            lo = self.ev(r.lo, env)
            hi = self.ev(r.hi, env)
            if type(lo) is str and type(hi) is str:
                return Domain(f"{lo!r}..{hi!r}", lo, max(0, ord(hi) - ord(lo) + (1 if r.incl else 0)), char=True, anon=True)
            return Domain(f"{lo}..{hi}", lo, max(0, hi - lo + (1 if r.incl else 0)), anon=True)
        v = self.ty(d, env)
        if type(v) is int:
            return Domain(f"0..{v}", 0, max(0, v), anon=True)
        return v

    # ---- types as values
    def ty(self, t, env):
        k = t.kind
        if k == "tunit":
            return TypeV("Unit")
        if k == "tifconst":
            return self.ty(t.then_ty if self.ev(t.cond, env) else t.else_ty, env)
        if k == "tpath":
            return self.field(self.ty(t.base, env), t.name, t)
        if k == "tname":
            try:
                v = env.get(t.name)
            except KeyError:
                st, ok = self.member_const(t.name, env)
                if not ok:
                    self.fail(f"unknown type `{t.name}`", t)
                v = self.assoc(st, t.name)
            if t.args:
                if isinstance(v, StructDecl):
                    args = []
                    for i, a in enumerate(t.args):
                        tp = v.tparams[i] if i < len(v.tparams) else None
                        if tp is not None and tp in v.const_tparams:
                            args.append(self.ev(a.expr if a.kind == "texpr" else N("name", a.line, name=a.name), env))
                        else:
                            args.append(self.ty(a, env) if a.kind.startswith("t") else self.ev(a, env))
                    for tp in v.tparams[len(args):]:
                        if tp in v.tparam_defaults:
                            dnode = v.tparam_defaults[tp]
                            args.append(self.ev(dnode, v.env) if tp in v.const_tparams else self.ty(dnode, v.env))
                    return v.inst(args)
                args = [self.ty(a, env) if a.kind.startswith("t") else self.ev(a, env) for a in t.args]
                if isinstance(v, TypeV):
                    return TypeV(v.name, args)
                if isinstance(v, GConst):
                    return self.gconst(v, args, t)
                self.fail(f"`{t.name}` takes no type arguments", t)
            if isinstance(v, StructDecl):
                if not v.tparams:
                    return v.inst(())
                if all(tp in v.tparam_defaults for tp in v.tparams):
                    args = [self.ev(v.tparam_defaults[tp], v.env) if tp in v.const_tparams else self.ty(v.tparam_defaults[tp], v.env) for tp in v.tparams]
                    return v.inst(args)
                return v.open
            return v
        if k == "tarray":
            return TypeV("Array", [tuple(self.dim_ty(d, env) for d in t.dims), self.ty(t.elem, env)])
        if k == "tlist":
            return TypeV("List", [self.ty(t.elem, env)])
        if k == "topt":
            return TypeV("Opt", [self.ty(t.inner, env)])
        if k == "ttuple":
            return TypeV("Tuple", [self.ty(f[1], env) for f in t.fields if f[1].kind != "tfresh"])
        if k == "trange":
            return INT
        if k == "texpr":
            return self.ev(t.expr, env)
        if k in ("tfn", "tyields", "tfresh"):
            return TypeV("Fn")
        self.fail(f"unsupported type syntax ({k})", t)

    def default(self, t, node):
        if t == INT:
            return 0
        if t == FLOAT:
            return 0.0
        if t == BOOL:
            return False
        if isinstance(t, TypeV):
            if t.name == "List":
                return CList([])
            if t.name == "Str":
                return CList([], True)
            if t.name == "Opt":
                return None
            if t.name == "Array":
                dims = t.args[0]
                n = 1
                for d in dims:
                    n *= self.dim_k(d, node)
                return CArray(dims, [self.default(t.args[1], node) for _ in range(n)])
            if t.name == "Tuple":
                return Tup([self.default(a, node) for a in t.args])
        self.fail(f"`{t!r}` has no default value; give the variable a value", node)

    def gconst(self, g, args, node):
        key = tuple(id(a) if not isinstance(a, (int, TypeV)) else a for a in args)
        if key not in g.cache:
            if len(args) != len(g.node.tparams):
                self.fail(f"`{g.node.name}` takes {len(g.node.tparams)} type argument(s)", node)
            g.cache[key] = self.ev(g.node.value, Env(g.env, dict(zip(g.node.tparams, args))))
        return g.cache[key]

    def typeof(self, v):
        t = type(v)
        if t is DomVal:
            return v.dom
        if t is bool:
            return BOOL
        if t is int:
            return INT
        if t is float:
            return FLOAT
        if t is str:
            return CHAR
        if t is Obj:
            return v.stype
        return None

    def unify(self, t, v, tps, bind, env, node):
        """Bind the type parameters in `t` from the argument `v`; check domains on the way."""
        k = t.kind
        if k == "tname" and not t.args:
            if t.name in tps:
                if t.name in env.vars:
                    bind[t.name] = env.vars[t.name]
                got = self.typeof(v)
                if got is None:
                    return
                have = bind.get(t.name)
                if have is None:
                    bind[t.name] = got
                elif isinstance(have, Domain) and got is not have:
                    self.fail(f"expected `{t.name}` = {have!r}, got {self.show(v)} ({got!r})", node)
                return
            want = bind.get(t.name)
            if want is None and env.has(t.name):
                want = env.get(t.name)
            if isinstance(want, Domain) and not (type(v) is DomVal and v.dom is want):
                self.fail(f"expected a {want!r}, got {self.show(v)}" + (f" (a {v.dom!r})" if type(v) is DomVal else ""), node)
            return
        if k == "tname":
            if type(v) is Obj and v.stype.targs and v.stype.decl.name == t.name:
                for a, ta in zip(t.args, v.stype.targs):
                    if a.kind == "tname" and not a.args and a.name in tps and a.name not in bind:
                        bind[a.name] = ta
            elif type(v) is CList and v.items and t.name in ("List", "Iter") and t.args[0].kind.startswith("t"):
                self.unify(t.args[0], v.items[0], tps, bind, env, node)
            return
        if k == "tarray" and type(v) is CArray:
            for d, vd in zip(t.dims, v.dims):
                if d.kind == "tname" and not d.args:
                    if d.name in tps and d.name not in bind:
                        bind[d.name] = vd
                    else:
                        want = bind.get(d.name) or (env.get(d.name) if env.has(d.name) else None)
                        if isinstance(want, Domain) and want is not vd:
                            self.fail(f"expected an array indexed by {want!r}, got one indexed by {vd!r}", node)
            if v.items:
                self.unify(t.elem, v.items[0], tps, bind, env, node)
        elif k == "tlist" and type(v) is CList and v.items:
            self.unify(t.elem, v.items[0], tps, bind, env, node)
        elif k == "topt" and v is not None:
            self.unify(t.inner, v, tps, bind, env, node)
        elif k == "ttuple" and type(v) is Tup:
            for (_, ft), x in zip(t.fields, v.items):
                self.unify(ft, x, tps, bind, env, node)

    # ---- calls
    def call(self, f, args, kwargs=None, node=None):
        t = type(f)
        if t is Closure:
            return self.call_fn(f.node, f.env, f.mod, args, kwargs, node)
        if t is Bound:
            return self.call_method_node(f.obj, f.node, args, kwargs, node)
        if t is StructType or t is StructDecl:
            return self.construct(f if t is StructType else f.open, args, kwargs, node, None)
        if t is Domain:
            return self.dom_new(f, args[0], node)
        if t is TypeV:
            return self.type_call(self, f, args, kwargs or {}, node)
        if callable(f):
            return f(self, args, kwargs or {}, node)
        self.fail(f"{self.show(f)} is not a fn", node)

    def call_fn(self, fn, env, mod, args, kwargs, node, self_obj=None, raw=False):
        frame = Env(env)
        if fn.kind == "lambda":
            ps = fn.params
            if len(ps) != len(args):
                if len(ps) > 1 and len(args) == 1 and type(args[0]) is Tup and len(args[0].items) == len(ps):
                    args = args[0].items
                else:
                    self.fail(f"this lambda takes {len(ps)} argument(s), got {len(args)}", node)
            for p, a in zip(ps, args):
                self.bind(p, a, frame)
            saved = self.mod, self.line
            self.mod = mod
            try:
                return self.ev(fn.body, frame)
            finally:
                self.mod, self.line = saved
        params = fn.params
        if self_obj is not None:
            frame.vars["self"] = self_obj
            params = params[1:]
        if len(args) > len(params):
            self.fail(f"`{fn.name}` takes {len(params)} argument(s), got {len(args)}", node)
        vals = frame.vars
        tps = fn.tparams
        bind = {}
        for i, p in enumerate(params):
            if i < len(args):
                a = args[i]
            elif kwargs and p.name in kwargs:
                a = kwargs[p.name]
            elif getattr(p, "default", None) is not None:
                a = self.ev(p.default, frame)
            else:
                self.fail(f"`{fn.name}` is missing its argument `{p.name}`", node)
            if p.ty is not None:
                self.unify(p.ty, a, tps, bind, env, node)
                frame.declare_type(p.name, p.ty, frame)
            vals[p.name] = a
        if kwargs:
            names = {p.name for p in params}
            for kname in kwargs:
                if kname not in names:
                    self.fail(f"`{fn.name}` has no parameter `{kname}`", node)
        if tps:
            # a type parameter no argument's annotation fixed is read off a struct argument that
            # has one of the same name: `scc(g)` learns `V` from a `Graph<V, E>`. This stands in
            # for inference through a trait (`N: Neighbours<V>`), which needs static types.
            for tp in tps:
                if tp in bind or env.has(tp):
                    continue
                for a in args:
                    if type(a) is Obj and a.stype.targs and tp in a.stype.decl.tparams:
                        got = a.stype.targs[a.stype.decl.tparams.index(tp)]
                        if got is not None:
                            bind[tp] = got
                            break
        if bind:
            for kname, v in bind.items():
                vals.setdefault(kname, v)
        if is_gen(fn):
            def run(emit):
                frame.vars["$yield"] = emit
                self.run_body(fn, frame, mod)
            return GenVal(run)
        if fn.ret is not None:
            if frame.types is None:
                frame.types = {}
            frame.types["$ret"] = (fn.ret, frame, fn.body[-1] if fn.body else None)
        r = self.run_body(fn, frame, mod)
        if type(r) is Lent and not raw:
            if fn.lends is None:
                self.fail("`lend` is only allowed in a fn declared with `lends`", fn)
            saved = self.mod
            self.mod = mod
            try:
                return self.ev(r.node, r.env)
            finally:
                self.mod = saved
        self.last_call = (fn, frame)
        return r

    def run_body(self, fn, frame, mod):
        saved = self.mod, self.line
        self.mod = mod
        self.depth += 1
        if self.depth > 200:
            self.depth = 0
            self.fail(f"recursion deeper than 200 calls in `{fn.name}` (library code must not recurse on input size)", fn)
        try:
            return self.block(fn.body, frame)
        except ReturnEx as r:
            return r.value
        finally:
            self.depth -= 1
            self.mod, self.line = saved

    def call_method_node(self, obj, m, args, kwargs, node=None, raw=False):
        st = obj.stype
        return self.call_fn(m, Env(st.env, obj.f), st.decl.mod, args, kwargs, node, self_obj=obj, raw=raw)

    def call_static(self, st, m, args, kwargs=None, node=None):
        return self.call_fn(m, st.env, st.decl.mod, args, kwargs, node)

    def construct(self, st, args, kwargs, node, env):
        decl = st.decl
        inside = env is not None and env.has("$decl") and env.get("$decl") is decl
        if st.targs is None and inside:
            st = env.get("Self")
        new = decl.methods.get("new")
        if new is not None and not inside:
            if st.targs is None and decl.tparams:
                st = self.infer(st, [(p.ty, p.name) for p in new.params], args, kwargs, node)
            return self.call_static(st, new, args, kwargs, node)
        names = decl.fields
        if len(args) > len(names):
            self.fail(f"`{decl.name}` has {len(names)} field(s), got {len(args)} values", node)
        f = {}
        erased = set()
        if st.targs is not None:
            for nm in names:
                ft = decl.ftypes[nm]
                if ft.kind in ("tifconst", "tunit") and self.ty(ft, st.env) == TypeV("Unit"):
                    erased.add(nm)
        for i, nm in enumerate(names):
            if i < len(args):
                f[nm] = args[i]
            elif kwargs and nm in kwargs:
                f[nm] = kwargs[nm]
            elif nm in decl.defaults:
                f[nm] = self.ev(decl.defaults[nm], st.env if st.targs is not None else decl.env)
            elif nm in erased:
                continue
            else:
                self.fail(f"`{decl.name}` is missing its field `{nm}`", node)
        if st.targs is None and decl.tparams:
            st = self.infer(st, [(decl.ftypes[nm], nm) for nm in names if nm in f], args, kwargs, node)
        return Obj(st, f)

    def infer(self, st, params, args, kwargs, node):
        decl = st.decl
        bind = {}
        for i, (ty, nm) in enumerate(params):
            a = args[i] if i < len(args) else (kwargs or {}).get(nm)
            if ty is not None and a is not None:
                self.unify(ty, a, decl.tparams, bind, decl.env, node)
        return decl.inst([bind.get(tp) for tp in decl.tparams])

    def dom_new(self, d, x, node):
        if d.of is not None:
            self.fail(f"a {d!r} is built from a position: `v.l`, `v.r`", node)
        i = (ord(x) - ord(d.lo)) if d.char and isinstance(x, str) else (x - d.lo if type(x) is int else None)
        if i is None or not 0 <= i < d.K:
            hi = d.lo + d.K - 1 if not d.char else chr(ord(d.lo) + d.K - 1)
            self.fail(f"{self.show(x)} is not in domain {d.name} ({self.show(d.lo)}..={self.show(hi)})", node)
        return DomVal(d, i)

    def token(self, node):
        while self.tok_i >= len(self.tokens) and self.stdin_stream is not None:
            line = self.stdin_stream.readline()
            if not line:
                break
            self.tokens.extend(line.split())
        if self.tok_i >= len(self.tokens):
            self.fail("read(): the input ended", node)
        self.tok_i += 1
        return self.tokens[self.tok_i - 1]

    def read(self, t, env):
        k = t.kind
        if k == "tarray":
            dims = tuple(self.dim_ty(d, env) for d in t.dims)
            n = 1
            for d in dims:
                n *= self.dim_k(d, t)
            return CArray(dims, [self.read(t.elem, env) for _ in range(n)])
        if k == "ttuple":
            return Tup([self.read(ft, env) for _, ft in t.fields], [nm for nm, _ in t.fields])
        tok = self.token(t)
        if k == "trange":
            x = int(tok)
            if x not in self.ev(t.range, env).values():
                self.fail(f"read(): {x} is outside {self.src(t)}", t)
            return x
        tv = self.ty(t, env)
        if tv == INT:
            return int(tok)
        if tv == FLOAT:
            return float(tok)
        if tv == STR:
            return CList(list(tok), True)
        if tv == CHAR:
            return tok[0]
        if isinstance(tv, Domain):
            return self.dom_new(tv, tok[0] if tv.char else int(tok), t)
        self.fail(f"read() cannot read a {tv!r}", t)

    def show(self, v):
        t = type(v)
        if t is bool:
            return "true" if v else "false"
        if v is None:
            return "none"
        if t is str:
            return v
        if t is CList:
            if v.is_str:
                return "".join(v.items)
            return "[" + ", ".join(self.show(x) for x in v.items) + "]"
        if t is CArray:
            return "[" + ", ".join(self.show(x) for x in v.items) + "]"
        if t is DomVal:
            return self.show(v.label) if v.dom.of is None else f"{v.dom.name}#{v.ord}"
        if t is Tup:
            if v.names and any(v.names):
                return "(" + ", ".join(f"{n}: {self.show(x)}" if n else self.show(x) for n, x in zip(v.names, v.items)) + ")"
            return "(" + ", ".join(self.show(x) for x in v.items) + ")"
        if t is float:
            return repr(v)
        return repr(v)

    # ---- expressions
    def ev(self, e, env):
        m = self.methods.get(e.kind)
        if m is None:
            m = getattr(self, "e_" + e.kind, None)
            if m is None:
                self.fail(f"`{e.kind}` expressions are not supported by the reference interpreter", e)
            self.methods[e.kind] = m
        return m(e, env)

    def e_num(self, e, env):
        return e.value

    def e_bool(self, e, env):
        return e.value

    def e_none(self, e, env):
        return None

    def e_char(self, e, env):
        return e.value

    def e_keepassign(self, e, env):
        """`x min= y` / `x max= y` as a Bool: true when the place changed. `none` on either side is skipped."""
        t = e.target
        if t.kind == "index":
            o = self.ev(t.obj, env)
            if type(o) is CMap:
                key = self.ev(t.idx[0], env)
                if key not in o.items:
                    o.items[key] = self.ev(e.value, env)
                    return True
        get, put = self.place(t, env)
        v = self.ev(e.value, env)
        cur = get()
        if v is None or (cur is not None and not ((v < cur) if e.op == "min=" else (cur < v))):
            return False
        put(v)
        return True

    def e_paren(self, e, env):
        return self.ev(e.a, env)

    def e_str(self, e, env):
        parts = getattr(e, "_parts", None)
        if parts is None:
            parts, buf, s, i = [], "", e.value, 0
            while i < len(s):
                c = s[i]
                if c == "\\" and i + 1 < len(s):
                    buf += ESC.get(s[i + 1], s[i + 1])
                    i += 2
                elif c == "{" and "}" in s[i:]:
                    j = s.index("}", i)
                    parts.append(buf)
                    body, _, spec = s[i + 1:j].partition(":")
                    parts.append((LibParser(body).expr(), spec))
                    buf, i = "", j + 1
                else:
                    buf += c
                    i += 1
            parts.append(buf)
            e._parts = parts
        if len(parts) == 1:
            return CList(list(parts[0]), True)
        return CList(list("".join(p if isinstance(p, str) else self.fmt(self.ev(p[0], env), p[1], e) for p in parts)), True)

    def fmt(self, v, spec, node):
        """`{x}` shows a value; `{x:.3f}` and `{n:04}` format a number (never `-0.000`)."""
        if not spec:
            return self.show(v)
        # the whole grammar: `{v:W}` width, `{v:0W}` zero-filled, `{x:.D}` / `{x:.Df}` fixed decimals,
        # `{n:x}` hex and `{n:b}` binary (each after an optional width). No alignment, no nested widths.
        m = re.fullmatch(r"(0?)(\d*)(?:\.(\d+)f?|([xb]))?", spec)
        help = "use `:.6f` for an F64, `:04` / `:x` / `:b` for an Int"
        if not m or isinstance(v, bool) or not isinstance(v, (int, float)):
            self.fail(f"`:{spec}` is not a number format this value takes ({help})", node)
        zero, width = m.group(1), int(m.group(2) or 0)
        if m.group(3) is not None:
            out = format(float(v), f".{m.group(3)}f")
            if out.startswith("-") and float(out) == 0:
                out = out[1:]
        elif isinstance(v, float):
            self.fail(f"`:{spec}` does not say how many decimals an F64 gets ({help})", node)
        elif m.group(4):
            out = ("-" if v < 0 else "") + format(abs(v), m.group(4))
        else:
            out = str(v)
        if len(out) < width:
            if zero:
                sign = "-" if out.startswith("-") else ""
                out = sign + out[len(sign):].rjust(width - len(sign), "0")
            else:
                out = out.rjust(width)
        return out

    def member_const(self, name, env):
        """An associated const or domain of the enclosing struct, by its bare name."""
        if env.has("Self"):
            st = env.get("Self")
            if name in st.decl.assoc:
                return st, True
        return None, False

    def e_domexpr(self, e, env):
        tmp = Env(env)
        self.x_domain(e, tmp)
        d = tmp.vars[e.name]
        if env.has("Self"):
            d.name = f"{env.get('Self')!r}.{e.name}"
        return d

    def e_name(self, e, env):
        try:
            v = env.get(e.name)
        except KeyError:
            st, ok = self.member_const(e.name, env)
            if ok:
                return self.assoc(st, e.name)
            self.fail(f"unknown name `{e.name}`", e)
        if type(v) is StructDecl:
            return v.inst(()) if not v.tparams else v.open
        return v

    def e_tapp(self, e, env):
        if e.obj.kind == "name":
            return self.e_generic(N("generic", e.line, name=e.obj.name, args=e.targs), env)
        return self.ev(e.obj, env)

    def e_generic(self, e, env):
        try:
            v = env.get(e.name)
        except KeyError:
            self.fail(f"unknown name `{e.name}`", e)
        args = [self.ty(a, env) if a.kind in TYPE_KINDS else self.ev(a, env) for a in e.args]
        if type(v) is StructDecl:
            if len(args) > len(v.tparams):
                self.fail(f"`{v.name}` takes {len(v.tparams)} type argument(s), got {len(args)}", e)
            # Fill defaulted type/const parameters (Synthesis #3 Leap 4); trailing non-defaulted ones stay None for inference
            for tp in v.tparams[len(args):]:
                if tp in v.tparam_defaults:
                    dn = v.tparam_defaults[tp]
                    args.append(self.ev(dn, v.env) if tp in v.const_tparams else self.ty(dn, v.env))
                else:
                    args.append(None)
            return v.inst(args)
        if type(v) is GConst:
            return self.gconst(v, args, e)
        if type(v) is TypeV:
            return TypeV(v.name, args)
        if type(v) is Closure and getattr(v.node, "tparams", None):
            if len(args) > len(v.node.tparams):
                self.fail(f"`{e.name}` takes {len(v.node.tparams)} type argument(s), got {len(args)}", e)
            return Closure(v.node, Env(v.env, dict(zip(v.node.tparams, args))), v.mod, v.stype)
        self.fail(f"`{e.name}` takes no type arguments", e)

    def e_tuple(self, e, env):
        return Tup([self.ev(x, env) for x in e.items], e.names if any(e.names) else None)

    def e_list(self, e, env):
        out = []
        for x in e.items:
            if x.kind == "gen":
                out.extend(self.e_gen(x, env).items)
            else:
                out.append(self.ev(x, env))
        return CList(out)

    def e_gen(self, e, env):
        out = []

        def go(i, env):
            if i == len(e.clauses):
                out.append(self.ev(e.elem, env))
                return
            c = e.clauses[i]
            if c[0] == "for":
                def step(x):
                    inner = Env(env)
                    self.bind(c[1], x, inner)
                    go(i + 1, inner)
                self.iterate(self.ev(c[2], env), step, e)
            elif c[0] == "if":
                if self.ev(c[1], env):
                    go(i + 1, env)
            else:
                v = self.ev(c[2], env)
                if v is not None:
                    inner = Env(env)
                    self.bind(c[1], v, inner)
                    go(i + 1, inner)
        go(0, env)
        return CList(out)

    def e_table(self, e, env):
        dims = tuple(self.ev(d, env) for d in e.dims)
        cells = [[]]
        for d in dims:
            cells = [c + [x] for c in cells for x in self.index_values(d, e)]
        items = []
        for c in cells:
            inner = Env(env)
            self.bind(e.pat, c[0] if len(c) == 1 else Tup(c), inner)
            items.append(self.ev(e.body, inner))
        return CArray(dims, items)

    def index_values(self, d, node):
        if type(d) is Domain:
            return [DomVal(d, i) for i in range(d.K)]
        if type(d) is StructType and "from_idx" in d.decl.methods:
            return [self.call_static(d, d.decl.methods["from_idx"], [i]) for i in range(self.dim_k(d, node))]
        self.fail(f"{self.show(d)} is not an index type", node)

    def e_lambda(self, e, env):
        return Closure(e, env, self.mod)

    def e_ifexpr(self, e, env):
        return self.ev(e.a, env) if self.ev(e.cond, env) else self.ev(e.b, env)

    def e_doblock(self, e, env):
        return self.block(e.body, env)

    def e_unwrap(self, e, env):
        return self.unwrap(self.ev(e.a, env), e)

    def unwrap(self, v, e):
        if v is None:
            self.fail("unwrapped `none` with `!`", e)
        return v

    def e_range(self, e, env):
        hi = self.ev(e.hi, env) if e.hi is not None else None
        if e.lo is None:
            # `..i` and `..=i`: from the start of whatever `i` indexes
            if type(hi) is int:
                return RangeVal(None, 0, hi + 1 if e.incl else hi)
            if type(hi) is DomVal:
                d = hi.dom
                if d.of is not None:
                    if e.incl:
                        self.fail("a range up to a boundary is written `..b`", e)
                    return RangeVal(d.of, 0, hi.ord)
                return RangeVal(d, 0, hi.ord + 1 if e.incl else hi.ord)
            self.fail(f"cannot build a range up to {self.show(hi)}", e)
        lo = self.ev(e.lo, env)
        if type(lo) is int and type(hi) is int:
            return RangeVal(None, lo, hi + 1 if e.incl else hi)
        if type(lo) is DomVal:
            d = lo.dom
            if d.of is not None:
                if hi is None:
                    return RangeVal(d.of, lo.ord, d.of.K)
                if type(hi) is DomVal and hi.dom is d and not e.incl:
                    return RangeVal(d.of, lo.ord, hi.ord)
                self.fail("a range between two boundaries is written `a..b`", e)
            if hi is None:
                return RangeVal(d, lo.ord, d.K)
            if type(hi) is DomVal and hi.dom is d:
                return RangeVal(d, lo.ord, hi.ord + 1 if e.incl else hi.ord)
        self.fail(f"cannot build a range from {self.show(lo)} to {self.show(hi)}", e)

    def e_unop(self, e, env):
        a = self.ev(e.a, env)
        if e.op == "not":
            return not a
        t = type(a)
        if t is int:
            return self.int(-a, e)
        if t is float:
            return -a
        if t is Obj:
            m = a.stype.decl.methods.get("neg")
            if m is not None:
                return self.call_static(a.stype, m, [a], None, e)
        self.fail(f"cannot negate {self.show(a)}", e)

    def int(self, r, node):
        if r < I64_MIN or r > I64_MAX:
            self.fail(f"Int overflow: {r} does not fit in 64 bits", node)
        return r

    def e_binop(self, e, env):
        op = e.op
        if op == "and":
            return bool(self.ev(e.a, env)) and bool(self.ev(e.b, env))
        if op == "or":
            return bool(self.ev(e.a, env)) or bool(self.ev(e.b, env))
        a = self.ev(e.a, env)
        if op == "??":
            return a if a is not None else self.ev(e.b, env)
        return self.binop(op, a, self.ev(e.b, env), e)

    def binop(self, op, a, b, node):
        ta, tb = type(a), type(b)
        if ta is int and tb is int:
            if op == "+":
                r = a + b
            elif op == "-":
                r = a - b
            elif op == "*":
                r = a * b
            elif op == "/" or op == "%":
                # Euclidean: the remainder is never negative (decisions 3.9)
                if b == 0:
                    self.fail("division by zero", node)
                m = a % abs(b)
                r = m if op == "%" else (a - m) // b
            elif op == "&":
                return a & b
            elif op == "|":
                return a | b
            elif op == "^":
                return a ^ b
            elif op == "<<":
                if not 0 <= b < 64:
                    self.fail(f"shift by {b}", node)
                r = a << b
            elif op == ">>":
                if not 0 <= b < 64:
                    self.fail(f"shift by {b}", node)
                return a >> b
            elif op == "**":
                if b < 0:
                    self.fail("negative exponent", node)
                r = a ** b
            else:
                self.fail(f"no `{op}` for Int", node)
            if r < I64_MIN or r > I64_MAX:
                self.fail(f"Int overflow: {a} {op} {b} does not fit in 64 bits", node)
            return r
        if ta is Obj:
            m = a.stype.decl.methods.get(OPS.get(op, ""))
            if m is not None:
                if tb is not Obj or b.stype.decl is not a.stype.decl:
                    self.fail(f"`{op}` needs two {a.stype!r} values, got {self.show(b)}", node)
                return self.call_static(a.stype, m, [a, b], None, node)
            self.fail(f"{a.stype!r} has no `{op}` (no impl of {OPS.get(op, op).capitalize()})", node)
        if ta in (int, float) and tb in (int, float):
            a, b = float(a), float(b)
            if op == "+":
                return a + b
            if op == "-":
                return a - b
            if op == "*":
                return a * b
            if op == "/":
                if b == 0:
                    self.fail("division by zero", node)
                return a / b
            if op == "**":
                return a ** b
        if ta is bool and tb is bool and op in "&|^":
            return {"&": a and b, "|": a or b, "^": a != b}[op]
        if ta is str and tb is str and op == "-":
            return ord(a) - ord(b)
        if ta is str and tb is int and op in "+-":
            c = ord(a) + (b if op == "+" else -b)
            if not 0 <= c < 128:
                self.fail(f"Char out of range: {c}", node)
            return chr(c)
        if ta is CList and a.is_str and tb is str and op == "+":
            return CList(a.items + [b], True)       # Str + Char
        if ta is CList and tb is CList and op == "+":
            return CList(a.items + b.items, a.is_str and b.is_str)
        if ta is DomVal and tb is int and op in "+-":
            # the optional step, the operator form of `v.offset(k)`: none when it leaves the domain (decisions.md 2.9)
            i = a.ord + (b if op == "+" else -b)
            return DomVal(a.dom, i) if 0 <= i < a.dom.K else None
        if ta is DomVal:
            self.fail(f"a {a.dom!r} is a position, not a number: use `v + k` (a ?{a.dom!r}), `v.next`, `v.cyc(k)` or `v.to_idx`", node)
        self.fail(f"no `{op}` for {self.show(a)} and {self.show(b)}", node)

    def e_cmp(self, e, env):
        a = self.ev(e.first, env)
        for op, rn in zip(e.ops, e.rest):
            b = self.ev(rn, env)
            if not self.compare(op, a, b, e):
                return False
            a = b
        return True

    def compare(self, op, a, b, node):
        if op == "in":
            return self.contains(b, a, node)
        if op == "not in":
            return not self.contains(b, a, node)
        ta, tb = type(a), type(b)
        if op in ("==", "!=") and (ta is Closure or tb is Closure or callable(a) or callable(b)):
            return same_fn(a, b) == (op == "==")
        if ta is not tb and a is not None and b is not None and not (ta in (int, float) and tb in (int, float)):
            self.fail(f"cannot compare {self.show(a)} with {self.show(b)}", node)
        if ta is DomVal and tb is DomVal and a.dom is not b.dom:
            self.fail(f"cannot compare a {a.dom!r} with a {b.dom!r}", node)
        if a is None or b is None:
            if op == "==":
                return a is b
            if op == "!=":
                return a is not b
            self.fail(f"`{op}` on `none`: unwrap with `!` or `??` first", node)
        try:
            if op == "==":
                return a == b
            if op == "!=":
                return not (a == b)
            if op == "<":
                return a < b
            if op == "<=":
                return not (b < a)
            if op == ">":
                return b < a
            return not (a < b)
        except TypeError:
            self.fail(f"cannot compare {self.show(a)} with {self.show(b)}", node)

    def contains(self, c, x, node):
        t = type(c)
        if t is Obj and "contains" in c.stype.decl.methods:
            # `x in s` on a library container asks its `contains`
            return bool(self.call_method_node(c, c.stype.decl.methods["contains"], [x], None, node))
        if t is RangeVal:
            if c.dom is None:
                return type(x) is int and c.lo <= x < c.hi
            return type(x) is DomVal and x.dom is c.dom and c.lo <= x.ord < c.hi
        if t is CList:
            return x in c.items
        if t is CSet or t is CMap:
            return x in c.items
        if t is Domain:
            if type(x) is DomVal:
                return x.dom is c
            return type(x) is int and not c.char and 0 <= x - c.lo < c.K
        self.fail(f"`in` needs a range, list, set or map, got {self.show(c)}", node)

    def offset(self, o, idx, node):
        t = type(o)
        if t is CList:
            i = idx[0]
            if len(idx) != 1 or type(i) is not int:
                self.fail(f"a List is indexed by Int, got {self.show(idx[0])}", node)
            if not 0 <= i < len(o.items):
                self.fail(f"index {i} is out of bounds for a list of length {len(o.items)}", node)
            return i
        if t is CArray:
            if len(idx) == 1 and type(idx[0]) is Tup and len(o.dims) > 1:
                idx = idx[0].items
            if len(idx) != len(o.dims):
                self.fail(f"this array has {len(o.dims)} index(es), got {len(idx)}", node)
            off = 0
            for d, i in zip(o.dims, idx):
                if type(i) is DomVal and i.dom is d:
                    k = i.ord
                elif isinstance(d, Domain) and d.anon and type(i) is int:
                    k = i - d.lo
                    if not 0 <= k < d.K:
                        self.fail(f"index {i} is out of bounds for {d!r}", node)
                elif type(i) is Obj and i.stype is d:
                    k = self.call_method_node(i, d.decl.methods["to_idx"], [], None, node)
                    if not 0 <= k < self.dim_k(d, node):
                        self.fail(f"to_idx gave {k}, outside 0..{self.dim_k(d, node)}", node)
                else:
                    got = f"a {i.dom!r}" if type(i) is DomVal else ("an Int" if type(i) is int else self.show(i))
                    self.fail(f"this array is indexed by {d!r}, got {got}", node)
                off = off * self.dim_k(d, node) + k
            return off
        self.fail(f"cannot index {self.show(o)}", node)

    def e_index(self, e, env):
        o = self.ev(e.obj, env)
        idx = [self.ev(i, env) for i in e.idx]
        t = type(o)
        if t is CMap:
            if idx[0] not in o.items:
                self.fail(f"key {self.show(idx[0])} is not in the map", e)
            return o.items[idx[0]]
        if t is Obj:
            # `xs[i]` on a struct reads through its `get`
            m = o.stype.decl.methods.get("get")
            if m is None:
                self.fail(f"{o.stype!r} cannot be indexed: it has no `get`", e)
            return self.call_method_node(o, m, idx, None, e)
        if len(idx) == 1 and type(idx[0]) is RangeVal:
            r = idx[0]
            if t is CList and r.dom is None or t is CArray and len(o.dims) == 1 and r.dom is o.dims[0]:
                if r.lo < 0 or r.hi > len(o.items):
                    self.fail(f"slice {r!r} is out of bounds for length {len(o.items)}", e)
                return CList(o.items[r.lo:max(r.lo, r.hi)], t is CList and o.is_str)
            self.fail(f"cannot slice {self.show(o)} with {r!r}", e)
        return o.items[self.offset(o, idx, e)]

    def tup_index(self, o, name, node):
        if name.isdigit():
            i = int(name)
            if i < len(o.items):
                return i
        elif o.names and name in o.names:
            return o.names.index(name)
        self.fail(f"{self.show(o)} has no field `{name}`", node)

    def e_field(self, e, env):
        return self.field(self.ev(e.obj, env), e.name, e)

    def field(self, o, name, node):
        t = type(o)
        if t is Obj:
            f = o.f
            if name in f:
                return f[name]
            m = o.stype.decl.methods.get(name)
            if m is not None:
                if any(d.name == "get" for d in m.decos):
                    return self.call_method_node(o, m, [], None, node)
                return Bound(o, m)
            # a value's type arguments are members too: `xs.D` of a `Compressed<D, T>`
            st = o.stype
            if st.targs and name in st.decl.tparams:
                return st.targs[st.decl.tparams.index(name)]
            x = self.extension(o, name)
            if x is not None:
                if self.is_get(x):
                    return self.call_extension(x, o, [], None, node)
                return lambda it, args, kwargs, node: it.call_extension(x, o, args, kwargs, node)
            self.fail(f"{o.stype!r} has no field or method `{name}`", node)
        if t is Tup:
            if not (name.isdigit() or (o.names and name in o.names)):
                x = self.extension(o, name)
                if x is not None:
                    if self.is_get(x):
                        return self.call_extension(x, o, [], None, node)
                    return lambda it, args, kwargs, node: it.call_extension(x, o, args, kwargs, node)
            return o.items[self.tup_index(o, name, node)]
        if t is Module:
            if name not in o.pub:
                self.fail(f"`{name}` is not `pub` in module {o.name}" if name in o.env.vars else f"module {o.name} has no `{name}`", node)
            v = o.env.vars[name]
            return (v.inst(()) if not v.tparams else v.open) if type(v) is StructDecl else v
        if t is StructType:
            d = o.decl
            if name in d.assoc:
                return self.assoc(o, name)
            m = d.methods.get(name)
            if m is not None:
                return Closure(m, o.env, d.mod, o)
            self.fail(f"{o!r} has no associated `{name}`", node)
        p = self.props.get(t)
        if p is not None and name in p:
            return p[name](self, o, node)
        ms = self.vmethods.get(t)
        if ms is not None and name in ms:
            m = ms[name]
            return lambda it, args, kwargs, node: m(it, o, args, kwargs, node)
        x = self.extension(o, name)
        if x is not None:
            if self.is_get(x):
                return self.call_extension(x, o, [], None, node)
            return lambda it, args, kwargs, node: it.call_extension(x, o, args, kwargs, node)
        self.fail(f"{self.show(o)} has no field or method `{name}`", node)

    def e_call(self, e, env, raw=False):
        fn = e.fn
        if fn.kind == "tapp" and fn.obj.kind == "name":
            fn = N("generic", fn.line, name=fn.obj.name, args=fn.targs)
        if fn.kind == "generic" and fn.name in ("read", "ask") and not env.has(fn.name):
            if len(fn.args) != 1:
                self.fail(f"`{fn.name}::<T>()` takes one type", e)
            bare = N("call", e.line, fn=N("name", e.line, name=fn.name), args=e.args)
            return self.io(bare, fn.args[0], env, env)
        fn = e.fn
        if self.is_io(e, env):
            self.fail(f"{fn.name}() cannot tell what type to read here: give the place a type "
                      f"(`let x: T = {fn.name}(...)`), or say `{fn.name}::<T>(...)`", e)
        if fn.kind == "name" and fn.name == "swap" and len(e.args) == 2:
            ga, pa = self.place(e.args[0], env)
            gb, pb = self.place(e.args[1], env)
            x, y = ga(), gb()
            pa(y)
            pb(x)
            return None
        target = None
        if fn.kind == "field":
            target = self.ev(fn.obj, env)
        else:
            f = self.ev(fn, env)
        args, kwargs = [], None
        muts = None
        for i, a in enumerate(e.args):
            if a.kind == "kwarg":
                if kwargs is None:
                    kwargs = {}
                kwargs[a.name] = self.ev(a.value, env)
            else:
                if getattr(a, "mut", False):
                    muts = (muts or []) + [i]
                if a.kind == "call" and self.is_io(a, env):
                    # an argument is read as the parameter's type: `solve(read(), read())`
                    ps = self.callee_params(f if target is None and fn.kind != "field" else None)
                    if ps is not None and len(args) < len(ps[0]) and ps[0][len(args)].ty is not None:
                        args.append(self.io(a, ps[0][len(args)].ty, ps[1], env))
                        continue
                args.append(self.ev(a, env))
        self.last_call = None
        if fn.kind == "field":
            r = self.method(target, fn.name, args, kwargs, e, env, raw)
        else:
            t = type(f)
            if t is StructType:
                r = self.construct(f, args, kwargs, e, env)
            elif t is Closure:
                if f.stype is not None and f.stype.targs is None and f.stype.decl.tparams:
                    st = self.infer(f.stype, [(p.ty, p.name) for p in f.node.params], args, kwargs, e)
                    r = self.call_static(st, f.node, args, kwargs, e)
                else:
                    r = self.call_fn(f.node, f.env, f.mod, args, kwargs, e, raw=raw)
            else:
                r = self.call(f, args, kwargs, e)
        if muts and self.last_call is not None:
            fnode, frame = self.last_call
            # only the frame of the fn called here; a builtin such as `sort` leaves none
            if fn.kind == "field":
                callee = target.stype.decl.methods.get(fn.name) if type(target) in (Obj, StructType) else None
            else:
                callee = f.node if type(f) is Closure else None
            if fnode is not callee:
                return r
            params = fnode.params[1:] if fnode.params and fnode.params[0].name == "self" else fnode.params
            for i in muts:
                if i < len(params):
                    nv = frame.vars.get(params[i].name)
                    if nv is not args[i]:
                        self.place(e.args[i], env)[1](nv)
        return r

    def callee_params(self, f):
        if type(f) is Closure and f.node.kind == "fn" and not f.node.tparams:
            return f.node.params, f.env
        return None

    def method(self, o, name, args, kwargs, node, env, raw=False):
        t = type(o)
        if t is Obj:
            m = o.stype.decl.methods.get(name)
            if m is not None:
                if not m.params or m.params[0].name != "self":
                    return self.call_static(o.stype, m, args, kwargs, node)
                return self.call_method_node(o, m, args, kwargs, node, raw)
            if name in o.f:
                return self.call(o.f[name], args, kwargs, node)
            x = self.extension(o, name)
            if x is not None:
                return self.call_extension(x, o, args, kwargs, node)
            self.fail(f"{o.stype!r} has no method `{name}`", node)
        if t is StructType:
            m = o.decl.methods.get(name)
            if m is None:
                self.fail(f"{o!r} has no fn `{name}`", node)
            st = o
            if st.targs is None and o.decl.tparams:
                if env.has("$decl") and env.get("$decl") is o.decl:
                    st = env.get("Self")
                else:
                    st = self.infer(st, [(p.ty, p.name) for p in m.params], args, kwargs, node)
            return self.call_static(st, m, args, kwargs, node)
        if t is Module:
            v = self.field(o, name, node)
            if type(v) is StructType:
                return self.construct(v, args, kwargs, node, env)
            return self.call(v, args, kwargs, node)
        ms = self.vmethods.get(t)
        if ms is not None and name in ms:
            return ms[name](self, o, args, kwargs, node)
        if t is Tup and (o.names and name in o.names):
            return self.call(o.items[o.names.index(name)], args, kwargs, node)
        x = self.extension(o, name)
        if x is not None:
            return self.call_extension(x, o, args, kwargs, node)
        self.fail(f"{self.show(o)} has no method `{name}`", node)


TYPE_KINDS = {"tname", "tpath", "tarray", "tlist", "topt", "ttuple", "trange", "texpr", "tfn", "tfresh", "tyields"}
