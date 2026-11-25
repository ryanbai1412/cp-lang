//! Contract 7: the C++ emitter. A printer over canonical IR: one C++ shape per
//! node kind, and no decision of its own.
use crate::inst::{CTy, Program};
use crate::ir::*;
use std::collections::{HashMap, HashSet};

/// Appends formatted text to a `String`.
macro_rules! put {
    ($s:expr, $($arg:tt)*) => {
        $s.push_str(&format!($($arg)*))
    };
}
/// `put!`, then a newline.
macro_rules! put_line {
    ($s:expr, $($arg:tt)*) => {{
        put!($s, $($arg)*);
        $s.push('\n');
    }};
}

const RUNTIME: &str = r#"#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cerrno>
#include <pthread.h>
#include <csignal>
#include <cstring>
#include <unistd.h>
#include <climits>
#include <vector>
struct cpl_unit {};
template <class T> using cpl_raw = std::vector<T>;
// The innermost line of the user's file that is running (`ir::Site`).
static int cpl_at = 0;
[[noreturn]] static void cpl_trap(const char* code, const char* msg) {
    fflush(stdout);
    fprintf(stderr, "trap: %s: line %d: %s\n", code, cpl_at, msg);
    exit(3);
}
static char cpl_altstack[1 << 16];
static void cpl_segv(int) {
    // Only `write` is safe here, so the line is printed by hand.
    char m[64] = "trap: R0601: line ";
    char d[12]; int n = 0, k = (int)strlen(m);
    for (int v = cpl_at; v || !n; v /= 10) d[n++] = (char)('0' + v % 10);
    while (n) m[k++] = d[--n];
    static const char t[] = ": the stack ran out\n";
    memcpy(m + k, t, sizeof t - 1);
    if (write(2, m, k + sizeof t - 1)) {}
    _exit(3);
}
static void cpl_guard_stack() {
#ifdef __SANITIZE_ADDRESS__
    return; // the sanitizer owns the signal stack
#endif
    stack_t ss; ss.ss_sp = cpl_altstack; ss.ss_size = sizeof cpl_altstack; ss.ss_flags = 0;
    sigaltstack(&ss, nullptr);
    struct sigaction sa; memset(&sa, 0, sizeof sa); sa.sa_handler = cpl_segv; sa.sa_flags = SA_ONSTACK;
    sigaction(SIGSEGV, &sa, nullptr); sigaction(SIGBUS, &sa, nullptr);
}
static inline int64_t cpl_add(int64_t a, int64_t b) { int64_t r; if (__builtin_add_overflow(a, b, &r) || r == INT64_MIN) cpl_trap("R0202", "integer overflow"); return r; }
static inline int64_t cpl_sub(int64_t a, int64_t b) { int64_t r; if (__builtin_sub_overflow(a, b, &r) || r == INT64_MIN) cpl_trap("R0202", "integer overflow"); return r; }
static inline int64_t cpl_mul(int64_t a, int64_t b) { int64_t r; if (__builtin_mul_overflow(a, b, &r) || r == INT64_MIN) cpl_trap("R0202", "integer overflow"); return r; }
static inline int64_t cpl_neg(int64_t a) { if (a == INT64_MIN) cpl_trap("R0202", "integer overflow"); return -a; }
static inline int64_t cpl_div(int64_t a, int64_t b) {
    if (b == 0) cpl_trap("R0203", "division by zero");
    if (a == INT64_MIN && b == -1) cpl_trap("R0202", "integer overflow");
    int64_t q = a / b, r = a % b;
    if (r < 0) q += b > 0 ? -1 : 1;
    return q;
}
static inline int64_t cpl_rem(int64_t a, int64_t b) {
    if (b == 0) cpl_trap("R0203", "division by zero");
    if (a == INT64_MIN && b == -1) cpl_trap("R0202", "integer overflow");
    int64_t r = a % b;
    if (r < 0) r += b > 0 ? b : -b;
    return r;
}
template <class T> static inline cpl_raw<T> cpl_raw_new(int64_t n, const T& fill) {
    if (n < 0) cpl_trap("R0001", "negative storage size");
    if (n > (int64_t(1) << 28)) cpl_trap("R0001", "storage larger than the memory limit");
    return cpl_raw<T>((size_t)n, fill);
}
template <class T> static inline T cpl_get(const cpl_raw<T>& r, int64_t i) {
    if (i < 0 || (size_t)i >= r.size()) cpl_trap("R0101", "read outside storage");
    return r[(size_t)i];
}
template <class T> static inline cpl_unit cpl_set(cpl_raw<T>& r, int64_t i, const T& x) {
    if (i < 0 || (size_t)i >= r.size()) cpl_trap("R0101", "write outside storage");
    r[(size_t)i] = x;
    return cpl_unit{};
}
static inline int64_t cpl_read() {
    char buf[32];
    if (scanf("%31s", buf) != 1) cpl_trap("R0501", "expected an integer in the input");
    errno = 0;
    char* end;
    long long x = strtoll(buf, &end, 10);
    if (*end || end == buf || errno == ERANGE || x == LLONG_MIN) cpl_trap("R0501", "expected an integer in the input");
    return (int64_t)x;
}
// The caller's code handed to a generator or lender that calls itself: two words, not owning.
// Used only there; elsewhere the callee is a template and the code is inlined.
template <class T> struct cpl_body {
    void* env;
    int (*fn)(void*, T);
    template <class L> cpl_body(L& l) : env(&l), fn([](void* e, T x) -> int { return (*static_cast<L*>(e))(static_cast<T>(x)); }) {}
    int operator()(T x) const { return fn(env, static_cast<T>(x)); }
};
static inline void cpl_ev(int64_t x) { fprintf(stderr, " %lld", (long long)x); }
static inline void cpl_ev(bool b) { fputs(b ? " true" : " false", stderr); }
static inline void cpl_out(int64_t x) { printf("%lld", (long long)x); }
static inline void cpl_out(bool b) { fputs(b ? "true" : "false", stdout); }
"#;

/// What `break`, `continue` and `return` mean at a point in the output.
#[derive(Clone, Copy)]
struct Ctx {
    /// The innermost thing a `break` leaves: a loop, or a region body (`true`: a lender's).
    inner: Inner,
    /// Inside a region body, which is a C++ lambda: `return` cannot leave the fn directly.
    in_lambda: bool,
    /// The fn being printed is a generator or a lender.
    fn_region: bool,
}
#[derive(Clone, Copy)]
enum Inner {
    None,
    Loop(usize),
    Region(bool),
}

struct Emit<'a> {
    p: &'a Program,
    funcs: &'a [Func<CTy>],
    /// Generators and lenders that can reach themselves.
    recursive: Vec<bool>,
    names: HashMap<CTy, String>,
    order: Vec<CTy>,
}

pub fn emit(p: &Program, funcs: &[Func<CTy>]) -> String {
    let mut e = Emit {
        p,
        funcs,
        recursive: recursive_regions(funcs),
        names: HashMap::new(),
        order: Vec::new(),
    };
    for f in funcs {
        for t in &f.locals {
            e.name(t);
        }
        e.name(&f.ret);
        if let Some((_, t)) = &f.region {
            e.name(t);
        }
    }
    for t in &p.globals {
        e.name(t);
    }
    e.order = e.by_value_order();
    let mut s = String::from(RUNTIME);
    for t in &e.order {
        put_line!(s, "struct {};", e.names[t]);
    }
    for t in e.order.clone() {
        let n = e.names[&t].clone();
        match &t {
            CTy::Struct(..) => {
                let fs: Vec<String> = p.structs[&t]
                    .iter()
                    .enumerate()
                    .map(|(i, f)| format!("{} f{i};", e.names[f]))
                    .collect();
                put_line!(s, "struct {n} {{ {} }};", fs.join(" "));
            }
            CTy::Clo(k) => {
                let fs: Vec<String> = p
                    .captures(k)
                    .iter()
                    .enumerate()
                    .map(|(i, f)| format!("{} f{i};", e.names[f]))
                    .collect();
                put_line!(s, "struct {n} {{ {} }};", fs.join(" "));
            }
            CTy::Set(ms) => {
                let mut body = String::from("int64_t tag;");
                for (i, k) in ms.iter().enumerate() {
                    put!(body, " {} v{i};", e.names[&CTy::Clo(Box::new(k.clone()))]);
                }
                for (i, k) in ms.iter().enumerate() {
                    put!(body, " static {n} m{i}(const {}& x) {{ {n} u{{}}; u.tag = {i}; u.v{i} = x; return u; }}", e.names[&CTy::Clo(Box::new(k.clone()))]);
                }
                put_line!(s, "struct {n} {{ {body} }};");
            }
            _ => {}
        }
    }
    for (i, t) in p.globals.iter().enumerate() {
        put_line!(s, "static {} g{i}{{}};", e.names[t]);
    }
    let by_ref: Vec<Vec<bool>> = funcs.iter().map(const_ref_params).collect();
    for (i, f) in funcs.iter().enumerate() {
        put_line!(s, "{};", e.sig(i, f, &by_ref[i]));
    }
    for (i, f) in funcs.iter().enumerate() {
        put_line!(s, "// {}", f.name);
        put_line!(s, "{} {{", e.sig(i, f, &by_ref[i]));
        for (l, t) in f.locals.iter().enumerate() {
            if !f.params.iter().any(|(p, _)| *p == l) {
                put_line!(s, "    {} l{l}{{}};", e.names[t]);
            }
        }
        let mut label = 0;
        let ctx = Ctx {
            inner: Inner::None,
            in_lambda: false,
            fn_region: f.region.is_some(),
        };
        if f.region.is_some() {
            put_line!(s, "    int cpl_code = 0;");
        } else {
            put_line!(s, "    {} cpl_ret{{}};", e.names[&f.ret]);
        }
        e.block(&f.body, 1, &mut s, &mut label, ctx);
        if f.region.is_some() {
            put_line!(s, "    return cpl_code;");
        } else if f.ret == CTy::Unit {
            put_line!(s, "    return cpl_unit{{}};");
        } else {
            put_line!(s, "    __builtin_unreachable();");
        }
        put_line!(s, "}}");
    }
    // Spec 9.6: `main` runs on a thread with a 1 GB stack.
    put_line!(s, "static void* cpl_main(void*) {{ cpl_guard_stack(); f{}(); fflush(stdout); return nullptr; }}", p.main);
    put_line!(
        s,
        "int main() {{ pthread_attr_t a; pthread_t t; pthread_attr_init(&a); pthread_attr_setstacksize(&a, (size_t)1 << 30);\n    if (pthread_create(&t, &a, cpl_main, nullptr)) {{ cpl_main(nullptr); return 0; }}\n    pthread_join(t, nullptr); return 0; }}"
    );
    s
}

/// A parameter that is never written is passed by `const&`: passing a list does not copy it.
fn const_ref_params(f: &Func<CTy>) -> Vec<bool> {
    fn root(p: &Place) -> Option<usize> {
        match p {
            Place::Local(l) => Some(*l),
            Place::Global(_) => None,
            Place::Field(b, _) | Place::Variant(b, _) | Place::Deref(b) => root(b),
        }
    }
    fn expr(e: &Expr<CTy>, w: &mut Vec<usize>) {
        match &e.kind {
            EK::Intr(_, args, _) | EK::Call(_, args, _) => {
                for a in args {
                    match a {
                        Arg::Lend(p) => w.extend(root(p)),
                        Arg::Value(v) => expr(v, w),
                    }
                }
            }
            _ => {}
        }
    }
    fn block(b: &[Stmt<CTy>], w: &mut Vec<usize>) {
        for s in b {
            match s {
                Stmt::Let(l, e) => {
                    w.push(*l);
                    expr(e, w);
                }
                Stmt::Assign(p, e) => {
                    w.extend(root(p));
                    expr(e, w);
                }
                Stmt::Expr(e) => expr(e, w),
                Stmt::If(_, a, b2) | Stmt::Loop(a, b2) => {
                    block(a, w);
                    block(b2, w);
                }
                Stmt::Switch(_, arms) => arms.iter().for_each(|a| block(a, w)),
                Stmt::LendOut(p) => w.extend(root(p)),
                Stmt::Region {
                    args, var, body, ..
                } => {
                    w.push(*var);
                    for a in args {
                        if let Arg::Lend(p) = a {
                            w.extend(root(p));
                        }
                    }
                    block(body, w);
                }
                _ => {}
            }
        }
    }
    let mut written = Vec::new();
    block(&f.body, &mut written);
    f.params
        .iter()
        .map(|(l, lent)| {
            !lent
                && !written.contains(l)
                && !matches!(f.locals[*l], CTy::Int | CTy::Bool | CTy::Unit | CTy::Dom(_))
        })
        .collect()
}

impl Emit<'_> {
    /// Definition order for C++: a type comes after every type it holds by value. Storage
    /// (`cpl_raw<T>`) and references need only T's declaration, which is what lets a
    /// function value reach itself through a list.
    fn by_value_order(&self) -> Vec<CTy> {
        fn visit(e: &Emit, t: &CTy, done: &mut HashSet<CTy>, out: &mut Vec<CTy>) {
            let parts: Vec<CTy> = match t {
                CTy::Struct(..) => e.p.structs[t].clone(),
                CTy::Clo(k) => e.p.captures(k),
                CTy::Set(ms) => ms.iter().map(|k| CTy::Clo(Box::new(k.clone()))).collect(),
                _ => return,
            };
            if !done.insert(t.clone()) {
                return;
            }
            for f in &parts {
                visit(e, f, done, out);
            }
            out.push(t.clone());
        }
        let (mut done, mut out) = (HashSet::new(), Vec::new());
        for t in &self.order {
            visit(self, t, &mut done, &mut out);
        }
        out
    }
    fn name(&mut self, t: &CTy) -> String {
        if let Some(n) = self.names.get(t) {
            return n.clone();
        }
        let n = match t {
            CTy::Int | CTy::Dom(_) | CTy::Error => "int64_t".to_string(),
            CTy::Bool => "bool".to_string(),
            CTy::Unit => "cpl_unit".to_string(),
            CTy::Raw(e) => format!("cpl_raw<{}>", self.name(e)),
            CTy::Ref(e) => format!("{}*", self.name(e)),
            CTy::Struct(..) => {
                let n = format!("S{}", self.names.len());
                self.names.insert(t.clone(), n.clone());
                for f in self.p.structs[t].clone() {
                    self.name(&f);
                }
                self.order.push(t.clone());
                n
            }
            CTy::Clo(k) => {
                let n = format!("C{}", self.names.len());
                self.names.insert(t.clone(), n.clone());
                for f in self.p.captures(k) {
                    self.name(&f);
                }
                self.order.push(t.clone());
                n
            }
            CTy::Set(ms) => {
                let n = format!("U{}", self.names.len());
                self.names.insert(t.clone(), n.clone());
                for k in ms {
                    self.name(&CTy::Clo(Box::new(k.clone())));
                }
                self.order.push(t.clone());
                n
            }
        };
        self.names.insert(t.clone(), n.clone());
        n
    }
    /// Spec R1: a value with no heap part is copied when passed, so the callee never sees it change.
    fn plain(&self, t: &CTy, depth: usize) -> bool {
        match t {
            CTy::Int | CTy::Bool | CTy::Unit | CTy::Dom(_) => true,
            CTy::Struct(..) if depth < 8 => self
                .p
                .structs
                .get(t)
                .is_some_and(|fs| fs.iter().all(|f| self.plain(f, depth + 1))),
            _ => false,
        }
    }
    fn sig(&self, i: usize, f: &Func<CTy>, by_ref: &[bool]) -> String {
        let ps: Vec<String> = f
            .params
            .iter()
            .zip(by_ref)
            .map(|((l, lent), cref)| {
                let t = &self.names[&f.locals[*l]];
                if *lent {
                    format!("{t}& l{l}")
                } else if *cref && !self.plain(&f.locals[*l], 0) {
                    format!("{t} const& l{l}")
                } else {
                    format!("{t} l{l}")
                }
            })
            .collect();
        if let Some((lend, t)) = &f.region {
            let mut ps = ps;
            if self.recursive[i] {
                ps.push(format!(
                    "cpl_body<{}{}> body",
                    self.names[t],
                    if *lend { "&" } else { "" }
                ));
                return format!("static int f{i}({})", ps.join(", "));
            }
            ps.push("B&& body".to_string());
            return format!("template <class B> static int f{i}({})", ps.join(", "));
        }
        format!("static {} f{i}({})", self.names[&f.ret], ps.join(", "))
    }
    /// The type a generator or lender hands to the caller's code.
    fn locals_of(&self, f: usize, _lend: bool) -> CTy {
        self.funcs[f]
            .region
            .as_ref()
            .map(|(_, t)| t.clone())
            .unwrap_or(CTy::Unit)
    }
    fn place(&self, p: &Place) -> String {
        match p {
            Place::Local(l) => format!("l{l}"),
            Place::Global(g) => format!("g{g}"),
            Place::Field(b, i) => format!("{}.f{i}", self.place(b)),
            Place::Variant(b, i) => format!("{}.v{i}", self.place(b)),
            Place::Deref(b) => format!("(*{})", self.place(b)),
        }
    }
    fn arg(&self, a: &Arg<CTy>) -> String {
        match a {
            Arg::Value(v) => self.expr(v),
            Arg::Lend(p) => self.place(p),
        }
    }
    fn expr(&self, e: &Expr<CTy>) -> String {
        let list = |xs: &[Expr<CTy>]| {
            xs.iter()
                .map(|x| self.expr(x))
                .collect::<Vec<_>>()
                .join(", ")
        };
        match &e.kind {
            EK::Int(i) if *i == i64::MIN => "INT64_MIN".to_string(),
            EK::Int(i) => format!("INT64_C({i})"),
            EK::Bool(b) => b.to_string(),
            EK::Unit => "cpl_unit{}".to_string(),
            EK::DomLen(_) => "/* unresolved domain size */".to_string(),
            EK::Copy(p) => self.place(p),
            EK::AddrOf(p) => format!("(&{})", self.place(p)),
            EK::Tag(p) => format!("{}.tag", self.place(p)),
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                format!("{}{{{}}}", self.names[&e.ty], list(xs))
            }
            EK::MakeUnion(i, x) => format!("{}::m{i}({})", self.names[&e.ty], self.expr(x)),
            EK::Inject(x) | EK::Widen(x) => self.expr(x),
            EK::Call(Target::Inst(f), args, _) => format!(
                "f{f}({})",
                args.iter()
                    .map(|a| self.arg(a))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            EK::Call(..) | EK::CallValue(..) => "/* not canonical */".to_string(),
            EK::Intr(op, args, _) => {
                let a: Vec<String> = args.iter().map(|a| self.arg(a)).collect();
                let bin = |o: &str| format!("({} {o} {})", a[0], a[1]);
                match op {
                    Intr::Add => format!("cpl_add({}, {})", a[0], a[1]),
                    Intr::Sub => format!("cpl_sub({}, {})", a[0], a[1]),
                    Intr::Mul => format!("cpl_mul({}, {})", a[0], a[1]),
                    Intr::Div => format!("cpl_div({}, {})", a[0], a[1]),
                    Intr::Rem => format!("cpl_rem({}, {})", a[0], a[1]),
                    Intr::Neg => format!("cpl_neg({})", a[0]),
                    Intr::BitAnd => bin("&"),
                    Intr::BitOr => bin("|"),
                    Intr::BitXor => bin("^"),
                    Intr::Eq => bin("=="),
                    Intr::Ne => bin("!="),
                    Intr::Lt => bin("<"),
                    Intr::Le => bin("<="),
                    Intr::Gt => bin(">"),
                    Intr::Ge => bin(">="),
                    Intr::Not => format!("(!{})", a[0]),
                    Intr::RawEmpty => format!("{}{{}}", self.names[&e.ty]),
                    Intr::RawNew => format!("cpl_raw_new({}, {})", a[0], a[1]),
                    Intr::RawGet => format!("cpl_get({}, {})", a[0], a[1]),
                    Intr::RawSet => format!("cpl_set({}, {}, {})", a[0], a[1], a[2]),
                    Intr::ReadInt => "cpl_read()".to_string(),
                }
            }
        }
    }
    fn leave(&self, what: &str, ctx: Ctx) -> String {
        match (what, ctx.inner) {
            ("break", Inner::Loop(_)) => "break;".to_string(),
            ("break", Inner::Region(_)) => "return 1;".to_string(),
            ("continue", Inner::Loop(l)) => format!("goto next{l};"),
            ("continue", Inner::Region(false)) => "return 0;".to_string(),
            ("continue", Inner::Region(true)) => "return 3;".to_string(),
            // A `return` that was carried out of a region body.
            ("return", _) if ctx.in_lambda => "return 2;".to_string(),
            ("return", _) if ctx.fn_region => "return cpl_code;".to_string(),
            ("return", _) => "return cpl_ret;".to_string(),
            _ => "/* nothing to leave */;".to_string(),
        }
    }
    fn block(&self, b: &[Stmt<CTy>], depth: usize, s: &mut String, label: &mut usize, ctx: Ctx) {
        let pad = "    ".repeat(depth);
        for st in b {
            // `cpl_at` is set where the interpreter sets `Machine::at`: at an operation that can
            // trap, and around a call, which may run other lines of the user's file.
            let around = match st {
                Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) => match &e.kind {
                    EK::Intr(op, _, site) if op.checked() && *site != 0 => {
                        put_line!(s, "{pad}cpl_at = {site};");
                        None
                    }
                    EK::Call(_, _, site) => Some(*site),
                    _ => None,
                },
                Stmt::Region { site, .. } => Some(*site),
                Stmt::Yield(_) | Stmt::LendOut(_) => Some(0),
                _ => None,
            };
            match around {
                Some(0) => put_line!(s, "{pad}{{ int cpl_was = cpl_at;"),
                Some(site) => put_line!(s, "{pad}{{ const int cpl_was = cpl_at = {site};"),
                None => {}
            }
            match st {
                Stmt::Let(l, e) => put_line!(s, "{pad}l{l} = {};", self.expr(e)),
                Stmt::Assign(p, e) => put_line!(s, "{pad}{} = {};", self.place(p), self.expr(e)),
                Stmt::Expr(e) => put_line!(s, "{pad}(void)({});", self.expr(e)),
                Stmt::If(c, a, b2) => {
                    put_line!(s, "{pad}if ({}) {{", self.expr(c));
                    self.block(a, depth + 1, s, label, ctx);
                    if !b2.is_empty() {
                        put_line!(s, "{pad}}} else {{");
                        self.block(b2, depth + 1, s, label, ctx);
                    }
                    put_line!(s, "{pad}}}");
                }
                Stmt::Loop(body, step) => {
                    *label += 1;
                    let me = *label;
                    put_line!(s, "{pad}for (;;) {{");
                    self.block(
                        body,
                        depth + 1,
                        s,
                        label,
                        Ctx {
                            inner: Inner::Loop(me),
                            ..ctx
                        },
                    );
                    put_line!(s, "{pad}    next{me}:;");
                    self.block(
                        step,
                        depth + 1,
                        s,
                        label,
                        Ctx {
                            inner: Inner::None,
                            ..ctx
                        },
                    );
                    put_line!(s, "{pad}}}");
                }
                // Printed as a chain of tests, so a `break` inside an arm still leaves the loop around it.
                Stmt::Switch(c, arms) => {
                    for (i, a) in arms.iter().enumerate() {
                        let head = if i == 0 {
                            format!("{pad}if")
                        } else {
                            format!("{pad}}} else if")
                        };
                        put_line!(s, "{head} ({} == {i}) {{", self.expr(c));
                        self.block(a, depth + 1, s, label, ctx);
                    }
                    if !arms.is_empty() {
                        put_line!(s, "{pad}}}");
                    }
                }
                Stmt::Break => put_line!(s, "{pad}{}", self.leave("break", ctx)),
                Stmt::Continue => put_line!(s, "{pad}{}", self.leave("continue", ctx)),
                Stmt::Return(e) => {
                    if ctx.in_lambda || ctx.fn_region {
                        if let (Some(e), false) = (e, ctx.fn_region) {
                            put_line!(s, "{pad}cpl_ret = {};", self.expr(e));
                        }
                        put_line!(s, "{pad}{}", self.leave("return", ctx));
                    } else {
                        match e {
                            Some(e) => put_line!(s, "{pad}return {};", self.expr(e)),
                            None => put_line!(s, "{pad}return cpl_unit{{}};"),
                        }
                    }
                }
                // The caller's code is a C++ lambda the callee calls. Its result says how it left:
                // 0 went on, 1 `break`, 2 `return`, 3 `continue` of a loop around a lend.
                Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body,
                    ..
                } => {
                    let Target::Inst(f) = target else { continue };
                    let mut a: Vec<String> = args.iter().map(|a| self.arg(a)).collect();
                    let vt = &self.names[&self.locals_of(*f, *lend)];
                    let mut lam = String::new();
                    self.block(
                        body,
                        depth + 2,
                        &mut lam,
                        label,
                        Ctx {
                            inner: Inner::Region(*lend),
                            in_lambda: true,
                            fn_region: ctx.fn_region,
                        },
                    );
                    put_line!(s, "{pad}{{");
                    put_line!(s, "{pad}    auto code = [&]({vt}{} l{var}) -> int {{\n{lam}{pad}        return 0;\n{pad}    }};", if *lend { "&" } else { "" });
                    a.push("code".to_string());
                    put_line!(s, "{pad}    int c = f{f}({});", a.join(", "));
                    put_line!(s, "{pad}    cpl_at = cpl_was;");
                    put_line!(s, "{pad}    if (c == 2) {}", self.leave("return", ctx));
                    if *lend && !matches!(ctx.inner, Inner::None) {
                        put_line!(s, "{pad}    if (c == 1) {}", self.leave("break", ctx));
                        put_line!(s, "{pad}    if (c == 3) {}", self.leave("continue", ctx));
                    }
                    put_line!(s, "{pad}}}");
                }
                // A generator stops as soon as the caller's code leaves.
                Stmt::Yield(e) if ctx.in_lambda => put_line!(
                    s,
                    "{pad}{{ int c = body({}); cpl_at = cpl_was; if (c) {{ cpl_code = c; return 2; }} }}",
                    self.expr(e)
                ),
                Stmt::Yield(e) => put_line!(
                    s,
                    "{pad}{{ int c = body({}); cpl_at = cpl_was; if (c) return c; }}",
                    self.expr(e)
                ),
                // Events go to stderr in order; stdout is flushed first so the two streams line up.
                Stmt::Event(label, vals) => {
                    put_line!(s, "{pad}fputs({:?}, stderr);", format!("ev {label}"));
                    for v in vals {
                        put_line!(s, "{pad}cpl_ev({});", self.expr(v));
                    }
                    put_line!(s, "{pad}fputc('\\n', stderr);");
                }
                // A lender finishes its own code first; the caller leaves after it.
                Stmt::LendOut(p) => put_line!(s, "{pad}cpl_code = body({});", self.place(p)),
                Stmt::Check {
                    kind,
                    cond,
                    msg,
                    site,
                } => put_line!(
                    s,
                    "{pad}if (!{}) {{ {}cpl_trap({:?}, {msg:?}); }}",
                    self.expr(cond),
                    if *site != 0 {
                        format!("cpl_at = {site}; ")
                    } else {
                        String::new()
                    },
                    match kind {
                        CheckKind::Assert => cpl_diag::codes::ASSERTION_FAILED.0,
                        CheckKind::Domain => cpl_diag::codes::DOMAIN_CAST.0,
                    }
                ),
                Stmt::Print(ps) => {
                    for p in ps {
                        match p {
                            PrintPiece::Text(t) => put_line!(
                                s,
                                "{pad}fwrite({}, 1, {}, stdout);",
                                c_string(t),
                                t.len()
                            ),
                            PrintPiece::Value(v) => put_line!(s, "{pad}cpl_out({});", self.expr(v)),
                        }
                    }
                }
            }
            if around.is_some() {
                put_line!(s, "{pad}cpl_at = cpl_was; }}");
            }
        }
    }
}

/// A generator or lender that can call itself cannot take the caller's code as a template
/// parameter: each level would be a new type. Those take it as two words.
fn recursive_regions(funcs: &[Func<CTy>]) -> Vec<bool> {
    fn edges(b: &[Stmt<CTy>], out: &mut Vec<usize>) {
        fn ex(e: &Expr<CTy>, out: &mut Vec<usize>) {
            if let EK::Call(Target::Inst(j), _, _) = &e.kind {
                out.push(*j);
            }
        }
        for s in b {
            match s {
                Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) => ex(e, out),
                Stmt::If(_, a, b2) | Stmt::Loop(a, b2) => {
                    edges(a, out);
                    edges(b2, out);
                }
                Stmt::Switch(_, arms) => arms.iter().for_each(|a| edges(a, out)),
                Stmt::Region { target, body, .. } => {
                    if let Target::Inst(j) = target {
                        out.push(*j);
                    }
                    edges(body, out);
                }
                _ => {}
            }
        }
    }
    let graph: Vec<Vec<usize>> = funcs
        .iter()
        .map(|f| {
            let mut out = Vec::new();
            edges(&f.body, &mut out);
            out
        })
        .collect();
    (0..funcs.len())
        .map(|i| {
            if funcs[i].region.is_none() {
                return false;
            }
            let mut seen = vec![false; funcs.len()];
            let mut stack = graph[i].clone();
            while let Some(j) = stack.pop() {
                if j == i {
                    return true;
                }
                if !std::mem::replace(&mut seen[j], true) {
                    stack.extend(graph[j].iter().copied());
                }
            }
            false
        })
        .collect()
}

/// A C string literal for these bytes: every byte that is not plain printable ASCII is a
/// three-digit octal escape, so the literal means the same bytes whatever follows it.
fn c_string(t: &str) -> String {
    let mut o = String::from("\"");
    for b in t.bytes() {
        match b {
            b'"' => o.push_str("\\\""),
            b'\\' => o.push_str("\\\\"),
            b'?' => o.push_str("\\?"),
            0x20..=0x7e => o.push(b as char),
            _ => o.push_str(&format!("\\{b:03o}")),
        }
    }
    o.push('"');
    o
}
