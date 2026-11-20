"""Library syntax on top of the solution-file parser in ../cpc.

Adds what a library module needs and a single solution file does not:
`use`, `pub`, generic structs with a field header, generic `const`
operation values, fn types, `trait` and `impl` blocks, `where` clauses.
The cpc parser is subclassed, not edited, so the C++ prototype is untouched.
"""
from cpc.parser import Parser, ParseError, OP_FNS  # noqa: F401 (re-exported)
from cpc.ast import N

DECL_KW = {"fn", "struct", "const", "type", "enum", "pure", "domain"}


class LibParser(Parser):
    def __init__(self, src):
        super().__init__(src)
        self.docs = doc_comments(src)

    def tok_is(self, k, kind, val):
        t = self.peek(k)
        return t.kind == kind and t.val == val

    def at(self, *vals):
        # `Fenwick<V, Mod<P>>`: in a type-argument list `>>` closes two lists
        if vals == (">",) and self.t.kind == "OP" and self.t.val == ">>":
            t = self.t
            t.val = ">"
            self.toks.insert(self.i + 1, type(t)("OP", ">", t.line, t.col + 1))
        return super().at(*vals)

    def atom(self):
        e = super().atom()
        if e.kind == "tapp" and e.obj.kind == "name":
            return N("generic", e.line, name=e.obj.name, args=e.targs)
        # `scc::<V>(adj)`: explicit type arguments on a fn (decisions 2.13)
        if e.kind == "name" and self.at(":") and self.tok_is(1, "OP", ":") and self.tok_is(2, "OP", "<"):
            self.i += 3
            args = []
            while not self.at(">"):
                args.append(self.type_or_expr())
                if self.at(","):
                    self.next()
            self.expect(">")
            e = N("generic", e.line, name=e.name, args=args)
        return e

    def at_name(self, val):
        return self.t.kind == "NAME" and self.t.val == val

    # ---- statements
    def stmt(self):
        t = self.t
        line = t.line
        if self.at_name("pub") and (self.peek().kind == "KW" and self.peek().val in DECL_KW
                                    or self.tok_is(1, "NAME", "trait") or self.tok_is(1, "OP", "@")):
            self.next()
            s = self.stmt()
            s.pub = True
            return s
        if self.at_name("use") and self.peek().kind == "NAME":
            return self.use_stmt()
        if self.at_name("trait") and self.peek().kind == "NAME":
            return self.trait_def()
        if self.at_name("impl") and self.peek().kind == "NAME":
            return self.impl_def()
        if self.at("const") and self.tok_is(2, "OP", "<"):
            self.next()
            nm = self.name()
            tparams = self.tparams()
            self.expect("=")
            e = self.expr()
            self.skip_where()
            self.end_line()
            return N("gconst", line, name=nm, tparams=tparams, value=e)
        s = super().stmt()
        if s.kind in ("fn", "struct", "gconst") or (s.kind == "let" and getattr(s, "const", False)):
            s.doc = self.docs.get(line, "")
        return s

    def trait_def(self):
        """`trait Name:` and its signatures. A method written with a body is the trait's default."""
        line = self.next().line
        nm = self.name()
        defaults = []
        while not self.at_kind("NEWLINE") and not self.at_kind("EOF"):
            self.next()
        if self.at_kind("NEWLINE"):
            self.next()
        if self.at_kind("INDENT"):
            self.next()
            self.skip_newlines()
            while not self.at_kind("DEDENT") and not self.at_kind("EOF"):
                start = self.i
                try:
                    m = self.member()
                    if m is not None and m.kind == "fn":
                        defaults.append(m)
                    elif m is None:
                        raise ParseError("not a member")
                except ParseError:
                    # a signature with no body: nothing to run
                    self.i = start
                    while not self.at_kind("NEWLINE") and not self.at_kind("EOF"):
                        self.next()
                self.skip_newlines()
            if self.at_kind("DEDENT"):
                self.next()
        return N("trait", line, name=nm, defaults=defaults)

    def params(self):
        """Parameters, with `name: T = default` for a trailing argument the caller may leave out."""
        self.expect("(")
        ps = []
        pending = []
        while not self.at(")"):
            mode = None
            if self.t.kind == "NAME" and self.t.val == "mut" and self.peek().kind == "NAME":
                self.next()
                mode = "inout"
            nm = self.name()
            if self.at(":"):
                self.next()
                ty = self.type_()
                default = None
                if self.at("="):
                    self.next()
                    default = self.expr()
                for pn, pm in pending + [(nm, mode)]:
                    ps.append(N("param", self.t.line, name=pn, ty=ty, mode=pm, default=default if pn == nm else None))
                pending = []
            else:
                pending.append((nm, mode))
            if self.at(","):
                self.next()
        self.expect(")")
        for pn, pm in pending:
            ps.append(N("param", self.t.line, name=pn, ty=None, mode=pm, default=None))
        return ps

    def call_args(self):
        # `Action(..., over: +)`: a bare operator is also allowed as a named argument
        j, depth = self.i + 1, 0
        while j + 2 < len(self.toks) and depth >= 0:
            t = self.toks[j]
            if t.kind == "OP" and t.val in ("(", "[", "{"):
                depth += 1
            elif t.kind == "OP" and t.val in (")", "]", "}"):
                depth -= 1
            elif depth == 0 and t.kind == "NAME" and self.toks[j + 1].kind == "OP" and self.toks[j + 1].val == ":":
                op, after = self.toks[j + 2], self.toks[j + 3]
                if op.kind == "OP" and op.val in OP_FNS and after.kind == "OP" and after.val in (",", ")"):
                    a, b = type(t)("NAME", "_a", op.line, op.col), type(t)("NAME", "_b", op.line, op.col)
                    wrap = [type(t)("OP", "(", op.line, op.col), a, type(t)("OP", ",", op.line, op.col), b,
                            type(t)("OP", ")", op.line, op.col), type(t)("OP", "=>", op.line, op.col),
                            type(t)("NAME", "_a", op.line, op.col), op, type(t)("NAME", "_b", op.line, op.col)]
                    self.toks[j + 2:j + 3] = wrap
            j += 1
        return super().call_args()

    def skip_to_block_end(self):
        """Skip a header line and the indented block under it, if any."""
        while not self.at_kind("NEWLINE") and not self.at_kind("EOF"):
            self.next()
        if self.at_kind("NEWLINE"):
            self.next()
        if self.at_kind("INDENT"):
            depth = 0
            while True:
                k = self.next().kind
                if k == "INDENT":
                    depth += 1
                elif k == "DEDENT":
                    depth -= 1
                    if depth == 0:
                        break
                elif k == "EOF":
                    break

    def skip_where(self):
        """`where T: Ord + Add` only constrains; the interpreter checks by running."""
        if not self.at("where"):
            return
        j = self.i
        while self.toks[j].kind not in ("NEWLINE", "EOF"):
            j += 1
        end = j
        last = self.toks[j - 1]
        if last.kind == "OP" and last.val == ":":
            end = j - 1
        else:
            for k in range(self.i, j):
                tk = self.toks[k]
                if tk.kind == "OP" and tk.val == "=":
                    end = k
                    break
        del self.toks[self.i:end]

    def tparams(self):
        ps = []
        if self.at("<"):
            self.next()
            while not self.at(">"):
                if self.at("const"):
                    self.next()
                ps.append(self.name())
                if self.at(":"):
                    self.next()
                    self.type_()
                    while self.at("+"):
                        self.next()
                        self.type_()
                if self.at(","):
                    self.next()
            self.expect(">")
        return ps

    def impl_def(self):
        line = self.next().line
        trait = self.name()
        if not self.at("for"):
            self.err("expected `for` in `impl Trait for Type`")
        self.next()
        target = self.name()
        self.tparams()
        members = []
        if self.at(":"):
            self.next()
            if self.at_kind("NEWLINE"):
                self.next()
                self.expect_kind("INDENT")
                self.skip_newlines()
                while not self.at_kind("DEDENT") and not self.at_kind("EOF"):
                    members.append(self.member())
                    self.skip_newlines()
                if self.at_kind("DEDENT"):
                    self.next()
            else:
                members.append(self.member())
        else:
            self.end_line()
        return N("impl", line, trait=trait, target=target, body=members)

    def member(self):
        """One line of a struct or impl body: a fn, a law, or `let K = ...`."""
        line = self.t.line
        pub = False
        decos = []
        while True:
            if self.at_name("pub"):
                self.next()
                pub = True
            elif self.at("@"):
                self.next()
                decos.append(N("deco", line, name=self.name(), args=[]))
                self.skip_newlines()
            else:
                break
        if self.at("fn"):
            f = self.fn_def()
            f.decos = decos
            f.pub = pub
            f.doc = self.docs.get(line, "")
            return f
        if self.at_name("law"):
            self.next()
            nm = self.name()
            ps = self.params()
            self.expect("=")
            e = self.expr()
            self.end_line()
            return N("lawdef", line, name=nm, params=ps, expr=e)
        if self.at("domain"):
            # a domain owned by the struct: fresh for each choice of type arguments
            self.next()
            nm = self.name()
            self.expect("=")
            r = self.expr()
            self.end_line()
            return N("assoc", line, name=nm, value=N("domexpr", line, name=nm, range=r))
        if self.at("let") or self.at("const"):
            self.next()
            nm = self.name()
            if self.at(":"):
                self.next()
                self.type_()
            self.expect("=")
            e = self.expr()
            self.end_line()
            return N("assoc", line, name=nm, value=e)
        return None

    def fields(self, close):
        out = []
        while not self.at(close):
            names = [self.name()]
            while self.at(","):
                self.next()
                names.append(self.name())
            self.expect(":")
            ty = self.type_()
            out += [(f, ty) for f in names]
            if self.at(","):
                self.next()
        return out

    def struct_def(self):
        line = self.next().line
        nm = self.name()
        tparams = self.tparams()
        fields, methods, laws, assocs = [], [], [], []
        if self.at("("):
            self.next()
            fields = self.fields(")")
            self.expect(")")
        self.skip_where()
        if self.at(":"):
            self.next()
            self.expect_kind("NEWLINE")
            self.expect_kind("INDENT")
            self.skip_newlines()
            while not self.at_kind("DEDENT") and not self.at_kind("EOF"):
                m = self.member()
                if m is None:
                    names = [self.name()]
                    while self.at(","):
                        self.next()
                        names.append(self.name())
                    self.expect(":")
                    ty = self.type_()
                    fields += [(f, ty) for f in names]
                    self.end_line()
                elif m.kind == "fn":
                    methods.append(m)
                elif m.kind == "lawdef":
                    laws.append(m)
                else:
                    assocs.append(m)
                self.skip_newlines()
            if self.at_kind("DEDENT"):
                self.next()
        else:
            self.end_line()
        return N("struct", line, name=nm, tparams=tparams, fields=fields, methods=methods, laws=laws, assocs=assocs, decos=[])

    def fn_def(self):
        # drop the `where` clause of the header, then parse as usual
        j = self.i
        while self.toks[j].kind not in ("NEWLINE", "EOF"):
            if self.toks[j].kind == "KW" and self.toks[j].val == "where":
                save, self.i = self.i, j
                self.skip_where()
                self.i = save
                break
            j += 1
        return super().fn_def()

    # ---- types
    def type_(self):
        line = self.t.line
        ty = super().type_()
        # `Rooted<V>.Pre`: a domain or const owned by another type
        while ty.kind == "tname" and self.at(".") and self.peek().kind == "NAME" and self.peek().val[:1].isupper():
            self.next()
            ty = N("tpath", line, base=ty, name=self.next().val, args=[])
        if self.at("->"):
            self.next()
            params = [f[1] for f in ty.fields] if ty.kind == "ttuple" else [ty]
            return N("tfn", line, params=params, ret=self.type_())
        return ty

    # ---- expressions
    def looks_generic(self):
        j, d = self.i, 0
        while j < len(self.toks):
            t = self.toks[j]
            if t.kind == "OP" and t.val == "<":
                d += 1
            elif t.kind == "OP" and t.val == ">":
                d -= 1
                if d == 0:
                    nx = self.toks[j + 1]
                    return nx.kind in ("NEWLINE", "EOF") or (nx.kind == "OP" and nx.val in ("(", ".", ")", ",", "]"))
            elif t.kind == "OP" and t.val == ">>":
                d -= 2
                if d < 0:
                    return False
                if d == 0:
                    nx = self.toks[j + 1]
                    return nx.kind in ("NEWLINE", "EOF") or (nx.kind == "OP" and nx.val in ("(", ".", ")", ",", "]"))
            elif t.kind in ("NEWLINE", "EOF", "KW", "STR", "CHAR") or (
                    t.kind == "OP" and t.val not in (",", "(", ")", "[", "]", "?", ".")):
                return False
            j += 1
        return False


def doc_comments(src):
    """Map a declaration's line number to the `///` block right above it."""
    docs, buf = {}, []
    for ln, raw in enumerate(src.split("\n"), 1):
        s = raw.strip()
        if s.startswith("///"):
            buf.append(s[3:].strip() if len(s) > 3 else "")
        elif s.startswith("@") and buf:
            continue
        else:
            if buf and s:
                docs[ln] = "\n".join(buf)
            buf = []
    return docs


def parse(src):
    return LibParser(src).program()
