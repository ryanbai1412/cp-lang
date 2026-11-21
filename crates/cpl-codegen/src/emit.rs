//! Printing a lowered program as one C++17 file.

use crate::names::{cty, fn_name, ident, is_plain_enum, Niches};
use crate::ranges::Facts;
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
    /// Which of the packed optionals (`cpl::OptPos`, `cpl::OptInt`, ...)
    /// the file names, as bits.
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
    /// Globals stored once, with a literal: emitted as `static constexpr`.
    pub const_globals: BTreeSet<usize>,
    /// What follows the `let` being written, in its block.
    pub tail: Option<crate::stmt::Tail>,
    /// Local lists kept in one buffer for the whole run: see [`crate::scope::reused`].
    pub reused: BTreeSet<usize>,
    /// The `yields` fns that also exist as `cpl_each_f(args, body)`, with
    /// the slot of the list each one builds: see [`crate::scope::pusher`].
    /// `(struct, field)` to the cell count of a field kept inside the
    /// struct as a `cpl::Row<T, K>`, not as a `std::vector` of its own.
    pub rows: BTreeMap<(usize, usize), u32>,
    /// `(struct, field)` to 32-bit storage width for ranged `Int` fields.
    pub narrow_ints: BTreeMap<(usize, usize), cpl_ir::IntWidth>,
    /// Per fn, the parameters it takes by value to move into its result:
    /// see [`crate::scope::sunk_params`].
    pub sunk: Vec<BTreeSet<usize>>,
    /// Set while the call a `return` makes is written: its local arguments
    /// are at their last use.
    pub last_use_call: bool,
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
    /// For a profiled build: the span of each statement that counts its
    /// starts, by the number it counts in.
    pub sites: Option<Vec<cpl_diag::Span>>,
    /// The statements that count, from `cpl_ir::sites::user_stmts`.
    pub counted: std::collections::HashSet<*const Stmt>,
    pub target: crate::Target,
    /// What each part of the file is, by its text: for the size report.
    pub chunks: Vec<(crate::Part, String, String)>,
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
        let facts = Facts::collect(program);
        let rows = crate::scope::rows(program, &facts);
        let mut emitter = Emitter {
            program,
            facts,
            niches: crate::niche::small(program),
            devirt: crate::devirt::Devirt::collect(program),
            direct: BTreeSet::new(),
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
            const_globals: BTreeSet::new(),
            tail: None,
            in_setup: false,
            spent: BTreeSet::new(),
            taken: BTreeSet::new(),
            fresh: 0,
            break_targets: BTreeSet::new(),
            continue_targets: BTreeSet::new(),
            reused: BTreeSet::new(),
            rows,
            narrow_ints: crate::scope::narrow_ints(program),
            sunk: crate::scope::sunk_params(program),
            last_use_call: false,
            pushers: BTreeMap::new(),
            pushing: None,
            bodies: Vec::new(),
            found: Vec::new(),
            inside: Vec::new(),
            indent: 0,
            sites: None,
            counted: Default::default(),
            target: crate::Target::default(),
            chunks: Vec::new(),
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
        for (at, name) in ["cpl::OptPos", "cpl::OptBool", "cpl::OptChar", "cpl::OptTuple", "cpl::OptInt", "cpl::OptField"].iter().enumerate() {
            if cpp.contains(name) {
                self.niche_types.set(self.niche_types.get() | 1 << at);
            }
        }
        cpp
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
                self.need("cphashmap");
                self.need_ty(k);
                self.need_ty(v);
            }
            Ty::HashSet(e) => {
                self.need("cphashset");
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
            Ty::StrView => self.need("str_view"),
            Ty::ListView(t) => {
                self.need("list_view");
                self.need_ty(t);
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
        let tparams = self.devirt.template_params(id);
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
                if self.devirt.param_elidable(id, slot) {
                    return format!("[[maybe_unused]] bool {name}");
                }
                if let Some(ti) = self.devirt.template_param_index(id, slot) {
                    return if self.devirt.param_known(id, slot) || !named.contains(&slot) {
                        format!("[[maybe_unused]] F{ti}&& {name}")
                    } else {
                        format!("F{ti}&& {name}")
                    };
                }
                let cpp = self.ty(ty);
                if self.devirt.param_known(id, slot) {
                    format!("[[maybe_unused]] const {cpp}& {name}")
                } else if !named.contains(&slot) && !param.mutable {
                    // g++ warns of a parameter the body never names.
                    let by = if ty.is_scalar() { cpp } else { format!("const {cpp}&") };
                    format!("[[maybe_unused]] {by} {name}")
                } else if param.mutable {
                    format!("{cpp}& {name}")
                } else if ty.is_scalar()
                    || self.facts.stores((Some(id), false, slot)) > 2
                    || self.sunk[id].contains(&slot)
                {
                    format!("{cpp} {name}")
                } else {
                    format!("const {cpp}& {name}")
                }
            })
            .collect();
        let prefix = if tparams.is_empty() {
            String::new()
        } else {
            let args: Vec<String> = (0..tparams.len()).map(|i| format!("class F{i}")).collect();
            format!("template <{}>\n", args.join(", "))
        };
        format!(
            "{prefix}{} {}({})",
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
            let cpp = self.ty(ty);
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
                    let field_idx = names.len();
                    let name = ident(field);
                    if self.devirt.field_elidable(*id, field_idx) {
                        out.push_str(&format!("    bool {name} = false;\n"));
                        names.push(name);
                        continue;
                    }
                    self.need_ty(field_ty);
                    let mut cpp = self.ty(field_ty);
                    let cells = self.rows.get(&(*id, field_idx));
                    if let (Some(k), Some(cell)) = (
                        cells,
                        cpp.strip_prefix("std::vector<").and_then(|c| c.strip_suffix('>')),
                    ) {
                        cpp = format!("cpl::Row<{cell}, {k}>");
                        self.need("row");
                    } else if let Some(w) = self.narrow_ints.get(&(*id, names.len())) {
                        cpp = match w {
                            cpl_ir::IntWidth::U32 => "uint32_t".into(),
                            cpl_ir::IntWidth::I32 => "int32_t".into(),
                        };
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
        self.write(name, source)
    }

    /// [`Emitter::file`], with the profiled statements' spans.
    pub fn file_with_sites(
        mut self,
        name: &str,
        source: Option<&str>,
    ) -> (String, Vec<cpl_diag::Span>) {
        let cpp = self.write(name, source);
        (cpp, self.sites.take().unwrap_or_default())
    }

    /// [`Emitter::file`], with the text of each part of it.
    pub fn file_with_chunks(mut self, name: &str, source: Option<&str>) -> (String, Vec<(crate::Part, String, String)>) {
        let cpp = self.write(name, source);
        (cpp, self.chunks)
    }

    fn write(&mut self, name: &str, source: Option<&str>) -> String {
        let program = self.program;

        // Compute reachable fns, closures, structs, and enums together so uncalled
        // stdlib functions, unused types, and their trait hooks are omitted.
        let mut reachable: BTreeSet<usize> = BTreeSet::new();
        let mut reachable_closures: BTreeSet<usize> = BTreeSet::new();
        let mut reachable_structs: BTreeSet<usize> = BTreeSet::new();
        let mut reachable_enums: BTreeSet<usize> = BTreeSet::new();
        let mut fn_work: Vec<usize> = Vec::new();
        let mut closure_work: Vec<usize> = Vec::new();

        fn mark_ty(
            ty: &Ty,
            program: &Program,
            reachable: &mut BTreeSet<usize>,
            fn_work: &mut Vec<usize>,
            reachable_structs: &mut BTreeSet<usize>,
            reachable_enums: &mut BTreeSet<usize>,
        ) {
            match ty {
                Ty::Struct(sid) => {
                    if reachable_structs.insert(*sid) {
                        let s = &program.structs[*sid];
                        for (_, fty) in &s.fields {
                            mark_ty(fty, program, reachable, fn_work, reachable_structs, reachable_enums);
                        }
                        for hook in [s.eq_fn, s.lt_fn].into_iter().flatten() {
                            if reachable.insert(hook) {
                                fn_work.push(hook);
                            }
                        }
                    }
                }
                Ty::Enum(eid) => {
                    if reachable_enums.insert(*eid) {
                        for (_, fields) in &program.enums[*eid].variants {
                            for fty in fields {
                                mark_ty(fty, program, reachable, fn_work, reachable_structs, reachable_enums);
                            }
                        }
                    }
                }
                Ty::Range(t)
                | Ty::Opt(t)
                | Ty::List(t)
                | Ty::Set(t)
                | Ty::HashSet(t)
                | Ty::Mut(t) => mark_ty(t, program, reachable, fn_work, reachable_structs, reachable_enums),
                Ty::Array { elem, .. } => {
                    mark_ty(elem, program, reachable, fn_work, reachable_structs, reachable_enums);
                }
                Ty::Map(k, v) | Ty::HashMap(k, v) => {
                    mark_ty(k, program, reachable, fn_work, reachable_structs, reachable_enums);
                    mark_ty(v, program, reachable, fn_work, reachable_structs, reachable_enums);
                }
                Ty::Fn(args, ret) => {
                    for a in args {
                        mark_ty(a, program, reachable, fn_work, reachable_structs, reachable_enums);
                    }
                    mark_ty(ret, program, reachable, fn_work, reachable_structs, reachable_enums);
                }
                Ty::Tuple(ts) => {
                    for t in ts {
                        mark_ty(t, program, reachable, fn_work, reachable_structs, reachable_enums);
                    }
                }
                Ty::Record(fs) => {
                    for (_, t) in fs {
                        mark_ty(t, program, reachable, fn_work, reachable_structs, reachable_enums);
                    }
                }
                _ => {}
            }
        }

        fn mark_show_ty(
            ty: &Ty,
            program: &Program,
            reachable: &mut BTreeSet<usize>,
            fn_work: &mut Vec<usize>,
            visited_structs: &mut BTreeSet<usize>,
            visited_enums: &mut BTreeSet<usize>,
        ) {
            match ty {
                Ty::Struct(sid) => {
                    if visited_structs.insert(*sid) {
                        let s = &program.structs[*sid];
                        if let Some(show_fn) = s.show_fn {
                            if reachable.insert(show_fn) {
                                fn_work.push(show_fn);
                            }
                        } else {
                            for (_, fty) in &s.fields {
                                mark_show_ty(fty, program, reachable, fn_work, visited_structs, visited_enums);
                            }
                        }
                    }
                }
                Ty::Enum(eid) => {
                    if visited_enums.insert(*eid) {
                        for (_, fields) in &program.enums[*eid].variants {
                            for fty in fields {
                                mark_show_ty(fty, program, reachable, fn_work, visited_structs, visited_enums);
                            }
                        }
                    }
                }
                Ty::Range(t)
                | Ty::Opt(t)
                | Ty::List(t)
                | Ty::Set(t)
                | Ty::HashSet(t)
                | Ty::Mut(t) => mark_show_ty(t, program, reachable, fn_work, visited_structs, visited_enums),
                Ty::Array { elem, .. } => {
                    mark_show_ty(elem, program, reachable, fn_work, visited_structs, visited_enums);
                }
                Ty::Map(k, v) | Ty::HashMap(k, v) => {
                    mark_show_ty(k, program, reachable, fn_work, visited_structs, visited_enums);
                    mark_show_ty(v, program, reachable, fn_work, visited_structs, visited_enums);
                }
                Ty::Tuple(ts) => {
                    for t in ts {
                        mark_show_ty(t, program, reachable, fn_work, visited_structs, visited_enums);
                    }
                }
                Ty::Record(fs) => {
                    for (_, t) in fs {
                        mark_show_ty(t, program, reachable, fn_work, visited_structs, visited_enums);
                    }
                }
                _ => {}
            }
        }

        let mut needs_tie = false;

        fn has_adt(ty: &Ty) -> bool {
            match ty {
                Ty::Struct(_) | Ty::Enum(_) => true,
                Ty::Range(t)
                | Ty::Opt(t)
                | Ty::List(t)
                | Ty::ListView(t)
                | Ty::Set(t)
                | Ty::HashSet(t)
                | Ty::Mut(t) => has_adt(t),
                Ty::Array { elem, .. } => has_adt(elem),
                Ty::Map(k, v) | Ty::HashMap(k, v) => has_adt(k) || has_adt(v),
                Ty::Tuple(ts) => ts.iter().any(has_adt),
                Ty::Record(fs) => fs.iter().any(|(_, t)| has_adt(t)),
                _ => false,
            }
        }

        let mut scan_block = |b: &Block,
                              reachable: &mut BTreeSet<usize>,
                              fn_work: &mut Vec<usize>,
                              reachable_closures: &mut BTreeSet<usize>,
                              closure_work: &mut Vec<usize>,
                              reachable_structs: &mut BTreeSet<usize>,
                              reachable_enums: &mut BTreeSet<usize>| {
            walk::deep_exprs(b, &mut |e| {
                mark_ty(&e.ty, program, reachable, fn_work, reachable_structs, reachable_enums);
                match &e.ty {
                    Ty::Set(k) | Ty::HashSet(k) | Ty::Map(k, _) | Ty::HashMap(k, _)
                        if has_adt(k) =>
                    {
                        needs_tie = true;
                    }
                    _ => {}
                }
                match &e.kind {
                    ExprKind::Call(fid, _) if reachable.insert(*fid) => fn_work.push(*fid),
                    ExprKind::Closure(cid) if reachable_closures.insert(*cid) => {
                        closure_work.push(*cid);
                    }
                    ExprKind::Cmp(_, left, _) if has_adt(&left.ty) => {
                        needs_tie = true;
                    }
                    ExprKind::Sort(place, by, _) => match by {
                        cpl_ir::SortBy::Natural if has_adt(&place.ty) => needs_tie = true,
                        cpl_ir::SortBy::Key(_, k) if has_adt(&k.ty) => needs_tie = true,
                        cpl_ir::SortBy::Fn { func, is_order: false } => {
                            if let Ty::Fn(_, ret) = &func.ty {
                                if has_adt(ret) {
                                    needs_tie = true;
                                }
                            }
                        }
                        _ => {}
                    },
                    ExprKind::Builtin(
                        cpl_ir::Builtin::Min
                        | cpl_ir::Builtin::Max
                        | cpl_ir::Builtin::Contains
                        | cpl_ir::Builtin::Sorted
                        | cpl_ir::Builtin::MapGet
                        | cpl_ir::Builtin::SetMin
                        | cpl_ir::Builtin::SetMax
                        | cpl_ir::Builtin::FirstGe
                        | cpl_ir::Builtin::FirstGt
                        | cpl_ir::Builtin::LastLe
                        | cpl_ir::Builtin::LastLt,
                        args,
                    ) if args.iter().any(|a| has_adt(&a.ty)) => {
                        needs_tie = true;
                    }
                    ExprKind::Print(args, _) => {
                        let (mut vs, mut ve) = (BTreeSet::new(), BTreeSet::new());
                        for a in args {
                            mark_show_ty(&a.value.ty, program, reachable, fn_work, &mut vs, &mut ve);
                        }
                    }
                    ExprKind::Format(pieces) => {
                        let (mut vs, mut ve) = (BTreeSet::new(), BTreeSet::new());
                        for p in pieces {
                            if let cpl_ir::FmtPiece::Value(a) = p {
                                mark_show_ty(&a.value.ty, program, reachable, fn_work, &mut vs, &mut ve);
                            }
                        }
                    }
                    ExprKind::Builtin(cpl_ir::Builtin::ShowStr | cpl_ir::Builtin::Join, args) => {
                        let (mut vs, mut ve) = (BTreeSet::new(), BTreeSet::new());
                        for a in args {
                            mark_show_ty(&a.ty, program, reachable, fn_work, &mut vs, &mut ve);
                        }
                    }
                    _ => {}
                }
            });
        };

        for g in &program.globals {
            mark_ty(&g.ty, program, &mut reachable, &mut fn_work, &mut reachable_structs, &mut reachable_enums);
        }
        for l in &program.main_locals {
            mark_ty(&l.ty, program, &mut reachable, &mut fn_work, &mut reachable_structs, &mut reachable_enums);
        }
        for &entry in &program.entry_fns {
            if reachable.insert(entry) {
                fn_work.push(entry);
            }
        }
        scan_block(
            &program.setup,
            &mut reachable,
            &mut fn_work,
            &mut reachable_closures,
            &mut closure_work,
            &mut reachable_structs,
            &mut reachable_enums,
        );
        scan_block(
            &program.main,
            &mut reachable,
            &mut fn_work,
            &mut reachable_closures,
            &mut closure_work,
            &mut reachable_structs,
            &mut reachable_enums,
        );
        while !fn_work.is_empty() || !closure_work.is_empty() {
            while let Some(fid) = fn_work.pop() {
                let f = &program.fns[fid];
                mark_ty(&f.ret, program, &mut reachable, &mut fn_work, &mut reachable_structs, &mut reachable_enums);
                for l in &f.locals {
                    mark_ty(&l.ty, program, &mut reachable, &mut fn_work, &mut reachable_structs, &mut reachable_enums);
                }
                scan_block(
                    &f.body,
                    &mut reachable,
                    &mut fn_work,
                    &mut reachable_closures,
                    &mut closure_work,
                    &mut reachable_structs,
                    &mut reachable_enums,
                );
            }
            while let Some(cid) = closure_work.pop() {
                let c = &program.closures[cid];
                mark_ty(&c.ret, program, &mut reachable, &mut fn_work, &mut reachable_structs, &mut reachable_enums);
                for l in &c.locals {
                    mark_ty(&l.ty, program, &mut reachable, &mut fn_work, &mut reachable_structs, &mut reachable_enums);
                }
                scan_block(
                    &c.body,
                    &mut reachable,
                    &mut fn_work,
                    &mut reachable_closures,
                    &mut closure_work,
                    &mut reachable_structs,
                    &mut reachable_enums,
                );
            }
        }

        // Ensure runtime types used by reachable globals, fns, closures, and main are registered.
        for g in &program.globals {
            self.need_ty(&g.ty);
        }
        for &fid in &reachable {
            let f = &program.fns[fid];
            self.need_ty(&f.ret);
            for l in &f.locals {
                self.need_ty(&l.ty);
            }
        }
        for &cid in &reachable_closures {
            let c = &program.closures[cid];
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
        for &id in &reachable_structs {
            self.type_def(&Ty::Struct(id), &mut done, &mut types);
        }
        for &id in &reachable_enums {
            self.type_def(&Ty::Enum(id), &mut done, &mut types);
        }
        let has_tie = needs_tie && types.contains("cpl_tie");

        let mut vars = String::new();
        for d in &program.domains {
            let d = ident(d);
            vars.push_str(&format!("ll {d}_lo = 0, {d}_len = 0;\n"));
        }
        for (id, global) in program.globals.iter().enumerate() {
            if global.ty == Ty::Unit {
                continue;
            }
            let cpp = self.ty(&global.ty);
            if let Some(value) = self.const_global(id) {
                vars.push_str(&format!("static constexpr {cpp} {} = {value};\n", ident(&global.name)));
                self.const_globals.insert(id);
                continue;
            }
            vars.push_str(&format!(
                "{} {}{};\n",
                cpp,
                ident(&global.name),
                self.zero_init(&global.ty)
            ));
        }

        let mut protos: Vec<Proto> = Vec::new();
        let mut fns = String::new();
        let mut judge_decls = String::new();
        let mut entries = String::new();
        for (id, f) in program.fns.iter().enumerate() {
            if !reachable.contains(&id) {
                continue;
            }
            let start = fns.len();
            let part = if program.library_fns.get(id).copied().unwrap_or(false) { crate::Part::Library } else { crate::Part::Yours };
            self.enter(Some(id), &f.body, f.params.len());
            let signature = self.signature(f);
            let hook = program.structs.iter().any(|s| [s.eq_fn, s.lt_fn, s.show_fn].contains(&Some(id)));
            protos.push(Proto {
                name: fn_name(&f.name),
                line: format!("{signature};\n"),
                at: fns.len(),
                always: hook || program.judge_fns.contains(&id),
            });
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
                self.chunks.push((part, f.name.clone(), fns[start..].to_string()));
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
            for d in program.fn_guarded_domains(&f.ret, &f.body) {
                self.need("dom_guard");
                let dn = ident(&program.domains[d]);
                let g = self.temp();
                fns.push_str(&format!("    cpl::DomGuard {g}({dn}_lo, {dn}_len);\n"));
            }
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
                protos.push(Proto {
                    name: format!("cpl_each_{name}"),
                    line: format!("{head};\n"),
                    at: fns.len(),
                    always: false,
                });
                fns.push_str(&format!("\n{head} {{\n"));
                self.reused.clear();
                self.pushing = Some(acc);
                self.hoisted_decls(&mut fns, f.params.len());
                self.block(&f.body, &mut fns);
                self.pushing = None;
                fns.push_str("}\n");
            }
            self.chunks.push((part, f.name.clone(), fns[start..].to_string()));
        }
        let start = fns.len();
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
        self.chunks.push((crate::Part::Yours, "top-level code".into(), fns[start..].to_string()));

        // `main` writes through the runtime; what reads asks for the input itself.
        self.need("out");
        if program.multitest {
            self.need(if program.until_eof { "at_eof" } else { "read_int" });
        }

        let mut out = String::new();
        out.push_str(&header(name, source, self.target.stamp.as_deref()));
        let what = if source.is_some() { "the source" } else { "the note" };
        self.chunks.push((crate::Part::Header, what.into(), out.clone()));
        let includes = out.len();
        // Closures that are called by name (see `devirt`): emit dependencies
        // before callers so higher-order combinators (e.g. `flip_action`) can
        // call another `cpl_fnM` regardless of closure id order.
        let mut direct = String::new();
        let mut emitted_direct: BTreeSet<usize> = BTreeSet::new();
        let mut visiting: Vec<(usize, bool)> = Vec::new();
        loop {
            let next = self
                .direct
                .iter()
                .copied()
                .find(|cid| !emitted_direct.contains(cid));
            let Some(root) = next else { break };
            visiting.push((root, false));
            while let Some((cid, ready)) = visiting.pop() {
                if emitted_direct.contains(&cid) {
                    continue;
                }
                if ready {
                    emitted_direct.insert(cid);
                    self.enter(None, &program.closures[cid].body, 0);
                    let body = self.emit_closure(cid, true).text;
                    direct.push_str(&format!(
                        "static auto {} = {body};\n",
                        crate::devirt::name(cid)
                    ));
                } else {
                    visiting.push((cid, true));
                    let mut deps = Vec::new();
                    crate::walk::deep_exprs(&program.closures[cid].body, &mut |e| {
                        if let cpl_ir::ExprKind::CallExpr(callee, _) = &e.kind {
                            if let Some(dep) = self.devirt.target(None, callee) {
                                if dep != cid && !emitted_direct.contains(&dep) {
                                    deps.push(dep);
                                }
                            }
                        }
                    });
                    for dep in deps {
                        self.direct.insert(dep);
                        visiting.push((dep, false));
                    }
                }
            }
        }
        out.push_str(
            "#include <algorithm>\n#include <array>\n#include <cmath>\n#include <cstdint>\n#include <cstdio>\n#include <cstdlib>\n\
             #include <map>\n#include <optional>\n#include <set>\n#include <string>\n#include <tuple>\n#include <vector>\n#include <unistd.h>\n",
        );
        if types.contains("std::function")
            || vars.contains("std::function")
            || fns.contains("std::function")
            || direct.contains("std::function")
        {
            out.push_str("#include <functional>\n");
        }
        if self.used.contains("str_view") {
            out.push_str("#include <string_view>\n");
        }
        if self.used.contains("cphash") {
            out.push_str("#include <chrono>\n");
        }
        out.push_str("\nusing ll = long long;\n\n");
        self.chunks.push((crate::Part::Includes, "#include".into(), out[includes..].to_string()));
        if !judge_decls.is_empty() {
            out.push_str("// The judge's grader supplies these.\n");
            out.push_str(&judge_decls);
            out.push('\n');
        }

        out.push_str(
            "// What the program below leans on.\nnamespace cpl {\nusing u8 = unsigned char;\n",
        );
        for (at, piece) in ["niche", "niche_bool", "niche_char", "niche_tuple", "niche_int", "niche_field"].into_iter().enumerate() {
            if self.niche_types.get() & 1 << at != 0 {
                self.need(piece);
            }
        }
        for piece in PIECES {
            if self.used.contains(piece.name) {
                out.push_str(piece.code);
                self.chunks.push((crate::Part::Runtime, piece.name.into(), piece.code.into()));
            }
        }
        out.push_str("}  // namespace cpl\n\n// The program.\nnamespace prog {\n");
        if has_tie {
            out.push_str(TIE_OPS);
            self.chunks.push((crate::Part::Runtime, "comparisons".into(), TIE_OPS.into()));
        }
        out.push_str(&types);
        self.chunks.push((crate::Part::Types, "types".into(), types.clone()));
        self.chunks.push((crate::Part::Globals, "globals".into(), vars.clone()));
        self.chunks.push((crate::Part::Yours, "closures".into(), direct.clone()));
        if !types.is_empty() {
            out.push('\n');
        }
        out.push_str(&vars);
        if !vars.is_empty() {
            out.push('\n');
        }
        // A fn is declared ahead only when something written above it
        // names it: a closure called by name, a `show`, or a fn that
        // comes first. The rest are defined before they are used.
        for proto in &protos {
            let above = [direct.as_str(), &fns[..proto.at]];
            let named = above.into_iter().chain(self.shows.iter().map(String::as_str)).any(|text| names(text, &proto.name));
            if proto.always || named {
                out.push_str(&proto.line);
                self.chunks.push((crate::Part::Glue, "declarations ahead".into(), proto.line.clone()));
            }
        }
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
            self.chunks.push((crate::Part::Runtime, "show".into(), format!("{show}\n")));
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
        out.push_str("}  // namespace prog\n\n");
        out.push_str(match self.target.thread_stack {
            Some(bytes) => format!("// Runs on a thread with a {} MB stack.\nstatic void cpl_main() {{\n", bytes >> 20),
            None => "// The judge allows no threads: the program has the stack `main` is given.\nint main() {\n".into(),
        }.as_str());
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
        match self.target.thread_stack {
            Some(bytes) => {
                out.push_str("    cpl::flush();\n}\n");
                // A Windows judge's file also builds here, with POSIX threads.
                let mut runner = match self.target.windows {
                    true => format!("\n#ifdef _WIN32{WINDOWS_THREAD}#else{POSIX_THREAD}#endif\n"),
                    false => POSIX_THREAD.to_string(),
                };
                runner.push_str(&THREAD_MAIN.replace("STACK_BYTES", &format!("(size_t){} << 20", bytes >> 20)));
                out.push_str(&runner);
                self.chunks.push((crate::Part::Runtime, "thread stack".into(), runner));
            }
            None => out.push_str("    cpl::flush();\n    return 0;\n}\n"),
        }
        out
    }

    /// The C++ literal of a global stored once, by its declaration, with an
    /// `Int` or `Bool` literal: a compile-time constant, so `% M` by it
    /// compiles to a multiply instead of a divide.
    fn const_global(&self, id: usize) -> Option<String> {
        let v = (None, true, id);
        if self.facts.stores(v) != 1 {
            return None;
        }
        match (&self.program.globals[id].ty, &self.facts.init(v)?.kind) {
            (Ty::Int, ExprKind::Int(n)) => Some(crate::expr::int_literal(*n).text),
            (Ty::Bool, ExprKind::Bool(b)) => Some(b.to_string()),
            _ => None,
        }
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

/// `main` for a judge that allows threads: the program runs on one with
/// the profile's stack, or the largest the judge reserves short of that.
/// The stack is address space: pages are committed as calls reach them.
const THREAD_MAIN: &str = "\
int main() {
    for (size_t n = STACK_BYTES; n >= 8 << 20; n /= 2)
        if (cpl_on(n)) return 0;
    cpl_main();
}
";

const WINDOWS_THREAD: &str = "
#include <windows.h>
static DWORD WINAPI cpl_go(LPVOID) { cpl_main(); return 0; }
static bool cpl_on(size_t n) {
    HANDLE t = CreateThread(0, n, cpl_go, 0, STACK_SIZE_PARAM_IS_A_RESERVATION, 0);
    return t && (WaitForSingleObject(t, INFINITE), true);
}
";

const POSIX_THREAD: &str = "
#include <pthread.h>
#if defined(__GLIBC__) && __GLIBC__ * 100 + __GLIBC_MINOR__ < 234
#pragma weak pthread_create  // the judge may not link libpthread
#define CPL_LINKED &pthread_create
#else
#define CPL_LINKED 1
#endif
static void* cpl_go(void*) { cpl_main(); return 0; }
static bool cpl_on(size_t n) {
    pthread_attr_t a;
    pthread_t t;
    return CPL_LINKED && !pthread_attr_init(&a) && !pthread_attr_setstacksize(&a, n) &&
           !pthread_create(&t, &a, cpl_go, 0) && (pthread_join(t, 0), true);
}
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
fn header(name: &str, source: Option<&str>, stamp: Option<&str>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let stamp = stamp.map(|stamp| format!("// For {stamp}.\n")).unwrap_or_default();
    let Some(source) = source else {
        return format!(
            "// {name}: C++17 written by cpl {version} from CP Lang source, which is left out\n\
             // here to keep this file within the size a judge accepts.\n{stamp}\n"
        );
    };
    let mut out =
        format!("// {name}: C++17 written by cpl {version} from the CP Lang source below.\n{stamp}//\n");
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
        Ty::StrView => "cpl::StrView".into(),
        Ty::List(elem) => format!("std::vector<{}>", judge_ty(elem)),
        Ty::ListView(elem) => format!("cpl::ListView<{}>", judge_ty(elem)),
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(judge_ty).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        _ => unreachable!("lowering lets only these types cross to the judge"),
    }
}

/// A fn's declaration, and where its definition starts in the text of
/// the fns.
struct Proto {
    name: String,
    line: String,
    at: usize,
    /// Named by text that is written elsewhere: a struct's operators, the
    /// judge's side.
    always: bool,
}

/// True when `text` has `name` as a whole word.
fn names(text: &str, name: &str) -> bool {
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    text.match_indices(name).any(|(at, _)| {
        let before = text.as_bytes()[..at].last().copied();
        let after = text.as_bytes().get(at + name.len()).copied();
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}
