class N:
    """Generic AST node: N('kind', line, field=value, ...)."""
    def __init__(self, kind, line=0, **kw):
        self.kind = kind
        self.line = line
        self.__dict__.update(kw)
    def __repr__(self):
        fs = ", ".join(f"{k}={v!r}" for k, v in self.__dict__.items() if k not in ("kind", "line"))
        return f"{self.kind}({fs})"
