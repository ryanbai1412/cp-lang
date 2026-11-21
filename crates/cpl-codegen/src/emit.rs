//! Printing a lowered program as one C++17 file.

use crate::names::{cty, fn_name, ident, is_plain_enum, Niches};
use crate::ranges::{Facts, Var};
use crate::runtime::PIECES;
use crate::walk;
use cpl_ir::{Block, Expr, ExprKind, Function, Local, LoopId, Program, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

/// A C++ expression and how tightly it binds, so that it is wrapped in
/// parentheses only where C++ would otherwise read it differently.
#[derive(Clone, Debug)]
pub(crate) struct Code {
    pub text: String,
    pub prec: u8,
    /// True when the C++ type is `int`: a small literal, or arithmetic
    /// on two of them. Such a value is widened before arithmetic that
    /// could leave 32 bits.
    pub narrow: bool,
}

pub(crate) const PRIMARY: u8 = 16;
pub(crate) const UNARY: u8 = 15;
pub(crate) const MUL: u8 = 13;
pub(crate) const ADD: u8 = 12;
pub(crate) const SHIFT: u8 = 11;
pub(crate) const REL: u8 = 9;
pub(crate) const EQ: u8 = 8;
pub(crate) const BIT: u8 = 5;
pub(crate) const AND: u8 = 4;
pub(crate) const OR: u8 = 3;
pub(crate) const COND: u8 = 2;

impl Code {
    pub fn new(text: impl Into<String>, prec: u8) -> Code {
        Code {
            text: text.into(),
            prec,
            narrow: false,
        }
    }

    pub fn primary(text: impl Into<String>) -> Code {
        Code::new(text, PRIMARY)
    }

    /// The text, in parentheses unless it binds at least as tightly as `min`.
    pub fn at(&self, min: u8) -> String {
        if self.prec >= min {
            self.text.clone()
        } else {
            format!("({})", self.text)
        }
    }
}

/// What evaluating an expression touches, to tell when the order two
/// operands run in can be seen.
#[derive(Default, Debug)]
pub(crate) struct Touch {
    /// Reads input or writes output.
    pub io: bool,
    pub reads: BTreeSet<(bool, usize)>,
    pub writes: BTreeSet<(bool, usize)>,
}

impl Touch {
    /// True when running `self` before `later` can differ from after.
    pub fn conflicts(&self, later: &Touch) -> bool {
        (self.io && later.io)
            || self
                .writes
                .iter()
                .any(|v| later.reads.contains(v) || later.writes.contains(v))
            || later.writes.iter().any(|v| self.reads.contains(v))
    }
}

pub(crate) use cpl_ir::pushed::FnSummary;

/// A loop over a `yields` fn whose body is the fn's callback. The
/// callback gives 0 to go on, 1 for `break`, 2 for a `return` of the fn
/// the loop is in.
pub(crate) struct Pushed {
    pub id: cpl_ir::LoopId,
    /// How many loops of the body are open around what is being written.
    pub depth: usize,
    /// Where a `return` in the body leaves its value.
    pub result: Option<String>,
}

pub(crate) struct Emitter<'p> {
    pub program: &'p Program,
    pub facts: Facts<'p>,
    pub niches: Niches,
    pub devirt: crate::devirt::Devirt<'p>,
    /// Closures called by name, each emitted once as a fn of the program.
    pub direct: BTreeSet<usize>,
    /// The `?Int` tables held as `cpl::OptInt`.
    pub packed: BTreeSet<Var>,
    /// Set for the value a packed table is created from, which is then
    /// written with the packed cell type.
    pub pack_init: bool,
    /// Set for a cell of a packed table that is only tested or opened,
    /// so it is read as it is held.
    pub raw_cell: bool,
    /// Which of `cpl::OptPos`, `cpl::OptBool` and `cpl::OptChar` the file
    /// names, as bits.
    pub niche_types: std::cell::Cell<u8>,
    /// Runtime pieces the program uses, by name.
    pub used: BTreeSet<&'static str>,
    /// Names taken at the top level of the file.
    pub top_names: BTreeSet<String>,
    /// `show` helpers by the type they print, in the order first needed.
    pub shows: Vec<String>,
    pub show_index: BTreeMap<Ty, usize>,
    /// The types whose parts are being given helpers.
    pub showing: BTreeSet<Ty>,
    /// True when some type holds itself.
    pub show_cycle: bool,
    pub summaries: Vec<FnSummary>,
    /// The fn being emitted (`None` is the top level), and its locals.
    pub fn_id: Option<usize>,
    /// The closure currently being emitted, if inside a closure body.
    pub closure_id: Option<usize>,
    pub names: Vec<String>,
    pub hoisted: BTreeSet<usize>,
    /// What follows the `let` being written, in its block.
    pub tail: Option<crate::stmt::Tail>,
    /// Local lists kept in one buffer for the whole run: see [`crate::scope::reused`].
    pub reused: BTreeSet<usize>,
    /// The `yields` fns that also exist as `cpl_each_f(args, body)`, with
    /// the slot of the list each one builds: see [`crate::scope::pusher`].
    /// `(struct, field)` to the cell count of a field kept inside the
    /// struct as a `cpl::Row<T, K>`, not as a `std::vector` of its own.
    pub rows: BTreeMap<(usize, usize), u32>,
    pub pushers: BTreeMap<usize, usize>,
    /// Set while the `cpl_each_f` form of a fn is written: the slot of its list.
    pub pushing: Option<usize>,
    /// The loops over a `yields` fn being written as its callback, innermost last.
    pub bodies: Vec<Pushed>,
    /// Emitting what is above `@!multitest`, where a top-level `return`
    /// ends the program.
    pub in_setup: bool,
    /// Tuple temporaries whose fields are moved out: see [`crate::scope::spent`].
    pub spent: BTreeSet<usize>,
    pub taken: BTreeSet<String>,
    pub fresh: usize,
    /// Loops that a `break` or `continue` names from an inner loop.
    pub break_targets: BTreeSet<LoopId>,
    pub continue_targets: BTreeSet<LoopId>,
    /// Inside `if k in m:`, the map, the key and the name of the C++
    /// iterator that already stands on the entry, so `m[k]` is not
    /// looked up a second time.
    pub found: Vec<(Expr, Expr, String)>,
    /// `(list, index)` slots of the loops being written whose index
    /// walks `0..list.len` over a list the body leaves at that length.
    pub inside: Vec<(usize, usize)>,
    /// How deep the statement being written is indented.
    pub indent: usize,
}

pub(crate) fn pad(indent: usize) -> String {
    "    ".repeat(indent)
}

impl<'p> Emitter<'p> {
    pub fn new(program: &'p Program) -> Emitter<'p> {
        // What a local must not hide: the names at the top of the file.
        let mut top_names = BTreeSet::new();
        top_names.extend(program.structs.iter().map(|s| ident(&s.name)));
        top_names.extend(program.enums.iter().map(|e| ident(&e.name)));
        top_names.extend(program.globals.iter().map(|g| ident(&g.name)));
        top_names.extend(program.fns.iter().map(|f| fn_name(&f.name)));
        for d in &program.domains {
            let d = ident(d);
            top_names.insert(format!("{d}_lo"));
            top_names.insert(format!("{d}_len"));
            top_names.insert(d);
        }
        let mut emitter = Emitter {
            program,
            facts: Facts::collect(program),
            niches: crate::niche::small(program),
            devirt: crate::devirt::Devirt::collect(program),
            direct: BTreeSet::new(),
            packed: crate::niche::packed_tables(program),
            pack_init: false,
            raw_cell: false,
            niche_types: std::cell::Cell::new(0),
            used: BTreeSet::new(),
            top_names,
            shows: Vec::new(),
            show_index: BTreeMap::new(),
            showing: BTreeSet::new(),
            show_cycle: false,
            summaries: Vec::new(),
            fn_id: None,
            closure_id: None,
            names: Vec::new(),
            hoisted: BTreeSet::new(),
            tail: None,
            in_setup: false,
            spent: BTreeSet::new(),
            taken: BTreeSet::new(),
            fresh: 0,
            break_targets: BTreeSet::new(),
            continue_targets: BTreeSet::new(),
            reused: BTreeSet::new(),
            rows: crate::scope::rows(program),
            pushers: BTreeMap::new(),
            pushing: None,
            bodies: Vec::new(),
            found: Vec::new(),
            inside: Vec::new(),
            indent: 0,
        };
        let feeds = cpl_ir::pushed::Feeds::of(program);
        emitter.summaries = feeds.summaries;
        emitter.pushers = feeds.pushers;
        emitter
    }

    pub fn need(&mut self, piece: &'static str) {
        if self.used.insert(piece) {
            if let Some(found) = PIECES.iter().find(|p| p.name == piece) {
                for dep in found.needs {
                    self.need(dep);
                }
            }
        }
    }

    pub fn ty(&self, ty: &Ty) -> String {
        let cpp = cty(self.program, self.niches, ty);
        // A packed optional can be the type of an expression alone, so
        // what the file needs is noted where the type is written.
        for (at, name) in ["cpl::OptPos", "cpl::OptBool", "cpl::OptChar", "cpl::OptTuple"].iter().enumerate() {
            if cpp.contains(name) {
                self.niche_types.set(self.niche_types.get() | 1 << at);
            }
        }
        cpp
    }

    /// True when the variable is a `?Int` table held as `cpl::OptInt`. A
    /// closure's locals are its own and never are.
    pub fn is_packed(&self, global: bool, index: usize) -> bool {
        let var: Var = match global {
            true => (None, true, index),
            false if self.closure_id.is_some() => return false,
            false => (self.fn_id, false, index),
        };
        self.packed.contains(&var)
    }

    /// The C++ type of a packed table of type `ty`.
    pub fn packed_ty(&mut self, ty: &Ty) -> String {
        self.need("niche_int");
        self.ty(ty).replace("std::optional<ll>", "cpl::OptInt")
    }

    /// `text`, the C++ of the `?Int` value `value`, as a packed cell.
    pub fn pack(&mut self, value: &Expr, text: &str) -> String {
        self.need("niche_int");
        match (&value.kind, text.strip_prefix("std::optional<ll>(")) {
            (ExprKind::None, _) => "cpl::OptInt()".into(),
            (ExprKind::Some(_), Some(rest)) => format!("cpl::OptInt({rest}"),
            _ => format!("cpl::OptInt({text})"),
        }
    }



    /// A name nothing else in this fn or at the top level has.
    pub fn temp(&mut self) -> String {
        loop {
            let name = format!("t{}", self.fresh);
            self.fresh += 1;
            if !self.taken.contains(&name) && !self.top_names.contains(&name) {
                self.taken.insert(name.clone());
                return name;
            }
        }
    }

    /// `base`, changed until nothing else in this fn has it.
    pub fn unique(&mut self, base: &str) -> String {
        let mut name = base.to_string();
        let mut n = 2;
        while self.taken.contains(&name) || self.top_names.contains(&name) {
            name = format!("{base}_{n}");
            n += 1;
        }
        self.taken.insert(name.clone());
        name
    }

    pub fn locals(&self) -> &'p [Local] {
        if let Some(cid) = self.closure_id {
            return &self.program.closures[cid].locals;
        }
        match self.fn_id {
            Some(id) => &self.program.fns[id].locals,
            None => &self.program.main_locals,
        }
    }

    pub fn need_ty(&mut self, ty: &Ty) {
        match ty {
            Ty::Map(k, v) if self.program.unordered_maps.contains(k) => {
                self.need_ty(&Ty::HashMap(k.clone(), v.clone()))
            }
            Ty::Set(e) if self.program.unordered_sets.contains(e) => self.need_ty(&Ty::HashSet(e.clone())),
            Ty::HashMap(k, v) => {
                self.need("cphash");
                self.need_ty(k);
                self.need_ty(v);
            }
            Ty::HashSet(e) => {
                self.need("cphash");
                self.need_ty(e);
            }
            Ty::Range(inner) => {
                self.need("range");
                self.need_ty(inner);
            }
            Ty::DomainVal => self.need("domain_val"),
            Ty::Array { dims, elem } => {
                if dims.len() > 1 {
                    self.need("grid");
                }
                self.need_ty(elem);
            }
            Ty::Opt(t) | Ty::List(t) | Ty::Set(t) => self.need_ty(t),
            Ty::Map(k, v) => {
                self.need_ty(k);
                self.need_ty(v);
            }
            Ty::Fn(args, ret) => {
                for a in args {
                    self.need_ty(a);
                }
                self.need_ty(ret);
            }
            Ty::Tuple(ts) => ts.iter().for_each(|t| self.need_ty(t)),
            Ty::Record(fs) => fs.iter().for_each(|(_, t)| self.need_ty(t)),
            _ => {}
        }
    }

    /// Readies the emitter for one fn: names for its locals, and which
    /// of them are declared at the top because a use is out of sight of
    /// the declaration.
    fn enter(&mut self, fn_id: Option<usize>, body: &'p Block, params: usize) {
        self.fn_id = fn_id;
        self.taken.clear();
        self.fresh = 0;
        self.names.clear();
        let locals = self.locals();
        for local in locals {
            let name = if local.name.is_empty() {
                self.temp()
            } else {
                self.unique(&ident(&local.name))
            };
            self.names.push(name);
        }
        self.hoisted = crate::scope::hoisted(body, params);
        self.reused = crate::scope::reused(self.program, fn_id, body);
        self.spent = crate::scope::spent(self.program, body);
        self.break_targets.clear();
        self.continue_targets.clear();
        walk::deep_stmts(body, &mut |stmt| match stmt {
            Stmt::Break(Some(id)) => {
                self.break_targets.insert(*id);
            }
            Stmt::Continue(Some(id)) => {
                self.continue_targets.insert(*id);
            }
            _ => {}
        });
    }

    /// `ll gcd(ll a, ll b)`. Call after [`Emitter::enter`].
    fn signature(&self, f: &Function) -> String {
        let id = self.fn_id.expect("a fn");
        let named = crate::scope::named_params(&f.body, f.params.len());
        let params: Vec<String> = f
            .params
            .iter()
            .enumerate()
            // A parameter that holds no value (a bare operator the fn was
            // specialized for) is not a C++ parameter: `void x` is not one.
            .filter(|(slot, _)| f.locals[*slot].ty != Ty::Unit)
            .map(|(slot, param)| {
                let ty = &f.locals[slot].ty;
                let name = &self.names[slot];
                let cpp = self.ty(ty);
                if self.devirt.param_known(id, slot) {
                    format!("[[maybe_unused]] const {cpp}& {name}")
                } else if !named.contains(&slot) && !param.mutable {
                    // g++ warns of a parameter the body never names.
                    let by = if ty.is_scalar() { cpp } else { format!("const {cpp}&") };
                    format!("[[maybe_unused]] {by} {name}")
                } else if param.mutable {
                    format!("{cpp}& {name}")
                } else if ty.is_scalar() || self.facts.stores((Some(id), false, slot)) > 2 {
                    format!("{cpp} {name}")
                } else {
                    format!("const {cpp}& {name}")
                }
            })
            .collect();
        format!(
            "{} {}({})",
            self.ty(&f.ret),
            fn_name(&f.name),
            params.join(", ")
        )
    }

    /// The fn as the judge's grader declares it.
    fn judge_signature(&self, f: &Function) -> String {
        let params: Vec<String> = (0..f.params.len())
            .map(|slot| format!("{} {}", judge_ty(&f.locals[slot].ty), self.names[slot]))
            .collect();
        format!("{} {}({})", judge_ty(&f.ret), f.name, params.join(", "))
    }

    /// Declarations of the locals that could not be declared where they
    /// are first set.
    fn hoisted_decls(&mut self, out: &mut String, params: usize) {
        let locals = self.locals();
        for slot in self.hoisted.clone() {
            if slot < params || locals[slot].ty == Ty::Unit {
                continue;
            }
            let ty = &locals[slot].ty;
            let cpp = match self.is_packed(false, slot) {
                true => self.packed_ty(ty),
                false => self.ty(ty),
            };
            out.push_str(&format!(
                "{}{} {}{};\n",
                pad(self.indent),
                cpp,
                self.names[slot],
                self.zero_init(ty)
            ));
        }
    }

    /// The initializer that gives a new variable its type's zero.
    pub fn zero_init(&self, ty: &Ty) -> &'static str {
        match ty {
            Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Char | Ty::Pos(_) | Ty::Bound(_) => " = 0",
            Ty::Bool => " = false",
            Ty::Enum(id) if is_plain_enum(self.program, *id) => "{}",
            _ => "",
        }
    }

    /// The types `ty` holds by value, which must be complete before it.
    fn needs_complete(&self, ty: &Ty, out: &mut Vec<Ty>) {
        match ty {
            Ty::Struct(_) | Ty::Enum(_) => out.push(ty.clone()),
            Ty::Opt(inner)
            | Ty::List(inner)
            | Ty::Set(inner)
            | Ty::HashSet(inner)
            | Ty::Range(inner) => self.needs_complete(inner, out),
            Ty::Array { elem, .. } => self.needs_complete(elem, out),
            Ty::Map(key, value) | Ty::HashMap(key, value) => {
                self.needs_complete(key, out);
                self.needs_complete(value, out);
            }
            Ty::Fn(args, ret) => {
                args.iter().for_each(|t| self.needs_complete(t, out));
                self.needs_complete(ret, out);
            }
            Ty::Tuple(items) => items.iter().for_each(|t| self.needs_complete(t, out)),
            Ty::Record(fields) => fields.iter().for_each(|(_, t)| self.needs_complete(t, out)),
            _ => {}
        }
    }

    fn type_def(&mut self, ty: &Ty, done: &mut BTreeSet<Ty>, out: &mut String) {
        if !done.insert(ty.clone()) {
            return;
        }
        let program = self.program;
        let mut deps = Vec::new();
        match ty {
            Ty::Struct(id) => program.structs[*id]
                .fields
                .iter()
                .for_each(|(_, t)| self.needs_complete(t, &mut deps)),
            Ty::Enum(id) => {
                for (_, fields) in &program.enums[*id].variants {
                    fields
                        .iter()
                        .for_each(|t| self.needs_complete(t, &mut deps));
                }
            }
            _ => {}
        }
        for dep in deps {
            self.type_def(&dep, done, out);
        }
        match ty {
            Ty::Struct(id) => {
                let def = &program.structs[*id];
                out.push_str(&format!("struct {} {{\n", ident(&def.name)));
                let mut names = Vec::new();
                for (field, field_ty) in &def.fields {
                    self.need_ty(field_ty);
                    let name = ident(field);
                    let mut cpp = self.ty(field_ty);
                    let cells = self.rows.get(&(*id, names.len()));
                    if let (Some(k), Some(cell)) = (
                        cells,
                        cpp.strip_prefix("std::vector<").and_then(|c| c.strip_suffix('>')),
                    ) {
                        cpp = format!("cpl::Row<{cell}, {k}>");
                        self.need("row");
                    }
                    out.push_str(&format!(
                        "    {} {}{};\n",
                        cpp,
                        name,
                        self.zero_init(field_ty)
                    ));
                    names.push(name);
                }
                if Ty::Struct(*id).is_comparable(program) {
                    out.push_str(&format!(
                        "    auto cpl_tie() const {{ return std::tie({}); }}\n",
                        names.join(", ")
                    ));
                }
                out.push_str("};\n");
            }
            Ty::Enum(id) if is_plain_enum(program, *id) => {
                let def = &program.enums[*id];
                let names: Vec<String> = def.variants.iter().map(|(v, _)| ident(v)).collect();
                out.push_str(&format!(
                    "enum class {} {{ {} }};\n",
                    ident(&def.name),
                    names.join(", ")
                ));
            }
            Ty::Enum(id) => {
                let def = &program.enums[*id];
                let name = ident(&def.name);
                let tags: Vec<String> = def.variants.iter().map(|(v, _)| ident(v)).collect();
                out.push_str(&format!(
                    "struct {name} {{\n    enum Tag {{ {} }} tag = {};\n",
                    tags.join(", "),
                    tags[0]
                ));
                let mut tie = vec!["tag".to_string()];
                let mut makers = String::new();
                for (variant, fields) in &def.variants {
                    let variant = ident(variant);
                    let mut params = Vec::new();
                    let mut sets = String::new();
                    for (i, field_ty) in fields.iter().enumerate() {
                        let field = format!("{variant}_{i}");
                        let cpp = self.ty(field_ty);
                        out.push_str(&format!("    {cpp} {field}{};\n", self.zero_init(field_ty)));
                        params.push(if field_ty.is_scalar() {
                            format!("{cpp} a{i}")
                        } else {
                            format!("const {cpp}& a{i}")
                        });
                        sets.push_str(&format!(" v.{field} = a{i};"));
                        tie.push(field);
                    }
                    if !fields.is_empty() {
                        makers.push_str(&format!(
                            "    static {name} make_{variant}({}) {{ {name} v; v.tag = {variant};{sets} return v; }}\n",
                            params.join(", ")
                        ));
                    } else {
                        makers.push_str(&format!("    static {name} make_{variant}() {{ {name} v; v.tag = {variant}; return v; }}\n"));
                    }
                }
                out.push_str(&makers);
                out.push_str(&format!(
                    "    auto cpl_tie() const {{ return std::tie({}); }}\n}};\n",
                    tie.join(", ")
                ));
            }
            _ => {}
        }
    }

    /// The whole file.
    pub fn file(mut self, name: &str, source: Option<&str>) -> String {
        let program = self.program;

        // Ensure runtime types used by globals, fns, closures, and main are registered.
        for g in &program.globals {
            self.need_ty(&g.ty);
        }
        for f in &program.fns {
            self.need_ty(&f.ret);
            for l in &f.locals {
                self.need_ty(&l.ty);
            }
        }
        for c in &program.closures {
            self.need_ty(&c.ret);
            for l in &c.locals {
                self.need_ty(&l.ty);
            }
        }
        for l in &program.main_locals {
            self.need_ty(&l.ty);
        }

        // The program first, since writing it finds out what it needs.
        let mut types = String::new();
        let mut done = BTreeSet::new();
        for id in 0..program.structs.len() {
            self.type_def(&Ty::Struct(id), &mut done, &mut types);
        }
        for id in 0..program.enums.len() {
            self.type_def(&Ty::Enum(id), &mut done, &mut types);
        }
        let has_tie = types.contains("cpl_tie");

        let mut vars = String::new();
        for d in &program.domains {
            let d = ident(d);
            vars.push_str(&format!("ll {d}_lo = 0, {d}_len = 0;\n"));
        }
        for (id, global) in program.globals.iter().enumerate() {
            if global.ty == Ty::Unit {
                continue;
            }
            let cpp = match self.packed.contains(&(None, true, id)) {
                true => self.packed_ty(&global.ty),
                false => self.ty(&global.ty),
            };
            vars.push_str(&format!(
                "{} {}{};\n",
                cpp,
                ident(&global.name),
                self.zero_init(&global.ty)
            ));
        }

        // Compute reachable fns so uncalled stdlib stubs are omitted from the emitted C++17.
        let mut reachable: BTreeSet<usize> = BTreeSet::new();
        let mut work: Vec<usize> = Vec::new();
        let enqueue = |fid: usize, reachable: &mut BTreeSet<usize>, work: &mut Vec<usize>| {
            if reachable.insert(fid) {
                work.push(fid);
            }
        };
        let scan_block = |b: &Block, reachable: &mut BTreeSet<usize>, work: &mut Vec<usize>| {
            walk::deep_exprs(b, &mut |e| {
                if let ExprKind::Call(fid, _) = &e.kind {
                    if reachable.insert(*fid) {
                        work.push(*fid);
                    }
                }
            });
        };
        for &entry in &program.entry_fns {
            enqueue(entry, &mut reachable, &mut work);
        }
        scan_block(&program.setup, &mut reachable, &mut work);
        scan_block(&program.main, &mut reachable, &mut work);
        for c in &program.closures {
            scan_block(&c.body, &mut reachable, &mut work);
        }
        for s in &program.structs {
            for hook in [s.eq_fn, s.lt_fn, s.show_fn].into_iter().flatten() {
                enqueue(hook, &mut reachable, &mut work);
            }
        }
        while let Some(fid) = work.pop() {
            scan_block(&program.fns[fid].body, &mut reachable, &mut work);
        }

        let mut protos = String::new();
        let mut fns = String::new();
        let mut judge_decls = String::new();
        let mut entries = String::new();
        for (id, f) in program.fns.iter().enumerate() {
            if !reachable.contains(&id) {
                continue;
            }
            self.enter(Some(id), &f.body, f.params.len());
            let signature = self.signature(f);
            protos.push_str(&format!("{signature};\n"));
            if program.judge_fns.contains(&id) {
                // The judge's fn, declared at the top of the file, behind
                // a fn of the program's own types.
                self.need("judge");
                judge_decls.push_str(&format!("{};\n", self.judge_signature(f)));
                let args: Vec<String> = (0..f.params.len())
                    .map(|slot| {
                        format!(
                            "cpl::Judge<{}>::to({})",
                            self.ty(&f.locals[slot].ty),
                            self.names[slot]
                        )
                    })
                    .collect();
                let call = format!("::{}({})", f.name, args.join(", "));
                let body = match f.ret {
                    Ty::Unit => format!("{call};"),
                    _ => format!("return cpl::Judge<{}>::from({call});", self.ty(&f.ret)),
                };
                fns.push_str(&format!("\n{signature} {{\n    {body}\n}}\n"));
                continue;
            }
            if program.entry_fns.contains(&id) {
                // What the judge's grader calls: the same fn in its types.
                self.need("judge");
                let args: Vec<String> = (0..f.params.len())
                    .map(|slot| {
                        format!(
                            "cpl::Judge<{}>::from({})",
                            self.ty(&f.locals[slot].ty),
                            self.names[slot]
                        )
                    })
                    .collect();
                let call = format!("prog::{}({})", fn_name(&f.name), args.join(", "));
                // What it printed goes out before the grader runs on.
                let body = match f.ret {
                    Ty::Unit => format!("{call};\n    cpl::flush();"),
                    _ => format!(
                        "auto result = cpl::Judge<{}>::to({call});\n    cpl::flush();\n    return result;",
                        self.ty(&f.ret)
                    ),
                };
                entries.push_str(&format!(
                    "\n{} {{\n    cpl_start();\n    {body}\n}}\n",
                    self.judge_signature(f)
                ));
            }
            fns.push_str(&format!("\n{signature} {{\n"));
            self.indent = 1;
            self.hoisted_decls(&mut fns, f.params.len());
            self.block(&f.body, &mut fns);
            fns.push_str("}\n");
            if let Some(acc) = self.pushers.get(&id).copied() {
                // The same fn handing each value to `cpl_f` as it is made,
                // with no list. It may be running twice (a loop body can
                // walk it again), so it keeps no buffer of its own.
                let name = fn_name(&f.name);
                let params = signature.split_once(&format!(" {name}(")).map(|(_, p)| p.trim_end_matches(')')).unwrap_or("");
                let sep = if params.is_empty() { "" } else { ", " };
                let head = format!("template <class CplF> int cpl_each_{name}({params}{sep}CplF&& cpl_f)");
                protos.push_str(&format!("{head};\n"));
                fns.push_str(&format!("\n{head} {{\n"));
                self.reused.clear();
                self.pushing = Some(acc);
                self.hoisted_decls(&mut fns, f.params.len());
                self.block(&f.body, &mut fns);
                self.pushing = None;
                fns.push_str("}\n");
            }
        }
        if program.multitest {
            // What is above `@!multitest`. It gives false when a top-level
            // `return` there ended the program.
            self.enter(None, &program.setup, 0);
            fns.push_str("\nbool setup() {\n");
            self.indent = 1;
            self.hoisted_decls(&mut fns, 0);
            self.in_setup = true;
            self.block(&program.setup, &mut fns);
            self.in_setup = false;
            fns.push_str("    return true;\n}\n");
        }
        self.enter(None, &program.main, 0);
        fns.push_str("\nvoid solve() {\n");
        self.indent = 1;
        self.hoisted_decls(&mut fns, 0);
        self.block(&program.main, &mut fns);
        fns.push_str("}\n");

        // `main` writes through the runtime; what reads asks for the input itself.
        self.need("out");
        if program.multitest {
            self.need(if program.until_eof { "at_eof" } else { "read_int" });
        }

        let mut out = String::new();
        out.push_str(&header(name, source));
        // Closures that are called by name (see `devirt`): one calling
        // another adds to the set, so emit until it stops growing.
        let mut direct = String::new();
        let mut done: Vec<usize> = Vec::new();
        while let Some(cid) = self.direct.iter().copied().find(|cid| !done.contains(cid)) {
            done.push(cid);
            self.enter(None, &program.closures[cid].body, 0);
            let body = self.emit_closure(cid).text;
            direct.push_str(&format!("static auto {} = {body};\n", crate::devirt::name(cid)));
        }
        out.push_str(
            "#include <algorithm>\n#include <array>\n#include <cmath>\n#include <cstdint>\n#include <cstdio>\n#include <cstdlib>\n\
             #include <map>\n#include <optional>\n#include <set>\n#include <string>\n#include <tuple>\n#include <vector>\n#include <unistd.h>\n",
        );
        if !program.closures.is_empty() {
            out.push_str("#include <functional>\n");
        }
        if self.used.contains("cphash") {
            out.push_str("#include <chrono>\n#include <unordered_map>\n#include <unordered_set>\n");
        }
        out.push_str("\nusing ll = long long;\n\n");
        if !judge_decls.is_empty() {
            out.push_str("// The judge's grader supplies these.\n");
            out.push_str(&judge_decls);
            out.push('\n');
        }

        out.push_str(
            "// What the program below leans on.\nnamespace cpl {\nusing u8 = unsigned char;\n",
        );
        for (at, piece) in ["niche", "niche_bool", "niche_char", "niche_tuple"].into_iter().enumerate() {
            if self.niche_types.get() & 1 << at != 0 {
                self.need(piece);
            }
        }
        for piece in PIECES {
            if self.used.contains(piece.name) {
                out.push_str(piece.code);
            }
        }
        out.push_str("}  // namespace cpl\n\n// The program.\nnamespace prog {\n");
        if has_tie {
            out.push_str(TIE_OPS);
        }
        out.push_str(&types);
        if !types.is_empty() {
            out.push('\n');
        }
        out.push_str(&vars);
        if !vars.is_empty() {
            out.push('\n');
        }
        out.push_str(&protos);
        out.push_str(&direct);
        // A struct with its own `Ord` is ordered by it everywhere: in a
        // sort, a set, a heap and inside a tuple, not only in `a < b`.
        for (id, s) in program.structs.iter().enumerate() {
            let Some(lt) = s.lt_fn.filter(|lt| reachable.contains(lt)) else {
                continue;
            };
            let (ty, lt) = (self.ty(&Ty::Struct(id)), fn_name(&program.fns[lt].name));
            out.push_str(&format!(
                "inline bool operator<(const {ty}& a, const {ty}& b) {{ return {lt}(a, b); }}\n\
                 inline bool operator>(const {ty}& a, const {ty}& b) {{ return {lt}(b, a); }}\n\
                 inline bool operator<=(const {ty}& a, const {ty}& b) {{ return !{lt}(b, a); }}\n\
                 inline bool operator>=(const {ty}& a, const {ty}& b) {{ return !{lt}(a, b); }}\n"
            ));
        }
        if self.show_cycle {
            for show in &self.shows {
                if let Some(signature) =
                    show.lines().nth(1).and_then(|line| line.strip_suffix(" {"))
                {
                    out.push_str(&format!("{signature};\n"));
                }
            }
            out.push('\n');
        }
        for show in &self.shows {
            out.push_str(show);
            out.push('\n');
        }
        out.push_str(&fns);
        if !program.entry_fns.is_empty() {
            // No `main`: the judge's grader has it. The file's top-level
            // code runs once, before the first fn the grader calls.
            out.push_str(
                "}  // namespace prog\n\n// The judge's grader calls these.\n\
                 static void cpl_start() {\n    static bool started = false;\n    if (started) return;\n    started = true;\n",
            );
            out.push_str("    prog::solve();\n}\n");
            out.push_str(&entries);
            return out;
        }
        // A fn that calls itself can go deeper than the 8 MB a judge
        // gives the stack (a chain of 5e4 `@memo` calls does).
        let deep = recursive(program, &reachable);
        out.push_str("}  // namespace prog\n\n");
        if deep {
            out.push_str(BIG_STACK);
        }
        out.push_str(if deep { "static void cpl_main() {\n" } else { "int main() {\n" });
        // Each test case has its own query limit.
        let case = if self.used.contains("count_query") {
            "{ cpl::asked = 0; prog::solve(); }"
        } else {
            "prog::solve();"
        };
        if program.until_eof {
            out.push_str(&format!("    if (prog::setup()) while (!cpl::at_eof()) {case}\n"));
        } else if program.multitest {
            out.push_str(&format!(
                "    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) {case}\n",
            ));
        } else {
            out.push_str("    prog::solve();\n");
        }
        out.push_str(if deep {
            "    cpl::flush();\n}\n\nint main() {\n    cpl_on_big_stack(cpl_main);\n    return 0;\n}\n"
        } else {
            "    cpl::flush();\n    return 0;\n}\n"
        });
        out
    }

    /// True when `e` gives the same value wherever it runs in the
    /// statement: a literal, or a variable set once.
    pub fn is_fixed(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::DomainLen(_)
            | ExprKind::DomainLo(_) => true,
            ExprKind::Local(slot) => {
                e.ty.is_scalar() && self.facts.stable((self.fn_id, false, *slot))
            }
            ExprKind::Global(id) => e.ty.is_scalar() && self.facts.stable((None, true, *id)),
            ExprKind::Cast(_, inner) | ExprKind::Unary(_, inner) => self.is_fixed(inner),
            _ => false,
        }
    }
}

/// Whether some fn of the program can call itself, directly or through
/// others.
fn recursive(program: &Program, reachable: &BTreeSet<usize>) -> bool {
    let calls: Vec<Vec<usize>> = program
        .fns
        .iter()
        .enumerate()
        .map(|(id, f)| {
            let mut out = Vec::new();
            if reachable.contains(&id) {
                walk::deep_exprs(&f.body, &mut |e| {
                    if let ExprKind::Call(callee, _) = &e.kind {
                        out.push(*callee);
                    }
                });
            }
            out
        })
        .collect();
    // 0: not seen, 1: on the path, 2: done.
    let mut state = vec![0u8; calls.len()];
    for root in 0..calls.len() {
        if state[root] != 0 {
            continue;
        }
        state[root] = 1;
        let mut path = vec![(root, 0)];
        while let Some((id, next)) = path.last_mut() {
            let Some(&callee) = calls[*id].get(*next) else {
                state[*id] = 2;
                path.pop();
                continue;
            };
            *next += 1;
            match state[callee] {
                0 => {
                    state[callee] = 1;
                    path.push((callee, 0));
                }
                1 => return true,
                _ => {}
            }
        }
    }
    // A closure reaches itself only through a variable: any closure that
    // calls a closure value may.
    program.closures.iter().any(|c| {
        let mut found = false;
        walk::deep_exprs(&c.body, &mut |e| found |= matches!(e.kind, ExprKind::CallExpr(..)));
        found
    })
}

/// Runs the program on a 256 MB stack of its own where the platform's
/// calling convention is known; elsewhere on the stack it was given.
/// Pages are only committed as the recursion reaches them.
const BIG_STACK: &str = "\
// Deep recursion: the program runs on a 256 MB stack of its own.
#if defined(__x86_64__) && defined(__linux__) && !defined(__SANITIZE_ADDRESS__)
extern \"C\" void cpl_switch_stack(void (*run)(), void* top);
__asm__(\".text\\n.globl cpl_switch_stack\\n.type cpl_switch_stack,@function\\ncpl_switch_stack:\\n\"
        \"  pushq %rbp\\n  movq %rsp, %rbp\\n  movq %rsi, %rsp\\n  callq *%rdi\\n\"
        \"  movq %rbp, %rsp\\n  popq %rbp\\n  ret\\n.size cpl_switch_stack,.-cpl_switch_stack\\n\");
static void cpl_on_big_stack(void (*run)()) {
    const size_t size = (size_t)256 << 20;
    char* stack = (char*)std::malloc(size);
    if (!stack) return run();
    cpl_switch_stack(run, stack + size);
}
#else
static void cpl_on_big_stack(void (*run)()) { run(); }
#endif

";

/// Comparisons for the program's own types, from their fields in order.
const TIE_OPS: &str = "\
// A struct compares field by field, in order.
template <class T> auto operator==(const T& a, const T& b) -> decltype(a.cpl_tie() == b.cpl_tie()) { return a.cpl_tie() == b.cpl_tie(); }
template <class T> auto operator!=(const T& a, const T& b) -> decltype(a.cpl_tie() != b.cpl_tie()) { return a.cpl_tie() != b.cpl_tie(); }
template <class T> auto operator<(const T& a, const T& b) -> decltype(a.cpl_tie() < b.cpl_tie()) { return a.cpl_tie() < b.cpl_tie(); }
template <class T> auto operator<=(const T& a, const T& b) -> decltype(a.cpl_tie() <= b.cpl_tie()) { return a.cpl_tie() <= b.cpl_tie(); }
template <class T> auto operator>(const T& a, const T& b) -> decltype(a.cpl_tie() > b.cpl_tie()) { return a.cpl_tie() > b.cpl_tie(); }
template <class T> auto operator>=(const T& a, const T& b) -> decltype(a.cpl_tie() >= b.cpl_tie()) { return a.cpl_tie() >= b.cpl_tie(); }

";

/// The comment that opens the file: what it is, and the source it was
/// built from, so that whoever reads the C++ can read the program.
fn header(name: &str, source: Option<&str>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let Some(source) = source else {
        return format!(
            "// {name}: C++17 written by cpl {version} from CP Lang source, which is left out\n\
             // here to keep this file within the size a judge accepts.\n\n"
        );
    };
    let mut out =
        format!("// {name}: C++17 written by cpl {version} from the CP Lang source below.\n//\n");
    for line in source.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            out.push_str("//\n");
        } else if line.ends_with('\\') {
            // A `\` at the end would join the next line to this comment.
            out.push_str(&format!("//     {line} .\n"));
        } else {
            out.push_str(&format!("//     {line}\n"));
        }
    }
    out.push('\n');
    out
}

/// A type as the judge's grader writes it: what `cpl::Judge<T>::type` is.
fn judge_ty(ty: &Ty) -> String {
    match ty {
        Ty::Unit => "void".into(),
        Ty::Int => "int".into(),
        Ty::Float => "double".into(),
        Ty::Bool => "bool".into(),
        Ty::Char => "char".into(),
        Ty::Str => "std::string".into(),
        Ty::List(elem) => format!("std::vector<{}>", judge_ty(elem)),
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(judge_ty).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        _ => unreachable!("lowering lets only these types cross to the judge"),
    }
}
