"""Runtime values of the reference interpreter."""

I64_MIN, I64_MAX = -(1 << 63), (1 << 63) - 1


class CpError(Exception):
    """A runtime failure: a failed assert, a bad index, an overflow."""


class ReturnEx(Exception):
    def __init__(self, value):
        self.value = value


class BreakEx(Exception):
    pass


class ContinueEx(Exception):
    pass


class ConsumerExit(Exception):
    """The loop consuming a generator left early (break or return)."""
    def __init__(self, token, inner):
        self.token, self.inner = token, inner


class Env:
    __slots__ = ("vars", "parent")

    def __init__(self, parent=None, vars=None):
        self.vars = {} if vars is None else vars
        self.parent = parent

    def get(self, name):
        e = self
        while e is not None:
            v = e.vars
            if name in v:
                return v[name]
            e = e.parent
        raise KeyError(name)

    def has(self, name):
        e = self
        while e is not None:
            if name in e.vars:
                return True
            e = e.parent
        return False

    def set(self, name, val):
        e = self
        while e is not None:
            if name in e.vars:
                e.vars[name] = val
                return True
            e = e.parent
        return False


class TypeV:
    """A built-in type used as a value: Int, Float, Bool, Char, Str, List<T>, ..."""
    def __init__(self, name, args=()):
        self.name, self.args = name, tuple(args)

    def __eq__(self, o):
        return isinstance(o, TypeV) and self.name == o.name and self.args == o.args

    def __hash__(self):
        return hash((self.name, self.args))

    def __repr__(self):
        return self.name + ("<" + ", ".join(map(repr, self.args)) + ">" if self.args else "")


INT, FLOAT, BOOL, CHAR, STR = (TypeV(n) for n in ("Int", "Float", "Bool", "Char", "Str"))


class Domain:
    """A nominal bounded index type: `domain V = 1..=n` (plan/19 3.1).

    `lo` is the label of the first position; K is frozen when it is declared.
    `of` is set for Bounds<V>, the K + 1 boundaries around V's positions.
    """
    def __init__(self, name, lo, K, char=False, of=None):
        self.name, self.lo, self.K, self.char, self.of = name, lo, K, char, of
        self._bounds = None

    @property
    def bounds(self):
        if self._bounds is None:
            self._bounds = Domain(f"Bounds<{self.name}>", 0, self.K + 1, of=self)
        return self._bounds

    def __repr__(self):
        return self.name


class DomVal:
    __slots__ = ("dom", "ord")

    def __init__(self, dom, ord):
        self.dom, self.ord = dom, ord

    def __eq__(self, o):
        return isinstance(o, DomVal) and o.dom is self.dom and o.ord == self.ord

    def __lt__(self, o):
        return self.ord < o.ord

    def __le__(self, o):
        return self.ord <= o.ord

    def __hash__(self):
        return hash(self.ord)

    @property
    def label(self):
        d = self.dom
        return chr(ord(d.lo) + self.ord) if d.char else d.lo + self.ord

    def __repr__(self):
        return f"{self.dom.name}({self.label!r})" if self.dom.of is None else f"{self.dom.name}#{self.ord}"


class RangeVal:
    """`a..b` over Int (dom is None) or over the positions of a domain; hi is exclusive."""
    __slots__ = ("dom", "lo", "hi")

    def __init__(self, dom, lo, hi):
        self.dom, self.lo, self.hi = dom, lo, hi

    def __len__(self):
        return max(0, self.hi - self.lo)

    def values(self):
        if self.dom is None:
            return range(self.lo, self.hi)
        d = self.dom
        return (DomVal(d, i) for i in range(self.lo, self.hi))

    def __eq__(self, o):
        return isinstance(o, RangeVal) and (o.dom, o.lo, o.hi) == (self.dom, self.lo, self.hi)

    def __repr__(self):
        return f"{self.lo}..{self.hi}" if self.dom is None else f"{self.dom.name}[{self.lo}..{self.hi}]"


class CList:
    """List<T>; a Str is a List of one-byte Chars."""
    __slots__ = ("items", "is_str")

    def __init__(self, items, is_str=False):
        self.items, self.is_str = items, is_str

    def __eq__(self, o):
        return isinstance(o, CList) and self.items == o.items

    def __lt__(self, o):
        return self.items < o.items

    def __hash__(self):
        return hash(tuple(self.items))

    def __repr__(self):
        return repr("".join(self.items)) if self.is_str else repr(self.items)


class CArray:
    """[D]T: one flat block indexed by the positions of its domains."""
    __slots__ = ("dims", "items")

    def __init__(self, dims, items):
        self.dims, self.items = dims, items

    def __eq__(self, o):
        return isinstance(o, CArray) and self.dims == o.dims and self.items == o.items

    def __repr__(self):
        return repr(self.items)


class Tup:
    """A tuple or a record: (a, b) or (to: v, w: 3)."""
    __slots__ = ("items", "names")

    def __init__(self, items, names=None):
        self.items, self.names = items, names

    def __eq__(self, o):
        return isinstance(o, Tup) and self.items == o.items

    def __lt__(self, o):
        return self.items < o.items

    def __le__(self, o):
        return self.items <= o.items

    def __hash__(self):
        return hash(tuple(self.items))

    def __repr__(self):
        if self.names and any(self.names):
            return "(" + ", ".join(f"{n}: {v!r}" if n else repr(v) for n, v in zip(self.names, self.items)) + ")"
        return "(" + ", ".join(map(repr, self.items)) + ")"


class StructDecl:
    def __init__(self, node, env, mod):
        self.node, self.env, self.mod = node, env, mod
        self.name = node.name
        self.tparams = list(getattr(node, "tparams", []))
        self.fields = [f for f, _ in node.fields]
        self.ftypes = dict(node.fields)
        self.methods = {m.name: m for m in node.methods}
        self.laws = list(getattr(node, "laws", []))
        self.assoc = {a.name: (a.value, env) for a in getattr(node, "assocs", [])}
        self.insts = {}
        self.open = StructType(self, None)

    def inst(self, targs):
        key = tuple(id(t) if not isinstance(t, (int, TypeV)) else t for t in targs)
        st = self.insts.get(key)
        if st is None:
            st = self.insts[key] = StructType(self, tuple(targs))
        return st


class StructType:
    """A struct with its type arguments; `targs` is None until they are inferred."""
    def __init__(self, decl, targs):
        self.decl, self.targs = decl, targs
        self._env = None
        self.consts = {}

    @property
    def env(self):
        if self._env is None:
            vars = {"$decl": self.decl, "Self": self}
            if self.targs is not None:
                vars.update(zip(self.decl.tparams, self.targs))
            self._env = Env(self.decl.env, vars)
        return self._env

    def __repr__(self):
        if not self.targs:
            return self.decl.name
        return self.decl.name + "<" + ", ".join(map(repr, self.targs)) + ">"


class Obj:
    __slots__ = ("stype", "f")
    interp = None

    def __init__(self, stype, f):
        self.stype, self.f = stype, f

    def __eq__(self, o):
        if not isinstance(o, Obj) or o.stype.decl is not self.stype.decl:
            return False
        m = self.stype.decl.methods.get("eq")
        if m is not None:
            return bool(Obj.interp.call_static(self.stype, m, [self, o]))
        return self.f == o.f

    def __lt__(self, o):
        m = self.stype.decl.methods.get("lt")
        if m is None:
            return tuple(self.f.values()) < tuple(o.f.values())
        return bool(Obj.interp.call_static(self.stype, m, [self, o]))

    def __hash__(self):
        return hash(tuple(self.f.values()))

    def __repr__(self):
        m = self.stype.decl.methods.get("show")
        if m is not None:
            it = Obj.interp
            if m.params and m.params[0].name == "self":
                return it.show(it.call_method_node(self, m, [], {}))
            return it.show(it.call_static(self.stype, m, [self]))
        return self.stype.decl.name + "(" + ", ".join(f"{k}: {v!r}" for k, v in self.f.items()) + ")"


class Closure:
    __slots__ = ("node", "env", "mod", "stype")

    def __init__(self, node, env, mod, stype=None):
        self.node, self.env, self.mod, self.stype = node, env, mod, stype

    def __repr__(self):
        return f"<fn {getattr(self.node, 'name', 'lambda')}>"


class Bound:
    """A method bound to its receiver."""
    __slots__ = ("obj", "node")

    def __init__(self, obj, node):
        self.obj, self.node = obj, node


class GenVal:
    """A generator: `run(emit)` pushes each value into the consumer's loop body."""
    __slots__ = ("run",)

    def __init__(self, run):
        self.run = run


class Lent:
    """What a `lends` projection hands out: a place, usable only where it was lent."""
    __slots__ = ("node", "env")

    def __init__(self, node, env):
        self.node, self.env = node, env


class Module:
    def __init__(self, name, path, env):
        self.name, self.path, self.env = name, path, env
        self.pub = set()
        self.tests = []
        self.decls = []


class GConst:
    """`const Sum<T> = Group<T>(...)`: one value per type argument."""
    def __init__(self, node, env):
        self.node, self.env, self.cache = node, env, {}


class CSet:
    __slots__ = ("items",)

    def __init__(self, items):
        self.items = items

    def __repr__(self):
        return "{" + ", ".join(map(repr, sorted(self.items))) + "}"


class CMap:
    __slots__ = ("items",)

    def __init__(self, items):
        self.items = items

    def __repr__(self):
        return "{" + ", ".join(f"{k!r}: {v!r}" for k, v in sorted(self.items.items())) + "}"
