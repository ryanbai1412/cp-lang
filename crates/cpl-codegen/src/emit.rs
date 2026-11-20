//! Printing a lowered program as one C++17 file.

use crate::names::{cty, fn_name, ident, is_plain_enum};
use crate::ranges::Facts;
use crate::runtime::PIECES;
use crate::walk::{self, Effects};
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

/// What a fn does beyond computing its result, counting the fns it calls.
#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct FnSummary {
    pub io: bool,
    pub reads: BTreeSet<usize>,
    pub writes: BTreeSet<usize>,
}

pub(crate) struct Emitter<'p> {
    pub program: &'p Program,
    pub facts: Facts<'p>,
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
    /// Tuple temporaries whose fields are moved out: see [`crate::scope::spent`].
    pub spent: BTreeSet<usize>,
    pub taken: BTreeSet<String>,
    pub fresh: usize,
    /// Loops that a `break` or `continue` names from an inner loop.
    pub break_targets: BTreeSet<LoopId>,
    pub continue_targets: BTreeSet<LoopId>,
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
            spent: BTreeSet::new(),
            taken: BTreeSet::new(),
            fresh: 0,
            break_targets: BTreeSet::new(),
            continue_targets: BTreeSet::new(),
            indent: 0,
        };
        emitter.summaries = emitter.summarize();
        emitter
    }

    /// What each fn reads, writes and prints, through the fns it calls.
    fn summarize(&self) -> Vec<FnSummary> {
        let direct: Vec<(FnSummary, BTreeSet<usize>)> = self
            .program
            .fns
            .iter()
            .map(|f| {
                let effects = Effects::of_block(&f.body);
                let mut summary = FnSummary {
                    writes: effects.globals.clone(),
                    ..FnSummary::default()
                };
                walk::deep_exprs(&f.body, &mut |e| match &e.kind {
                    ExprKind::Read(_) | ExprKind::Print(_) => summary.io = true,
                    ExprKind::Global(id) => {
                        summary.reads.insert(*id);
                    }
                    _ => {}
                });
                walk::deep_stmts(&f.body, &mut |stmt| {
                    // Sizing a domain changes what later code sees.
                    if matches!(stmt, Stmt::DefDomain { .. }) {
                        summary.io = true;
                    }
                });
                (summary, effects.calls)
            })
            .collect();
        let mut summaries: Vec<FnSummary> = direct.iter().map(|(s, _)| s.clone()).collect();
        loop {
            let before = summaries.clone();
            for (id, (_, calls)) in direct.iter().enumerate() {
                for callee in calls {
                    let other = before[*callee].clone();
                    summaries[id].io |= other.io;
                    summaries[id].reads.extend(other.reads);
                    summaries[id].writes.extend(other.writes);
                }
            }
            if summaries == before {
                return summaries;
            }
        }
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
        cty(self.program, ty)
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
        let params: Vec<String> = f
            .params
            .iter()
            .enumerate()
            .map(|(slot, param)| {
                let ty = &f.locals[slot].ty;
                let name = &self.names[slot];
                let cpp = self.ty(ty);
                if param.mutable {
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

    /// Declarations of the locals that could not be declared where they
    /// are first set.
    fn hoisted_decls(&mut self, out: &mut String, params: usize) {
        let locals = self.locals();
        for slot in self.hoisted.clone() {
            if slot < params || locals[slot].ty == Ty::Unit {
                continue;
            }
            let ty = &locals[slot].ty;
            out.push_str(&format!(
                "{}{} {}{};\n",
                pad(self.indent),
                self.ty(ty),
                self.names[slot],
                self.zero_init(ty)
            ));
        }
    }

    /// The initializer that gives a new variable its type's zero.
    pub fn zero_init(&self, ty: &Ty) -> &'static str {
        match ty {
            Ty::Int | Ty::Float | Ty::Char | Ty::Pos(_) | Ty::Bound(_) => " = 0",
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
                    out.push_str(&format!(
                        "    {} {}{};\n",
                        self.ty(field_ty),
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
        for global in &program.globals {
            if global.ty == Ty::Unit {
                continue;
            }
            vars.push_str(&format!(
                "{} {}{};\n",
                self.ty(&global.ty),
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
        for (id, f) in program.fns.iter().enumerate() {
            if !reachable.contains(&id) {
                continue;
            }
            self.enter(Some(id), &f.body, f.params.len());
            let signature = self.signature(f);
            protos.push_str(&format!("{signature};\n"));
            fns.push_str(&format!("\n{signature} {{\n"));
            self.indent = 1;
            self.hoisted_decls(&mut fns, f.params.len());
            self.block(&f.body, &mut fns);
            fns.push_str("}\n");
        }
        self.enter(None, &program.main, 0);
        fns.push_str("\nvoid solve() {\n");
        self.indent = 1;
        self.hoisted_decls(&mut fns, 0);
        self.block(&program.main, &mut fns);
        fns.push_str("}\n");

        // `main` reads and writes through the runtime.
        self.need("in");
        self.need("out");
        if program.multitest {
            self.need("read_int");
        }

        let mut out = String::new();
        out.push_str(&header(name, source));
        out.push_str(
            "#include <algorithm>\n#include <array>\n#include <cmath>\n#include <cstdint>\n#include <cstdio>\n#include <cstdlib>\n\
             #include <map>\n#include <optional>\n#include <set>\n#include <string>\n#include <tuple>\n#include <vector>\n",
        );
        if !program.closures.is_empty() {
            out.push_str("#include <functional>\n");
        }
        if self.used.contains("cphash") {
            out.push_str("#include <chrono>\n#include <unordered_map>\n#include <unordered_set>\n");
        }
        out.push_str("\nusing ll = long long;\n\n");
        out.push_str(
            "// What the program below leans on.\nnamespace cpl {\nusing u8 = unsigned char;\n",
        );
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
        out.push_str("}  // namespace prog\n\nint main() {\n");
        out.push_str(if program.interactive {
            "    cpl::live = true;\n"
        } else {
            "    cpl::read_all();\n"
        });
        if program.multitest {
            out.push_str(
                "    for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();\n",
            );
        } else {
            out.push_str("    prog::solve();\n");
        }
        out.push_str("    cpl::flush();\n    return 0;\n}\n");
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
