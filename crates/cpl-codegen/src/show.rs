//! Printing and reading values, as C++.

use crate::emit::{pad, Code, Emitter, PRIMARY, UNARY};
use crate::expr::string_literal;
use crate::names::{elem_ty, ident, is_plain_enum};
use cpl_ir::{sep_of, DimSpec, FmtArg, Sep, Spec, Ty};

fn sep_char(sep: Sep) -> &'static str {
    match sep {
        Sep::Line => "'\\n'",
        _ => "' '",
    }
}

impl<'p> Emitter<'p> {
    /// One statement that writes the literal `bytes`.
    pub fn put_text(&mut self, bytes: &[u8]) -> String {
        self.need("out");
        if bytes.contains(&0) {
            format!(
                "cpl::put_str(std::string({}, {}));",
                string_literal(bytes),
                bytes.len()
            )
        } else {
            format!("cpl::put_str({});", string_literal(bytes))
        }
    }

    /// One statement that writes `value` the way `print` shows a `ty`.
    pub fn show_stmt(&mut self, value: &Code, ty: &Ty, digits: &str) -> String {
        let program = self.program;
        self.need("out");
        match ty {
            Ty::Int => {
                self.need("put_int");
                format!("cpl::put_int({});", value.text)
            }
            Ty::I128 => {
                self.need("put_i128");
                format!("cpl::put_i128({});", value.text)
            }
            Ty::U64 => {
                self.need("put_u64");
                format!("cpl::put_u64({});", value.text)
            }
            Ty::Pos(d) | Ty::Bound(d) => {
                self.need("put_int");
                format!(
                    "cpl::put_int({} + {}_lo);",
                    value.at(UNARY),
                    ident(&program.domains[*d])
                )
            }
            Ty::Float => {
                self.need("put_float");
                format!("cpl::put_float({}, {digits});", value.text)
            }
            Ty::Bool => format!("cpl::put_str({} ? \"YES\" : \"NO\");", value.at(UNARY)),
            Ty::Char => format!("cpl::put_char({});", value.text),
            Ty::Str => format!("cpl::put_str({});", value.text),
            Ty::StrView => {
                self.need("put_str_view");
                format!("cpl::put_str({});", value.text)
            }
            Ty::Unit | Ty::Error => format!("{};", value.text),
            _ => {
                let helper = self.show_helper(ty);
                format!("{helper}({}, {digits});", value.text)
            }
        }
    }

    /// The name of the fn that prints a value of the compound type `ty`.
    fn show_helper(&mut self, ty: &Ty) -> String {
        if let Some(index) = self.show_index.get(ty) {
            return format!("show_{}", index + 1);
        }
        // What `ty` is made of first, so a helper follows the ones it
        // calls. A type that holds itself cannot: its helpers are declared
        // ahead of them all.
        if self.showing.insert(ty.clone()) {
            for part in self.parts(ty) {
                if !part.is_scalar() && !matches!(part, Ty::Str | Ty::StrView) {
                    self.show_helper(&part);
                }
            }
            self.showing.remove(ty);
        } else {
            self.show_cycle = true;
        }
        if let Some(index) = self.show_index.get(ty) {
            return format!("show_{}", index + 1);
        }
        let index = self.shows.len();
        self.show_index.insert(ty.clone(), index);
        self.shows.push(String::new());
        let body = self.show_body(ty);
        let digits = if body.contains("digits") {
            "int digits"
        } else {
            "int"
        };
        let rest = format!("(const {}& v, {digits}) {{\n{body}}}\n", self.ty(ty));
        // Two types that are one in C++ and print alike share a helper.
        let twin = self.shows[..index]
            .iter()
            .position(|other| other.ends_with(&rest));
        if let (Some(twin), true) = (twin, index + 1 == self.shows.len()) {
            self.shows.pop();
            self.show_index.insert(ty.clone(), twin);
            return format!("show_{}", twin + 1);
        }
        self.shows[index] = format!(
            "// Prints a `{}`.\nvoid show_{}{rest}",
            self.program.show_ty(ty),
            index + 1
        );
        format!("show_{}", index + 1)
    }

    /// The types a value of the compound type `ty` is made of.
    fn parts(&self, ty: &Ty) -> Vec<Ty> {
        match ty {
            Ty::Opt(inner)
            | Ty::List(inner)
            | Ty::ListView(inner, _)
            | Ty::Keyed(inner, _)
            | Ty::Set(inner)
            | Ty::HashSet(inner) => {
                vec![(**inner).clone()]
            }
            Ty::Array { elem, .. } => vec![(**elem).clone()],
            Ty::Map(key, value) | Ty::HashMap(key, value) => {
                vec![(**key).clone(), (**value).clone()]
            }
            Ty::Struct(id) => {
                let def = &self.program.structs[*id];
                if def.show_fn.is_some() {
                    Vec::new()
                } else {
                    def.fields
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !def.cached_fields.contains(i))
                        .map(|(_, (_, t))| t.clone())
                        .collect()
                }
            }
            Ty::Enum(id) => self.program.enums[*id]
                .variants
                .iter()
                .flat_map(|(_, tys)| tys.iter().cloned())
                .collect(),
            _ => ty.fields().unwrap_or_default(),
        }
    }

    fn show_body(&mut self, ty: &Ty) -> String {
        let program = self.program;
        let mut b = String::new();
        let fields = |this: &mut Self, items: Vec<(Code, &Ty)>, lead: &str| -> String {
            let mut out = String::new();
            for (i, (code, field_ty)) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(&format!("{lead}cpl::put_char(' ');\n"));
                }
                out.push_str(&format!(
                    "{lead}{}\n",
                    this.show_stmt(code, field_ty, "digits")
                ));
            }
            out
        };
        match ty {
            Ty::Opt(inner) => {
                let some = self.show_stmt(&Code::new("*v", UNARY), inner, "digits");
                b.push_str(&format!(
                    "    if (v) {{ {some} }} else cpl::put_str(\"none\");\n"
                ));
            }
            Ty::Tuple(_) | Ty::Record(_) => {
                let tys = ty.fields().unwrap_or_default();
                let items: Vec<(Code, &Ty)> = tys
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (Code::primary(format!("std::get<{i}>(v)")), t))
                    .collect();
                b.push_str(&fields(self, items, "    "));
            }
            Ty::Struct(id) => {
                let def = &program.structs[*id];
                if let Some(show_fn) = def.show_fn {
                    let name = crate::names::fn_name(&program.fns[show_fn].name);
                    b.push_str(&format!("    cpl::put_str({name}(v));\n"));
                } else {
                    let items: Vec<(Code, &Ty)> = def
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !def.cached_fields.contains(i))
                        .map(|(_, (name, t))| (Code::primary(format!("v.{}", ident(name))), t))
                        .collect();
                    b.push_str(&fields(self, items, "    "));
                }
            }
            Ty::Enum(id) if is_plain_enum(program, *id) => {
                let def = &program.enums[*id];
                let names: Vec<String> = (0..def.variants.len())
                    .map(|tag| string_literal(def.token(tag).as_bytes()))
                    .collect();
                b.push_str(&format!(
                    "    static const char* const names[] = {{{}}};\n",
                    names.join(", ")
                ));
                b.push_str("    cpl::put_str(names[(int)v]);\n");
            }
            Ty::Enum(id) => {
                let def = &program.enums[*id];
                b.push_str("    switch (v.tag) {\n");
                for (tag, (variant, tys)) in def.variants.iter().enumerate() {
                    let name = ident(variant);
                    b.push_str(&format!(
                        "    case {}::{name}:\n        cpl::put_str({});\n",
                        ident(&def.name),
                        string_literal(def.token(tag).as_bytes())
                    ));
                    for (i, field_ty) in tys.iter().enumerate() {
                        let code = Code::primary(format!("v.{name}_{i}"));
                        b.push_str(&format!(
                            "        cpl::put_char(' ');\n        {}\n",
                            self.show_stmt(&code, field_ty, "digits")
                        ));
                    }
                    b.push_str("        break;\n");
                }
                b.push_str("    }\n");
            }
            Ty::Array { dims, elem } if dims.len() > 1 => {
                let cell = self.cell("v.d[i]", elem);
                let shown = self.show_stmt(&cell, elem, "digits");
                b.push_str(&format!(
                    "    size_t row = (size_t)std::max<ll>(v.n[{}], 1);\n",
                    dims.len() - 1
                ));
                b.push_str("    for (size_t i = 0; i < v.d.size(); i++) {\n");
                b.push_str(match sep_of(elem, &self.program.structs) {
                    Sep::None => "        if (i > 0 && i % row == 0) cpl::put_char('\\n');\n",
                    Sep::Space => "        if (i > 0) cpl::put_char(i % row == 0 ? '\\n' : ' ');\n",
                    Sep::Line => "        if (i > 0) cpl::put_char('\\n');\n",
                });
                b.push_str(&format!("        {shown}\n    }}\n"));
            }
            Ty::Array { elem, .. }
            | Ty::List(elem)
            | Ty::ListView(elem, _)
            | Ty::Keyed(elem, _)
            | Ty::MutView(elem) => {
                let cell = self.cell("v[i]", elem);
                let shown = self.show_stmt(&cell, elem, "digits");
                b.push_str("    for (size_t i = 0; i < v.size(); i++) {\n");
                let sep = sep_of(elem, &self.program.structs);
                if sep != Sep::None {
                    b.push_str(&format!(
                        "        if (i > 0) cpl::put_char({});\n",
                        sep_char(sep)
                    ));
                }
                b.push_str(&format!("        {shown}\n    }}\n"));
            }
            Ty::Set(elem) | Ty::HashSet(elem) => {
                let shown = self.show_stmt(&Code::primary("x"), elem, "digits");
                b.push_str("    bool first = true;\n    for (const auto& x : v) {\n");
                b.push_str(&format!(
                    "        if (!first) cpl::put_char({});\n        first = false;\n",
                    sep_char(sep_of(elem, &self.program.structs))
                ));
                b.push_str(&format!("        {shown}\n    }}\n"));
            }
            Ty::Map(key, value) | Ty::HashMap(key, value) => {
                let shown_key = self.show_stmt(&Code::primary("x.first"), key, "digits");
                let shown_value = self.show_stmt(&Code::primary("x.second"), value, "digits");
                b.push_str("    bool first = true;\n    for (const auto& x : v) {\n");
                b.push_str("        if (!first) cpl::put_char('\\n');\n        first = false;\n");
                b.push_str(&format!("        {shown_key}\n        cpl::put_char(' ');\n        {shown_value}\n    }}\n"));
            }
            Ty::Range(inner) => {
                if let Ty::Pos(d) = **inner {
                    let dlo = format!("{}_lo", ident(&program.domains[d]));
                    self.need("put_int");
                    b.push_str(&format!(
                        "    cpl::put_int((ll)v.lo + {dlo}); cpl::put_str(\"..\"); cpl::put_int((ll)v.hi + {dlo});\n"
                    ));
                } else {
                    self.need("put_int");
                    b.push_str(
                        "    cpl::put_int(v.lo); cpl::put_str(\"..\"); cpl::put_int(v.hi);\n",
                    );
                }
            }
            _ => {}
        }
        b
    }

    /// An element of a vector, as its own type: a `Bool` is stored as a byte.
    fn cell(&self, text: &str, elem: &Ty) -> Code {
        if *elem == Ty::Bool {
            Code::new(format!("(bool){text}"), UNARY)
        } else {
            Code::new(text, PRIMARY)
        }
    }

    fn show_expr(&mut self, e: &cpl_ir::Expr, digits: &str) -> String {
        if let cpl_ir::ExprKind::Builtin(cpl_ir::Builtin::StrConcat, args) = &e.kind {
            if args.len() == 2 && e.ty == Ty::Str {
                let a = self.show_expr(&args[0], digits);
                let b = self.show_expr(&args[1], digits);
                return format!("{a} {b}");
            }
        }
        let code = self.expr(e);
        self.show_stmt(&code, &e.ty, digits)
    }

    /// `print(a, b)` as one line of statements.
    pub fn print_stmts(&mut self, args: &[FmtArg], end: Option<&cpl_ir::Expr>, out: &mut String) {
        self.need("out");
        let mut parts = Vec::new();
        // An empty collection shows nothing and takes no space, so with
        // one among the arguments a space goes only between two that
        // showed something: `start` is where the line began.
        let sparse = args.len() > 1 && args.iter().any(|arg| arg.value.ty.may_show_nothing());
        if sparse {
            parts.push("{ size_t start = cpl::out.size();".to_string());
        }
        for (i, arg) in args.iter().enumerate() {
            let shown = self.show_expr(&arg.value, &arg.digits.to_string());
            match (i > 0, sparse) {
                (false, _) => parts.push(shown),
                (true, false) => parts.push(format!("cpl::put_char(' '); {shown}")),
                (true, true) if arg.value.ty.may_show_nothing() => parts.push(format!(
                    "{{ size_t at = cpl::out.size(); if (at > start) cpl::put_char(' '); size_t from = cpl::out.size(); {shown} if (cpl::out.size() == from) cpl::out.resize(at); }}"
                )),
                (true, true) => parts.push(format!("if (cpl::out.size() > start) cpl::put_char(' '); {shown}")),
            }
        }
        if sparse {
            parts.push("}".to_string());
        }
        match end {
            Some(end) => {
                let code = self.expr(end);
                parts.push(self.show_stmt(&code, &Ty::Str, "6"));
            }
            None => parts.push("cpl::put_line();".to_string()),
        }
        if self.program.interactive {
            parts.push("cpl::flush(); fflush(stdout);".to_string());
        }
        out.push_str(&format!("{}{}\n", pad(self.indent), parts.join(" ")));
    }

    /// `read()` of the type `spec` spells out.
    pub fn read(&mut self, spec: &Spec) -> Code {
        let program = self.program;
        let plain = |this: &mut Self, name: &'static str| {
            this.need(name);
            Code::primary(format!("cpl::{name}()"))
        };
        match spec {
            Spec::Line => plain(self, "read_line"),
            Spec::Int {
                lo: None, hi: None, ..
            }
            | Spec::Plain(Ty::Int) => plain(self, "read_int"),
            Spec::Int { lo, hi, .. } => {
                self.need("read_in");
                let lo = lo
                    .as_ref()
                    .map_or("INT64_MIN".to_string(), |e| self.expr(e).text);
                let hi = hi
                    .as_ref()
                    .map_or("INT64_MAX".to_string(), |e| self.expr(e).text);
                Code::primary(format!("cpl::read_in({lo}, {hi})"))
            }
            Spec::Plain(Ty::I128) => plain(self, "read_i128"),
            Spec::Plain(Ty::U64) => plain(self, "read_u64"),
            Spec::Plain(Ty::Float) => plain(self, "read_float"),
            Spec::Plain(Ty::Char) => plain(self, "read_char"),
            Spec::Plain(Ty::Bool) => plain(self, "read_bool"),
            Spec::Plain(Ty::Str) => plain(self, "read_str"),
            Spec::Plain(Ty::Pos(d)) => {
                // A position from the input is checked like `V(x)`: nothing
                // later tests an index of its domain.
                self.need("read_int");
                self.need("at");
                let d = ident(&program.domains[*d]);
                Code::primary(format!("cpl::pos(cpl::read_int() - {d}_lo, {d}_len)"))
            }
            Spec::Plain(Ty::Range(inner)) if matches!(**inner, Ty::Int | Ty::Pos(_)) => {
                self.need("read_int");
                self.need("fail");
                self.need("range");
                let ty = self.ty(&spec.ty());
                let dom_ident = match &**inner {
                    Ty::Pos(d) => {
                        self.need("at");
                        Some(ident(&program.domains[*d]))
                    }
                    _ => None,
                };
                self.lambda(&ty, |this, out| {
                    let lead = pad(this.indent);
                    if let Some(d) = dom_ident {
                        out.push_str(&format!(
                            "{lead}int l = cpl::pos(cpl::read_int() - {d}_lo, {d}_len);\n"
                        ));
                        out.push_str(&format!(
                            "{lead}int r = cpl::pos(cpl::read_int() - {d}_lo, {d}_len);\n"
                        ));
                        out.push_str(&format!(
                            "{lead}if (l > r) cpl::fail(\"range end is before start\");\n"
                        ));
                        out.push_str(&format!("{lead}return {ty}{{l, r + 1}};\n"));
                    } else {
                        out.push_str(&format!("{lead}ll l = cpl::read_int();\n"));
                        out.push_str(&format!("{lead}ll r = cpl::read_int();\n"));
                        out.push_str(&format!(
                            "{lead}if (l > r) cpl::fail(\"range end is before start\");\n"
                        ));
                        out.push_str(&format!("{lead}return {ty}{{l, r + 1}};\n"));
                    }
                })
            }
            Spec::Plain(ty) => Code::primary(format!("{}()", self.ty(ty))),
            // `read(none: 0)`: the number, or `none` where the input has 0.
            Spec::Opt(inner, Some(absent)) => {
                self.need("read_int");
                let ty = self.ty(&spec.ty());
                let absent = self.expr(absent).text;
                let value = match &**inner {
                    Spec::Plain(Ty::Pos(d)) => {
                        self.need("at");
                        let d = ident(&program.domains[*d]);
                        format!("cpl::pos(x - {d}_lo, {d}_len)")
                    }
                    _ => "x".to_string(),
                };
                self.lambda(&ty, |this, out| {
                    let lead = pad(this.indent);
                    out.push_str(&format!("{lead}ll x = cpl::read_int();\n"));
                    out.push_str(&format!("{lead}if (x == {absent}) return {ty}();\n"));
                    out.push_str(&format!("{lead}return {ty}({value});\n"));
                })
            }
            Spec::Opt(..) => Code::primary(format!("{}()", self.ty(&spec.ty()))),
            Spec::Call(call) => self.expr(call),
            // A bad token throws instead of stopping the program while the
            // string is the input; see `parse_str` in the runtime.
            Spec::Parse(text, inner) => {
                self.need("parse_str");
                let ty = self.ty(&spec.ty());
                let text = self.expr(text).text;
                let value = self.read(inner).text;
                self.lambda(&ty, |this, out| {
                    let lead = pad(this.indent);
                    out.push_str(&format!("{lead}cpl::parse_scope scope({text});\n"));
                    out.push_str(&format!("{lead}try {{\n"));
                    out.push_str(&format!("{lead}    auto value = {value};\n"));
                    out.push_str(&format!(
                        "{lead}    if (scope.whole()) return {ty}(std::move(value));\n"
                    ));
                    out.push_str(&format!("{lead}}} catch (const cpl::parse_failed&) {{}}\n"));
                    out.push_str(&format!("{lead}return {ty}();\n"));
                })
            }
            Spec::Fields(ty, fields) => {
                // `{...}` runs its elements in order: so are the fields read.
                let struct_id = match ty {
                    Ty::Struct(id) => Some(*id),
                    _ => None,
                };
                let fields: Vec<String> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let v = self.read(field).text;
                        match struct_id.and_then(|id| self.narrow_ints.get(&(id, i))) {
                            Some(w) => format!("{}({v})", w.cpp()),
                            None => v,
                        }
                    })
                    .collect();
                Code::primary(format!("{}{{{}}}", self.ty(ty), fields.join(", ")))
            }
            Spec::Array { dims, elem } => {
                let ty = self.ty(&spec.ty());
                self.lambda(&ty, |this, out| {
                    let name = this.temp();
                    this.read_array(&name, true, dims, elem, out);
                    out.push_str(&format!("{}return {name};\n", pad(this.indent)));
                })
            }
        }
    }

    /// Statements that read an array into `name`, declaring it if asked.
    pub fn read_array(
        &mut self,
        name: &str,
        declare: bool,
        dims: &[DimSpec],
        elem: &Spec,
        out: &mut String,
    ) {
        self.read_array_with_layout(name, declare, dims, elem, None, out);
    }

    pub fn read_array_with_layout(
        &mut self,
        name: &str,
        declare: bool,
        dims: &[DimSpec],
        elem: &Spec,
        layout: Option<&cpl_ir::NarrowLayout>,
        out: &mut String,
    ) {
        let elem_ty_ir = elem.ty();
        let cell = layout
            .and_then(|l| l.cpp_elem(&elem_ty_ir))
            .unwrap_or_else(|| elem_ty(self.program, self.niches, &elem_ty_ir));
        let lens: Vec<String> = dims.iter().map(|d| self.dim(d)).collect();
        let lead = pad(self.indent);
        let items = if dims.len() == 1 {
            if declare {
                out.push_str(&format!("{lead}std::vector<{cell}> {name}({});\n", lens[0]));
            } else {
                out.push_str(&format!("{lead}{name}.assign({}, {cell}());\n", lens[0]));
            }
            name.to_string()
        } else {
            self.need("grid");
            let made = format!(
                "cpl::grid<{cell}, {}>({{{}}}, {cell}())",
                dims.len(),
                lens.join(", ")
            );
            if declare {
                out.push_str(&format!(
                    "{lead}cpl::Grid<{cell}, {}> {name} = {made};\n",
                    dims.len()
                ));
            } else {
                out.push_str(&format!("{lead}{name} = {made};\n"));
            }
            format!("{name}.d")
        };
        let x = self.temp();
        match (layout, elem) {
            (
                Some(cpl_ir::NarrowLayout::Seq(inner)),
                Spec::Array {
                    dims: inner_dims,
                    elem: inner_elem,
                },
            ) => {
                out.push_str(&format!("{lead}for (auto& {x} : {items}) {{\n"));
                self.indent += 1;
                self.read_array_with_layout(&x, false, inner_dims, inner_elem, Some(inner), out);
                self.indent -= 1;
                out.push_str(&format!("{lead}}}\n"));
            }
            (Some(l @ (cpl_ir::NarrowLayout::Int(_) | cpl_ir::NarrowLayout::Tuple(_))), _) => {
                let value = self.read(elem).text;
                let rhs = self.narrow_cell_value(&value, l);
                out.push_str(&format!("{lead}for (auto& {x} : {items}) {x} = {rhs};\n"));
            }
            (None, Spec::Plain(Ty::Char)) => {
                self.need("read_chars");
                out.push_str(&format!("{lead}cpl::read_chars({items});\n"));
            }
            _ => {
                let value = self.read(elem).text;
                out.push_str(&format!("{lead}for (auto& {x} : {items}) {x} = {value};\n"));
            }
        }
    }
}
