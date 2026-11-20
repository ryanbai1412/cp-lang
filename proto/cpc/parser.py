import copy
from .lexer import lex
from .ast import N

class ParseError(Exception):
    pass

ASSIGN_OPS = {"=", "+=", "-=", "*=", "/=", "%=", "|=", "&=", "^="}
CMP_OPS = {"==", "!=", "<", ">", "<=", ">="}

class Parser:
    def __init__(self, src):
        self.toks = lex(src)
        self.i = 0

    # ---- token helpers
    @property
    def t(self):
        return self.toks[self.i]
    def peek(self, k=1):
        return self.toks[min(self.i + k, len(self.toks) - 1)]
    def at(self, *vals):
        t = self.t
        return t.kind in ("OP", "KW") and t.val in vals
    def at_kind(self, kind):
        return self.t.kind == kind
    def next(self):
        t = self.t
        self.i += 1
        return t
    def err(self, msg):
        t = self.t
        raise ParseError(f"line {t.line}:{t.col}: {msg} (at {t.val!r})")
    def expect(self, val):
        if not self.at(val):
            self.err(f"expected {val!r}")
        return self.next()
    def at_close_angle(self):
        return self.at(">", ">>")
    def close_angle(self):
        """The `>` that ends type arguments; `List<List<Int>>` ends two lists with one `>>` token."""
        if self.at(">>"):
            t = copy.copy(self.t)
            self.t.val, t.val, t.col = ">", ">", t.col + 1
            self.toks.insert(self.i + 1, t)
        return self.expect(">")
    def expect_kind(self, kind):
        if self.t.kind != kind:
            self.err(f"expected {kind}")
        return self.next()
    def name(self):
        return self.expect_kind("NAME").val
    def skip_newlines(self):
        while self.at_kind("NEWLINE"):
            self.next()

    # ---- program
    def program(self):
        body = []
        self.skip_newlines()
        while not self.at_kind("EOF"):
            body.append(self.stmt())
            self.skip_newlines()
        return N("program", 1, body=body)

    def block(self):
        self.expect(":")
        if self.at_kind("NEWLINE"):
            self.next()
            self.expect_kind("INDENT")
            body = []
            self.skip_newlines()
            while not self.at_kind("DEDENT") and not self.at_kind("EOF"):
                body.append(self.stmt())
                self.skip_newlines()
            if self.at_kind("DEDENT"):
                self.next()
            return body
        s = self.simple_stmt()
        body = [s]
        while self.at(";"):
            self.next()
            body.append(self.simple_stmt())
        if self.at("else") or self.at("elif"):
            return body
        self.end_line()
        return body

    def end_line(self):
        if getattr(self, "after_block", False):
            self.after_block = False
            return
        if self.at_kind("NEWLINE"):
            self.next()
        elif not (self.at_kind("DEDENT") or self.at_kind("EOF")):
            self.err("expected end of line")

    def stmt(self):
        t = self.t
        line = t.line
        if self.at("@"):
            decos = []
            while self.at("@"):
                self.next()
                dname = self.name()
                args = []
                if self.at("("):
                    args = self.call_args()
                decos.append(N("deco", line, name=dname, args=args))
                self.skip_newlines()
            s = self.stmt()
            s.decos = decos
            return s
        if t.kind == "KW":
            v = t.val
            if v == "domain":
                self.next()
                nm = self.name()
                self.expect("=")
                e = self.expr()
                self.end_line()
                return N("domain", line, name=nm, range=e)
            if v == "const":
                self.next()
                nm = self.name()
                ty = None
                if self.at(":"):
                    self.next()
                    ty = self.type_()
                self.expect("=")
                e = self.expr()
                self.end_line()
                return N("let", line, pat=N("pname", line, name=nm), ty=ty, value=e, mut=False, const=True)
            if v == "type":
                self.next()
                nm = self.name()
                self.expect("=")
                ty = self.type_()
                self.end_line()
                return N("typedef", line, name=nm, ty=ty)
            if v == "fn":
                return self.fn_def()
            if v == "pure":
                self.next()
                if not self.at("fn"):
                    self.err("expected fn after pure")
                f = self.fn_def()
                f.pure = True
                return f
            if v == "struct":
                return self.struct_def()
        if t.kind == "NAME" and t.val == "impl" and self.peek().kind == "NAME":
            # `impl Index for Cell:` followed by `let K = ...` and fns (plan/19 §3.1)
            self.next()
            trait = self.name()
            if not self.at("for"):
                self.err("expected `for` in `impl Trait for Type`")
            self.next()
            target = self.name()
            body = self.block() if self.at(":") else []
            if not body:
                self.end_line()
            return N("impl", line, trait=trait, target=target, body=body)
        if t.kind == "KW":
            v = t.val
            if v == "enum":
                self.next()
                nm = self.name()
                self.expect(":")
                self.expect_kind("NEWLINE")
                self.expect_kind("INDENT")
                variants = []
                self.skip_newlines()
                while not self.at_kind("DEDENT") and not self.at_kind("EOF"):
                    vn = self.name()
                    fields = []
                    if self.at("("):
                        self.next()
                        while not self.at(")"):
                            fns = [self.name()]
                            while self.at(","):
                                self.next()
                                fns.append(self.name())
                            self.expect(":")
                            ft = self.type_()
                            fields += [(f, ft) for f in fns]
                            if self.at(","):
                                self.next()
                        self.expect(")")
                    variants.append((vn, fields))
                    self.end_line()
                    self.skip_newlines()
                if self.at_kind("DEDENT"):
                    self.next()
                return N("enum", line, name=nm, variants=variants, decos=[])
            if v == "match":
                self.next()
                subj = self.expr()
                self.expect(":")
                self.expect_kind("NEWLINE")
                self.expect_kind("INDENT")
                arms = []
                self.skip_newlines()
                while not self.at_kind("DEDENT") and not self.at_kind("EOF"):
                    an = self.name()
                    binds = []
                    if self.at("("):
                        self.next()
                        while not self.at(")"):
                            binds.append(self.name())
                            if self.at(","):
                                self.next()
                        self.expect(")")
                    guard = None
                    if self.at("if"):
                        self.next()
                        self.no_lambda = 1
                        guard = self.expr()
                        self.no_lambda = 0
                    if self.at("=>"):
                        self.t.val = ":"
                    body = self.block()
                    arms.append(N("arm", line, name=an, binds=binds, guard=guard, body=body))
                    self.skip_newlines()
                if self.at_kind("DEDENT"):
                    self.next()
                return N("match", line, subject=subj, arms=arms)
            if v == "if":
                return self.if_stmt()
            if v == "for":
                self.next()
                pat = self.pattern()
                self.expect("in")
                it = self.expr()
                body = self.block()
                return N("for", line, pat=pat, iter=it, body=body)
            if v == "while":
                self.next()
                if self.at("let"):
                    self.next()
                    pat = self.pattern()
                    self.expect("=")
                    e = self.expr()
                    body = self.block()
                    return N("whilelet", line, pat=pat, value=e, body=body)
                c = self.expr()
                body = self.block()
                return N("while", line, cond=c, body=body)
        s = self.simple_stmt()
        self.end_line()
        return s

    def simple_stmt(self):
        t = self.t
        line = t.line
        if t.kind == "KW":
            v = t.val
            if v in ("let", "var"):
                self.next()
                pat = self.pattern()
                ty = None
                if self.at(":"):
                    self.next()
                    ty = self.type_()
                val = None
                if self.at("="):
                    self.next()
                    val = self.expr()
                if v == "let" and val is None:
                    self.err("let needs a value")
                where = None
                if self.at("where"):
                    self.next()
                    where = self.expr()
                els = None
                if self.at("else"):
                    self.next()
                    els = self.block()
                    self.after_block = True
                return N("let", line, pat=pat, ty=ty, value=val, mut=(v == "var"), const=False, where=where, orelse=els)
            if v == "yield":
                self.next()
                if self.t.kind == "NAME" and self.t.val == "from":
                    self.next()
                    return N("yieldfrom", line, value=self.expr())
                return N("yield", line, value=self.expr())
            if v == "return":
                self.next()
                e = None
                if not (self.at_kind("NEWLINE") or self.at_kind("DEDENT") or self.at(";")):
                    e = self.expr()
                return N("return", line, value=e)
            if v == "break":
                self.next()
                return N("break", line)
            if v == "continue":
                self.next()
                return N("continue", line)
            if v == "assert":
                self.next()
                e = self.expr()
                return N("assert", line, cond=e)
        if t.kind == "NAME" and t.val == "lend" and (self.peek().kind in ("NAME", "NUM") or
                                                     (self.peek().kind == "OP" and self.peek().val == "(")):
            # `lend place` / `lend mut place`: the one exit of a projection (plan/19 §3.4)
            self.next()
            mut = False
            if self.t.kind == "NAME" and self.t.val == "mut" and self.peek().kind in ("NAME", "NUM"):
                self.next()
                mut = True
            return N("lend", line, value=self.expr(), mut=mut)
        if t.kind == "NAME" and t.val == "law" and self.peek().kind == "NAME":
            # `law disjoint(i, j) = i != j`: when two lends of this projection don't overlap (plan/19 §3.6)
            self.next()
            nm = self.name()
            self.expect("(")
            ps = []
            while not self.at(")"):
                ps.append(self.name())
                if self.at(","):
                    self.next()
            self.expect(")")
            self.expect("=")
            return N("law", line, name=nm, params=ps, expr=self.expr())
        e = self.expr()
        if self.t.kind == "OP" and self.t.val in ASSIGN_OPS:
            op = self.next().val
            rhs = self.expr()
            return N("assign", line, target=e, op=op, value=rhs)
        if self.t.kind == "NAME" and self.t.val in ("min", "max") and self.peek().kind == "OP" and self.peek().val == "=":
            op = self.next().val + "="
            self.next()
            rhs = self.expr()
            return N("assign", line, target=e, op=op, value=rhs)
        return N("exprstmt", line, expr=e)

    def pattern(self, bare=True):
        line = self.t.line
        if self.at("("):
            self.next()
            names = [self.pattern(False)]
            while self.at(","):
                self.next()
                names.append(self.pattern(False))
            self.expect(")")
            return N("ptuple", line, items=names)
        nm = self.name()
        if bare and self.at(","):
            items = [N("pname", line, name=nm)]
            while self.at(","):
                self.next()
                items.append(N("pname", line, name=self.name()))
            return N("ptuple", line, items=items)
        return N("pname", line, name=nm)

    def input_stmt(self):
        line = self.next().line
        mut = False
        if self.at("var"):
            self.next()
            mut = True
        groups = []
        while True:
            names = [self.name()]
            while self.at(","):
                self.next()
                names.append(self.name())
            self.expect(":")
            ty = self.type_()
            groups.append((names, ty))
            if self.at(","):
                self.next()
                continue
            break
        where = None
        if self.at("where"):
            self.next()
            where = self.expr()
        self.end_line()
        return N("input", line, groups=groups, where=where, mut=mut)

    def params(self):
        self.expect("(")
        ps = []
        pending = []
        while not self.at(")"):
            mode = None
            if self.at("inout") or self.at("ref"):
                mode = self.next().val
            elif self.t.kind == "NAME" and self.t.val == "mut" and self.peek().kind == "NAME":
                self.next()
                mode = "inout"
            nm = self.name()
            if self.at(":"):
                self.next()
                ty = self.type_()
                for pn, pm in pending + [(nm, mode)]:
                    ps.append(N("param", self.t.line, name=pn, ty=ty, mode=pm))
                pending = []
            else:
                pending.append((nm, mode))
            if self.at(","):
                self.next()
        self.expect(")")
        for pn, pm in pending:
            ps.append(N("param", self.t.line, name=pn, ty=None, mode=pm))
        return ps

    def fn_def(self):
        line = self.next().line
        nm = self.name()
        tparams = []
        if self.at("<"):
            self.next()
            while not self.at_close_angle():
                tparams.append(self.name())
                if self.at(":"):
                    self.next()
                    self.type_()
                if self.at(","):
                    self.next()
            self.close_angle()
        ps = self.params()
        ret = None
        lends = None
        if self.t.kind == "NAME" and self.t.val == "lends":
            # a projection (plan/19 §3.4): `fn row(self, r: A) lends mut [B]T`
            self.next()
            lends = "read"
            if self.t.kind == "NAME" and self.t.val == "mut":
                self.next()
                lends = "mut"
            ret = self.type_()
        elif self.at("->"):
            self.next()
            if self.t.kind == "NAME" and self.t.val == "yields":
                yl = self.next().line
                ret = N("tyields", yl, inner=self.type_())
            else:
                ret = self.type_()
        if self.at("="):
            # `fn to_idx(self) -> Int = r.ord * w + c.ord`: a one-expression body
            self.next()
            body = [self.simple_stmt()]
            self.end_line()
        else:
            body = self.block()
        return N("fn", line, name=nm, tparams=tparams, params=ps, ret=ret, body=body, decos=[], pure=False, lends=lends)

    def struct_def(self):
        line = self.next().line
        nm = self.name()
        self.expect(":")
        self.expect_kind("NEWLINE")
        self.expect_kind("INDENT")
        fields, methods = [], []
        self.skip_newlines()
        while not self.at_kind("DEDENT"):
            if self.at("fn"):
                methods.append(self.fn_def())
            else:
                names = [self.name()]
                while self.at(","):
                    self.next()
                    names.append(self.name())
                self.expect(":")
                ty = self.type_()
                for f in names:
                    fields.append((f, ty))
                self.end_line()
            self.skip_newlines()
        self.next()
        return N("struct", line, name=nm, fields=fields, methods=methods, decos=[])

    def if_stmt(self):
        line = self.next().line
        if self.at("let"):
            self.next()
            pat = self.pattern()
            self.expect("=")
            e = self.not_()
            if self.at("??"):
                self.next()
                e = N("binop", line, op="??", a=e, b=self.not_())
            extra = None
            if self.at("and"):
                self.next()
                extra = self.expr()
            body = self.block()
            els = self.else_part()
            return N("iflet", line, pat=pat, value=e, extra=extra, body=body, orelse=els)
        c = self.expr()
        body = self.block()
        els = self.else_part()
        return N("if", line, cond=c, body=body, orelse=els)

    def else_part(self):
        self.skip_newlines_before_else()
        if self.at("elif"):
            self.toks[self.i].val = "if"
            return [self.if_stmt()]
        if self.at("else"):
            self.next()
            return self.block()
        return []

    def skip_newlines_before_else(self):
        j = self.i
        while self.toks[j].kind == "NEWLINE":
            j += 1
        if self.toks[j].kind in ("KW",) and self.toks[j].val in ("elif", "else"):
            self.i = j

    # ---- types
    def type_(self):
        t = self.t
        line = t.line
        if self.at("["):
            self.next()
            if self.at("]"):
                self.next()
                return N("tlist", line, elem=self.type_())
            dims = [self.type_or_domain()]
            while self.at(","):
                self.next()
                dims.append(self.type_or_domain())
            self.expect("]")
            elem = self.type_()
            return N("tarray", line, dims=dims, elem=elem)
        if self.at("?"):
            self.next()
            return N("topt", line, inner=self.type_())
        if self.at("("):
            self.next()
            fields = []
            pending = []
            while not self.at(")"):
                if self.t.kind == "NAME" and self.peek().kind == "OP" and self.peek().val in (":", ","):
                    # could be named fields `a, b: T` or positional types
                    j = self.i
                    names = []
                    ok = False
                    while self.toks[j].kind == "NAME":
                        names.append(self.toks[j].val)
                        if self.toks[j + 1].kind == "OP" and self.toks[j + 1].val == ":":
                            ok = True
                            j += 2
                            break
                        if self.toks[j + 1].kind == "OP" and self.toks[j + 1].val == ",":
                            j += 2
                            continue
                        break
                    if ok:
                        self.i = j
                        ty = self.type_()
                        for nm in names:
                            fields.append((nm, ty))
                        if self.at(","):
                            self.next()
                        continue
                fields.append((None, self.type_()))
                if self.at(","):
                    self.next()
            self.expect(")")
            return N("ttuple", line, fields=fields)
        if t.kind == "KW" and t.val == "domain":
            self.next()
            nm = self.name()
            rng = None
            if self.at("="):
                self.next()
                rng = self.expr()
            return N("tfresh", line, name=nm, range=rng)
        if t.kind == "NAME" and not (self.peek().kind == "OP" and self.peek().val in ("..", "..=", "+", "-", "*", "/", "(", ".")):
            self.next()
            args = []
            if self.at("<"):
                self.next()
                while not self.at_close_angle():
                    args.append(self.type_or_expr())
                    if self.at(","):
                        self.next()
                self.close_angle()
            return N("tname", line, name=t.val, args=args)
        e = self.range_expr()
        if e.kind != "range":
            return N("texpr", line, expr=e)
        return N("trange", line, range=e)

    def type_or_domain(self):
        return self.type_()

    def type_or_expr(self):
        if self.t.kind == "NUM":
            return self.range_expr()
        return self.type_()

    # ---- expressions
    def expr(self):
        t = self.t
        line = t.line
        if self.at("if"):
            self.next()
            c = self.expr()
            self.expect(":")
            a = self.expr()
            self.skip_newlines_before_else()
            if self.at("elif"):
                self.toks[self.i].val = "if"
                b = self.expr()
            else:
                self.expect("else")
                self.expect(":")
                b = self.expr()
            return N("ifexpr", line, cond=c, a=a, b=b)
        if self.at("{"):
            line = self.next().line
            body = [self.simple_stmt()]
            while self.at(";"):
                self.next()
                body.append(self.simple_stmt())
            self.expect("}")
            return N("doblock", line, body=body)
        if self.t.kind == "NAME" and self.t.val == "do" and self.peek().kind == "OP" and self.peek().val == ":":
            line = self.next().line
            body = self.block()
            self.after_block = True
            return N("doblock", line, body=body)
        if self.at("|"):
            self.next()
            ps = []
            while not self.at("|"):
                ps.append(self.pattern())
                if self.at(","):
                    self.next()
            self.next()
            body = self.expr()
            return N("lambda", line, params=ps, body=body)
        return self.coalesce()

    def coalesce(self):
        e = self.or_()
        while self.at("??"):
            line = self.next().line
            e = N("binop", line, op="??", a=e, b=self.or_())
        return e

    def or_(self):
        e = self.and_()
        while self.at("or"):
            line = self.next().line
            e = N("binop", line, op="or", a=e, b=self.and_())
        return e

    def and_(self):
        e = self.not_()
        while self.at("and"):
            line = self.next().line
            e = N("binop", line, op="and", a=e, b=self.not_())
        return e

    def not_(self):
        if self.at("not"):
            line = self.next().line
            return N("unop", line, op="not", a=self.not_())
        return self.cmp()

    def cmp(self):
        e = self.range_expr()
        ops, rest = [], []
        while True:
            if self.t.kind == "OP" and self.t.val in CMP_OPS:
                ops.append(self.next().val)
            elif self.at("in"):
                self.next()
                ops.append("in")
            elif self.at("not") and self.peek().kind == "KW" and self.peek().val == "in":
                self.next(); self.next()
                ops.append("not in")
            else:
                break
            rest.append(self.range_expr())
        if not ops:
            return e
        return N("cmp", e.line, first=e, ops=ops, rest=rest)

    def range_expr(self):
        e = self.bitor()
        if self.at("..", "..="):
            line = self.t.line
            incl = self.next().val == "..="
            hi = None
            if not self.ends_range():
                hi = self.bitor()
            return N("range", line, lo=e, hi=hi, incl=incl)
        return e

    def ends_range(self):
        t = self.t
        return t.kind in ("NEWLINE", "EOF", "DEDENT") or (t.kind == "OP" and t.val in (")", "]", ",", ":")) or (
            t.kind == "KW" and t.val in ("where", "if", "for"))

    def bitor(self):
        e = self.bitxor()
        while self.at("|") and not self.no_bitor:
            line = self.next().line
            e = N("binop", line, op="|", a=e, b=self.bitxor())
        return e
    no_bitor = False

    def bitxor(self):
        e = self.bitand()
        while self.at("^"):
            line = self.next().line
            e = N("binop", line, op="^", a=e, b=self.bitand())
        return e

    def bitand(self):
        e = self.shift()
        while self.at("&"):
            line = self.next().line
            e = N("binop", line, op="&", a=e, b=self.shift())
        return e

    def shift(self):
        e = self.add()
        while self.at("<<", ">>"):
            t = self.next()
            e = N("binop", t.line, op=t.val, a=e, b=self.add())
        return e

    def add(self):
        e = self.mul()
        while self.at("+", "-"):
            t = self.next()
            e = N("binop", t.line, op=t.val, a=e, b=self.mul())
        return e

    def mul(self):
        e = self.unary()
        while self.at("*", "/", "%"):
            t = self.next()
            e = N("binop", t.line, op=t.val, a=e, b=self.unary())
        return e

    def unary(self):
        if self.at("-"):
            line = self.next().line
            return N("unop", line, op="-", a=self.unary())
        return self.power()

    def power(self):
        e = self.postfix()
        if self.at("**"):
            line = self.next().line
            return N("binop", line, op="**", a=e, b=self.unary())
        return e

    def call_args(self):
        self.expect("(")
        args = []
        while not self.at(")"):
            line = self.t.line
            if self.t.kind in ("NAME", "KW") and self.peek().kind == "OP" and self.peek().val == ":" and self.t.kind == "NAME":
                kw = self.next().val
                self.next()
                args.append(N("kwarg", line, name=kw, value=self.expr()))
            else:
                mut = self.t.kind == "NAME" and self.t.val == "mut" and self.peek().kind == "NAME"
                if mut:
                    self.next()
                e = self.expr()
                if self.at("for"):
                    e = self.gen_tail(e)
                if mut:
                    e.mut = True
                args.append(e)
            if self.at(","):
                self.next()
        self.expect(")")
        return args

    def gen_tail(self, e):
        clauses = []
        line = e.line
        while self.at("for", "if"):
            if self.at("for"):
                self.next()
                pat = self.pattern()
                self.expect("in")
                it = self.range_expr()
                clauses.append(("for", pat, it))
            else:
                self.next()
                if self.at("let"):
                    self.next()
                    pat = self.pattern()
                    self.expect("=")
                    clauses.append(("iflet", pat, self.or_()))
                else:
                    clauses.append(("if", self.or_()))
        return N("gen", line, elem=e, clauses=clauses)

    def postfix(self):
        e = self.atom()
        while True:
            if self.at("("):
                args = self.call_args()
                e = N("call", e.line, fn=e, args=args)
            elif self.at("["):
                line = self.next().line
                idx = [self.expr()]
                while self.at(","):
                    self.next()
                    idx.append(self.expr())
                self.expect("]")
                e = N("index", line, obj=e, idx=idx)
            elif self.at("."):
                line = self.next().line
                if self.t.kind == "NUM":
                    e = N("field", line, obj=e, name=str(self.next().val))
                else:
                    t = self.next()
                    e = N("field", line, obj=e, name=t.val)
            elif self.at("!"):
                line = self.next().line
                e = N("unwrap", line, a=e)
            elif self.at("?"):
                line = self.next().line
                e = N("try", line, a=e)
            elif self.at("<") and e.kind == "name" and e.name[:1].isupper() and self.looks_generic():
                self.next()
                args = []
                while not self.at_close_angle():
                    args.append(self.type_or_expr())
                    if self.at(","):
                        self.next()
                self.close_angle()
                e = N("generic", e.line, name=e.name, args=args)
            else:
                return e

    def looks_generic(self):
        j, d = self.i, 0
        while j < len(self.toks):
            t = self.toks[j]
            if t.kind == "OP" and t.val == "<":
                d += 1
            elif t.kind == "OP" and t.val in (">", ">>"):
                d -= len(t.val)
                if d <= 0:
                    nx = self.toks[j + 1]
                    return nx.kind == "OP" and nx.val in ("(", ".")
            elif t.kind in ("NEWLINE", "EOF") or (t.kind == "OP" and t.val in ("=", ":")):
                return False
            j += 1
        return False

    def paren_lambda_ahead(self):
        j, d = self.i, 0
        while j < len(self.toks):
            t = self.toks[j]
            if t.kind == "OP" and t.val in ("(", "[", "{"):
                d += 1
            elif t.kind == "OP" and t.val in (")", "]", "}"):
                d -= 1
                if d == 0:
                    nx = self.toks[j + 1]
                    return nx.kind == "OP" and nx.val == "=>"
            elif t.kind in ("NEWLINE", "EOF"):
                return False
            j += 1
        return False

    def atom(self):
        t = self.t
        line = t.line
        # lambdas: `x => e` and `(a, b) => e`
        if not getattr(self, "no_lambda", 0) and ((t.kind == "NAME" and self.peek().kind == "OP" and self.peek().val == "=>") or
                (self.at("(") and self.paren_lambda_ahead())):
            if self.at("("):
                self.next()
                ps = []
                while not self.at(")"):
                    ps.append(self.pattern(False))
                    if self.at(","):
                        self.next()
                self.next()
            else:
                ps = [self.pattern(False)]
            self.expect("=>")
            return N("lambda", line, params=ps, body=self.expr())
        if t.kind == "NUM":
            self.next()
            return N("num", line, value=t.val)
        if t.kind == "STR":
            self.next()
            return N("str", line, value=t.val)
        if t.kind == "CHAR":
            self.next()
            return N("char", line, value=t.val)
        if t.kind == "NAME":
            self.next()
            return N("name", line, name=t.val)
        if t.kind == "KW":
            if t.val in ("true", "false"):
                self.next()
                return N("bool", line, value=t.val == "true")
            if t.val == "none":
                self.next()
                return N("none", line)
        if self.at("("):
            self.next()
            if self.at(")"):
                self.next()
                return N("tuple", line, items=[], names=[])
            items, names = [], []
            while True:
                nm = None
                if self.t.kind == "NAME" and self.peek().kind == "OP" and self.peek().val == ":":
                    nm = self.next().val
                    self.next()
                e = self.expr()
                if self.at("for"):
                    e = self.gen_tail(e)
                items.append(e)
                names.append(nm)
                if self.at(","):
                    self.next()
                    if self.at(")"):
                        break
                    continue
                break
            self.expect(")")
            if len(items) == 1 and names[0] is None:
                return N("paren", line, a=items[0])
            return N("tuple", line, items=items, names=names)
        if self.at("["):
            self.next()
            items = []
            while not self.at("]"):
                items.append(self.expr())
                if self.at("for"):
                    items[-1] = self.gen_tail(items[-1])
                if self.at(","):
                    self.next()
            self.expect("]")
            # table comprehension: [D] i => e   /  [A, B] (i, j) => e
            if (self.t.kind == "NAME" and self.peek().kind == "OP" and self.peek().val == "=>") or self.at("("):
                save = self.i
                try:
                    pat = self.pattern()
                    if self.at("=>"):
                        self.next()
                        body = self.expr()
                        return N("table", line, dims=items, pat=pat, body=body)
                except ParseError:
                    pass
                self.i = save
            if self.at("?") or (self.t.kind == "NAME" and self.t.val[:1].isupper()) or self.at("["):
                # an array type used as an expression, e.g. [V]Int(0)
                pass
            return N("list", line, items=items)
        self.err("expected an expression")

def parse(src):
    return Parser(src).program()
