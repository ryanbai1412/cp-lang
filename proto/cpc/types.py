INT_MAX = (1 << 63) - 1
INT_MIN = -(1 << 63)
DOM_MAX = (1 << 31) - 1    # a domain value is a 32-bit position

class T:
    def __eq__(self, o):
        return type(self) is type(o) and self.key() == o.key()
    def __hash__(self):
        return hash(self.key())
    def key(self):
        return ()

class TInt(T):
    def __init__(self, lo=None, hi=None):
        self.lo, self.hi = lo, hi
    def key(self):
        return ()
    def __repr__(self):
        if self.lo is None and self.hi is None:
            return "Int"
        return f"Int[{self.lo}..={self.hi}]"

class TBool(T):
    def __repr__(self): return "Bool"
class TChar(T):
    def __repr__(self): return "Char"
class TStr(T):
    def __repr__(self): return "Str"
class TF64(T):
    def __repr__(self): return "F64"
class TU64(T):
    def __repr__(self): return "U64"
class TI128(T):
    def __repr__(self): return "I128"
class TUnit(T):
    def __repr__(self): return "()"

class DomInfo:
    """A nominal domain. Values are stored as their 0-based ordinal (C++ int)."""
    def __init__(self, name, cpp, lo_cpp, len_cpp, lo_iv, len_iv, anon=False, labels="int"):
        self.name, self.cpp = name, cpp
        self.lo_cpp, self.len_cpp = lo_cpp, len_cpp
        self.lo_iv, self.len_iv = lo_iv, len_iv
        self.anon = anon
        self.labels = labels
        self.bounds = None
        self.of = None
        self.hi_cpp = None
    def __repr__(self):
        return self.name

class TDom(T):
    def __init__(self, info):
        self.info = info
    def key(self):
        return (id(self.info),)
    def __repr__(self):
        return self.info.name

class TArr(T):
    def __init__(self, dims, elem):
        self.dims, self.elem = dims, elem
    def key(self):
        return (tuple(self.dims), self.elem)
    def __repr__(self):
        return "[" + ", ".join(map(repr, self.dims)) + "]" + repr(self.elem)

class TOpt(T):
    def __init__(self, inner):
        self.inner = inner
    def key(self):
        return (self.inner,)
    def __repr__(self):
        return "?" + repr(self.inner)

class TTuple(T):
    def __init__(self, names, types, cpp):
        self.names, self.types, self.cpp = names, types, cpp
    def key(self):
        return (tuple(self.names), tuple(self.types))
    def field(self, name):
        if name in self.names:
            i = self.names.index(name)
            return i
        if name.isdigit() and int(name) < len(self.types):
            return int(name)
        return None
    def __repr__(self):
        return "(" + ", ".join(f"{n}: {t!r}" if n else repr(t) for n, t in zip(self.names, self.types)) + ")"

class TList(T):
    def __init__(self, elem):
        self.elem = elem
    def key(self):
        return (self.elem,)
    def __repr__(self):
        return f"List<{self.elem!r}>"

class TRange(T):
    def __init__(self, elem):
        self.elem = elem
    def key(self):
        return (self.elem,)
    def __repr__(self):
        return f"Range<{self.elem!r}>"

class TMod(T):
    def __init__(self, m):
        self.m = m
    def key(self):
        return (self.m,)
    def __repr__(self):
        return f"Mod<{self.m}>"

class TFn(T):
    def __init__(self, params, ret):
        self.params, self.ret = params, ret
    def key(self):
        return (id(self),)
    def __repr__(self):
        return f"fn({', '.join(map(repr, self.params))}) -> {self.ret!r}"

class TType(T):
    """A type used as a value (domain names, `Int`, ...)."""
    def __init__(self, t):
        self.t = t
    def key(self):
        return (self.t,)
    def __repr__(self):
        return f"type {self.t!r}"

INT = TInt()
BOOL, CHAR, STR, UNIT = TBool(), TChar(), TStr(), TUnit()
F64, U64, I128 = TF64(), TU64(), TI128()

def is_int(t):
    return isinstance(t, TInt)

# ---- interval arithmetic on (lo, hi), None = unbounded
def iv_add(a, b):
    return (None if a.lo is None or b.lo is None else a.lo + b.lo,
            None if a.hi is None or b.hi is None else a.hi + b.hi)

def iv_neg(a):
    return (None if a.hi is None else -a.hi, None if a.lo is None else -a.lo)

def iv_mul(a, b):
    if None in (a.lo, a.hi, b.lo, b.hi):
        # sign-aware partial knowledge
        if a.lo is not None and b.lo is not None and a.lo >= 0 and b.lo >= 0:
            return (a.lo * b.lo, None)
        return (None, None)
    c = [a.lo * b.lo, a.lo * b.hi, a.hi * b.lo, a.hi * b.hi]
    return (min(c), max(c))

def iv_join(a, b):
    lo = None if a.lo is None or b.lo is None else min(a.lo, b.lo)
    hi = None if a.hi is None or b.hi is None else max(a.hi, b.hi)
    return (lo, hi)

def fits(lo, hi):
    return (lo is None or lo >= INT_MIN) and (hi is None or hi <= INT_MAX)
