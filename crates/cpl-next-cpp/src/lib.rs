//! Contract 7: the C++ emitter. A printer over canonical IR: one C++ shape per
//! node kind, and no decision of its own.
use cpl_next_ir::ir::*;
use cpl_next_ir::ops;
use cpl_next_ir::prog::{CTy, Program};
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

/// The runtime up to `cpl_trap`.
const RUNTIME_HEAD: &str = r#"#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cerrno>
#include <pthread.h>
#include <csignal>
#include <cstring>
#include <unistd.h>
#include <climits>
#include <vector>
#include <string>
struct cpl_unit {};
struct cpl_char { uint8_t code; };
/// A Bool cell: `std::vector<bool>` packs bits, so its cells could not be lent as `bool&`.
struct cpl_bool_cell {
    bool v;
    cpl_bool_cell(bool b = false) : v(b) {}
    operator bool&() { return v; }
    operator bool() const { return v; }
};
template <class T> struct cpl_raw : std::vector<T> {
    using std::vector<T>::vector;
};
template <> struct cpl_raw<bool> : std::vector<cpl_bool_cell> {
    using std::vector<cpl_bool_cell>::vector;
};
// The innermost line of the user's file that is running (`ir::Site`).
static int cpl_at = 0;
[[noreturn]] static void cpl_trap(const char* code, const char* msg) {
    fflush(stdout);
    fprintf(stderr, "trap: %s: line %d: %s\n", code, cpl_at, msg);
    exit(3);
}
"#;

/// The rest of the runtime, after the trap table `ops::cpp_trap_table` writes.
const RUNTIME: &str = r#"[[noreturn]] static void cpl_fail(int f) { cpl_trap(cpl_faults[f][0], cpl_faults[f][1]); }
static char cpl_altstack[1 << 16];
static void cpl_segv(int) {
    // Only `write` is safe here, so the line is printed by hand.
    const char* code = cpl_faults[CPL_STACK_EXHAUSTED][0];
    const char* msg = cpl_faults[CPL_STACK_EXHAUSTED][1];
    char m[160] = "trap: ";
    int k = (int)strlen(m);
    memcpy(m + k, code, strlen(code)); k += (int)strlen(code);
    memcpy(m + k, ": line ", 7); k += 7;
    char d[12]; int n = 0;
    for (int v = cpl_at; v || !n; v /= 10) d[n++] = (char)('0' + v % 10);
    while (n) m[k++] = d[--n];
    m[k++] = ':'; m[k++] = ' ';
    memcpy(m + k, msg, strlen(msg)); k += (int)strlen(msg);
    m[k++] = '\n';
    if (write(2, m, k)) {}
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
"#;

/// The runtime's storage and input, after the rows `ops::cpp_runtime` writes.
fn runtime() -> String {
    let (table, rows, rand) = (ops::cpp_trap_table(), ops::cpp_runtime(), ops::cpp_rand());
    format!("{RUNTIME_HEAD}{table}{rand}{RUNTIME}{rows}{RUNTIME_STORAGE}")
}

const RUNTIME_STORAGE: &str = r#"template <class T> static inline cpl_raw<T> cpl_raw_new(int64_t n, const T& fill) {
    return cpl_raw<T>(cpl_storage_size(n), fill);
}
static inline cpl_raw<cpl_char> cpl_raw_text(const char* s, size_t n) {
    cpl_raw<cpl_char> r(n);
    for (size_t i = 0; i < n; i++) r[i].code = (uint8_t)s[i];
    return r;
}
static inline int64_t cpl_f64_len(double x, int64_t d) { return snprintf(nullptr, 0, "%.*f", (int)d, x); }
static inline cpl_raw<cpl_char> cpl_f64_text(double x, int64_t d) {
    char s[512];
    int n = snprintf(s, sizeof s, "%.*f", (int)d, x);
    return cpl_raw_text(s, (size_t)n);
}
template <class T> static inline T cpl_get(const cpl_raw<T>& r, int64_t i) {
    return r[cpl_read_cell(i, (int64_t)r.size())];
}
template <class T> static inline T cpl_take(cpl_raw<T>& r, int64_t i) {
    return std::move(r[cpl_read_cell(i, (int64_t)r.size())]);
}
template <class R> static inline R cpl_raw_cap(int64_t n) {
    return R(cpl_storage_size(n));
}
template <class T> static inline cpl_unit cpl_set(cpl_raw<T>& r, int64_t i, const T& x) {
    r[cpl_write_cell(i, (int64_t)r.size())] = x;
    return cpl_unit{};
}
static inline bool cpl_is_space(int c) { return c == ' ' || (c >= '\t' && c <= '\r'); }
// The input, read in one piece; `ops::Input` keeps the same rules.
static std::vector<unsigned char> cpl_in;
static size_t cpl_in_at = 0;
static bool cpl_in_loaded = false;
static inline void cpl_in_load() {
    if (cpl_in_loaded) return;
    cpl_in_loaded = true;
    unsigned char buf[1 << 16];
    size_t n;
    while ((n = fread(buf, 1, sizeof buf, stdin)) > 0) cpl_in.insert(cpl_in.end(), buf, buf + n);
}
static inline bool cpl_input_left() {
    cpl_in_load();
    for (size_t i = cpl_in_at; i < cpl_in.size(); i++)
        if (!cpl_is_space(cpl_in[i])) return true;
    return false;
}
static inline int64_t cpl_peek_byte(int64_t k) {
    cpl_in_load();
    return k >= 0 && (size_t)k < cpl_in.size() - cpl_in_at ? (int64_t)cpl_in[cpl_in_at + (size_t)k] : -1;
}
static inline int64_t cpl_read_byte() {
    cpl_in_load();
    return cpl_in_at < cpl_in.size() ? (int64_t)cpl_in[cpl_in_at++] : -1;
}
// The `Int` token `t` spells, as `ops::read_int` reads it.
static inline int64_t cpl_int_token(const std::string& t) {
    size_t i = !t.empty() && (t[0] == '-' || t[0] == '+');
    if (i == t.size()) cpl_fail(CPL_BAD_INT_TOKEN);
    uint64_t v = 0;
    bool past = false;
    for (; i < t.size(); i++) {
        if (t[i] < '0' || t[i] > '9') cpl_fail(CPL_BAD_INT_TOKEN);
        uint64_t d = (uint64_t)(t[i] - '0');
        if (past) continue;
        if (v > (INT_LAST - d) / 10) past = true;
        else v = v * 10 + d;
    }
    if (past && t[0] == '+') cpl_fail(CPL_BAD_INT_TOKEN);
    if (past) cpl_trap(cpl_faults[CPL_INT_PAST_INT][0], ("`" + t + "` does not fit in an `Int`").c_str());
    return t[0] == '-' ? -(int64_t)v : (int64_t)v;
}
static inline int64_t cpl_read() {
    cpl_in_load();
    size_t n = cpl_in.size(), s = cpl_in_at;
    while (s < n && cpl_is_space(cpl_in[s])) s++;
    size_t e = s;
    while (e < n && !cpl_is_space(cpl_in[e])) e++;
    size_t k = e;
    while (k < n && (cpl_in[k] == ' ' || cpl_in[k] == '\t' || cpl_in[k] == '\r')) k++;
    cpl_in_at = k == n ? n : cpl_in[k] == '\n' ? k + 1 : e;
    if (s == e) cpl_fail(CPL_BAD_INT_TOKEN);
    return cpl_int_token(std::string((const char*)&cpl_in[s], e - s));
}
// The caller's code handed to a generator or lender that calls itself: two words, not owning.
// Used only there; elsewhere the callee is a template and the code is inlined.
template <class T> struct cpl_body {
    void* env;
    int (*fn)(void*, T);
    template <class L> cpl_body(L& l) : env(&l), fn([](void* e, T x) -> int { return (*static_cast<L*>(e))(static_cast<T>(x)); }) {}
    int operator()(T x) const { return fn(env, static_cast<T>(x)); }
};
// A scalar as `ops::show` writes it in an event.
static inline void cpl_ev(cpl_char c) { fprintf(stderr, " %c", (char)c.code); }
static inline void cpl_ev(bool b) { fputs(b ? " true" : " false", stderr); }
static inline void cpl_ev(int64_t x) { fprintf(stderr, " %lld", (long long)x); }
static inline void cpl_ev(uint64_t x) { fprintf(stderr, " %llu", (unsigned long long)x); }
static inline cpl_unit cpl_out(const cpl_raw<cpl_char>& r, int64_t n) {
    for (int64_t i = 0; i < n; i++) putchar((char)r[(size_t)i].code);
    return cpl_unit{};
}
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
    /// The lines of the fn being printed (`Func::lines`).
    lines: std::cell::Cell<&'a [u32]>,
}

pub fn emit(p: &Program, funcs: &[Func<CTy>]) -> String {
    let mut e = Emit {
        p,
        funcs,
        recursive: recursive_regions(funcs),
        names: HashMap::new(),
        order: Vec::new(),
        lines: std::cell::Cell::new(&[]),
    };
    for f in funcs {
        for t in &f.locals {
            e.name(t);
        }
        e.name(&f.ret);
        if let Some((_, t)) = &f.region {
            e.name(t);
        }
        e.name_built(&f.body);
    }
    for t in &p.globals {
        e.name(t);
    }
    e.order = e.by_value_order();
    let mut s = runtime();
    for t in &e.order {
        put_line!(s, "struct {};", e.names[t]);
    }
    for t in e.order.clone() {
        let n = e.names[&t].clone();
        match &t {
            CTy::Struct(..) if p.sums.contains_key(&t) => {
                let mut body = String::from("int64_t tag;");
                for (i, k) in p.sums[&t].iter().enumerate() {
                    put!(body, " {} v{i};", e.names[k]);
                }
                for (i, k) in p.sums[&t].iter().enumerate() {
                    put!(body, " static {n} m{i}(const {}& x) {{ {n} u{{}}; u.tag = {i}; u.v{i} = x; return u; }}", e.names[k]);
                }
                put_line!(s, "struct {n} {{ {body} }};");
            }
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
        e.lines.set(&f.lines);
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
            Place::Field(b, _)
            | Place::Variant(b, _)
            | Place::Deref(b)
            | Place::Through(b)
            | Place::Slot(b, _) => root(b),
        }
    }
    fn expr(e: &Expr<CTy>, w: &mut Vec<usize>) {
        match &e.kind {
            EK::Intr(_, args, _) | EK::Call(_, args, _) => {
                for a in args {
                    match a {
                        Arg::Lend(p) => w.extend(root(p)),
                        Arg::View(_) => {}
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
                Stmt::If(_, a, b2) | Stmt::Loop(a, b2, _) => {
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
                && !matches!(
                    f.locals[*l],
                    CTy::Int
                        | CTy::Bool
                        | CTy::Char
                        | CTy::U64
                        | CTy::F64
                        | CTy::Unit
                        | CTy::Dom(_)
                )
        })
        .collect()
}

impl Emit<'_> {
    fn line(&self, site: Site) -> u32 {
        self.lines.get()[site as usize]
    }
    /// Definition order for C++: a type comes after every type it holds by value. Storage
    /// (`cpl_raw<T>`) and references need only T's declaration, which is what lets a
    /// function value reach itself through a list.
    fn by_value_order(&self) -> Vec<CTy> {
        fn visit(e: &Emit, t: &CTy, done: &mut HashSet<CTy>, out: &mut Vec<CTy>) {
            let parts: Vec<CTy> = match t {
                CTy::Struct(..) => e.p.parts(t),
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
    /// Names the type of every value built in place, which no local may hold.
    fn name_built(&mut self, ss: &[Stmt<CTy>]) {
        fn expr(e: &mut Emit, x: &Expr<CTy>) {
            match &x.kind {
                EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                    e.name(&x.ty);
                    xs.iter().for_each(|y| expr(e, y));
                }
                EK::MakeUnion(_, y) => {
                    e.name(&x.ty);
                    expr(e, y);
                }
                EK::Inject(y) | EK::Widen(y) => expr(e, y),
                EK::Intr(_, args, _) | EK::Call(_, args, _) => args.iter().for_each(|a| {
                    if let Arg::Value(y) = a {
                        expr(e, y)
                    }
                }),
                EK::CallValue(c, xs, _) => {
                    expr(e, c);
                    xs.iter().for_each(|y| expr(e, y));
                }
                _ => {}
            }
        }
        for st in ss {
            match st {
                Stmt::Let(_, x) | Stmt::Assign(_, x) | Stmt::Expr(x) | Stmt::Yield(x) => {
                    expr(self, x)
                }
                Stmt::Return(x) => x.iter().for_each(|x| expr(self, x)),
                // Optimized IR, which this prints, has none.
                Stmt::Fact(_) => {}
                Stmt::Check { cond, .. } => expr(self, cond),
                Stmt::If(c, a, b) => {
                    expr(self, c);
                    self.name_built(a);
                    self.name_built(b);
                }
                Stmt::Loop(a, b, _) => {
                    self.name_built(a);
                    self.name_built(b);
                }
                Stmt::Switch(c, arms) => {
                    expr(self, c);
                    arms.iter().for_each(|a| self.name_built(a));
                }
                Stmt::Region { args, body, .. } => {
                    args.iter().for_each(|a| {
                        if let Arg::Value(y) = a {
                            expr(self, y)
                        }
                    });
                    self.name_built(body);
                }
                Stmt::Event(_, xs) => xs.iter().for_each(|x| expr(self, x)),
                Stmt::Break | Stmt::Continue | Stmt::LendOut(_) => {}
            }
        }
    }
    fn name(&mut self, t: &CTy) -> String {
        if let Some(n) = self.names.get(t) {
            return n.clone();
        }
        let n = match t {
            CTy::Int | CTy::Dom(_) | CTy::Error => "int64_t".to_string(),
            CTy::Bool => "bool".to_string(),
            CTy::Char => "cpl_char".to_string(),
            CTy::U64 => "uint64_t".to_string(),
            CTy::F64 => "double".to_string(),
            CTy::Unit => "cpl_unit".to_string(),
            CTy::Raw(e) => format!("cpl_raw<{}>", self.name(e)),
            CTy::Ref(e) => format!("{}*", self.name(e)),
            CTy::Lent(e) => format!("{} const*", self.name(e)),
            CTy::Struct(..) => {
                let n = format!("S{}", self.names.len());
                self.names.insert(t.clone(), n.clone());
                for f in self.p.parts(t) {
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
        cpl_next_ir::prog::plain(&|s| self.p.parts(s), t, depth)
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
            Place::Deref(b) | Place::Through(b) => format!("(*{})", self.place(b)),
            Place::Slot(b, l) => format!("{}[l{l}]", self.place(b)),
        }
    }
    fn arg(&self, a: &Arg<CTy>) -> String {
        match a {
            Arg::Value(v) => self.expr(v),
            Arg::View(p) | Arg::Lend(p) => self.place(p),
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
            // The shortest decimal that reads back as these bits.
            EK::F64(b) => format!("{:?}", f64::from_bits(*b)),
            EK::Bool(b) => b.to_string(),
            EK::Text(t) => {
                let lit: String = t
                    .bytes()
                    .map(|b| format!("\\x{b:02x}"))
                    .collect::<Vec<_>>()
                    .join("\"\"");
                format!("cpl_raw_text(\"{lit}\", {})", t.len())
            }
            EK::Unit => "cpl_unit{}".to_string(),
            EK::DomLen(_) | EK::DomLo(_) => "/* unresolved domain value */".to_string(),
            EK::HasBound(..) => "/* undecided bound test */".to_string(),
            EK::Copy(p) => self.place(p),
            EK::Lent(p) => format!("(&{})", self.place(p)),
            EK::AddrOf(p) => format!("(&{})", self.place(p)),
            EK::Tag(p) => format!("{}.tag", self.place(p)),
            EK::MakeStruct(xs) if self.p.structs.contains_key(&e.ty) => {
                // A lent slot filled from a lent place gets the pointer, not what it points at.
                let fs = &self.p.structs[&e.ty];
                let xs: Vec<String> = (xs.iter().zip(fs))
                    .map(|(x, f)| match (&x.kind, f) {
                        (EK::Copy(Place::Deref(q) | Place::Through(q)), CTy::Lent(_)) => {
                            self.place(q)
                        }
                        _ => self.expr(x),
                    })
                    .collect();
                format!("{}{{{}}}", self.names[&e.ty], xs.join(", "))
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => {
                format!("{}{{{}}}", self.names[&e.ty], list(xs))
            }
            EK::MakeUnion(i, x) => format!("{}::m{i}({})", self.names[&e.ty], self.expr(x)),
            EK::Inject(x) | EK::Widen(x) => self.expr(x),
            EK::Call(Target::Inst(f), args, site) if let Some(op) = self.funcs[*f].fast => self
                .expr(&Expr {
                    kind: EK::Intr(op, args.clone(), *site),
                    ty: e.ty.clone(),
                    site: e.site,
                }),
            EK::Call(Target::Inst(f), args, _) => format!(
                "f{f}({})",
                args.iter()
                    .map(|a| self.arg(a))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            EK::Call(..) | EK::CallValue(..) | EK::Pass(..) => "/* not canonical */".to_string(),
            EK::Intr(op, args, _) => {
                let a: Vec<String> = args.iter().map(|a| self.arg(a)).collect();
                let bin = |o: &str| format!("({} {o} {})", a[0], a[1]);
                let unsigned = matches!(args.first(), Some(Arg::Value(e)) if e.ty == CTy::U64);
                // `F64` arithmetic is IEEE in both engines (R02.1).
                if matches!(args.first(), Some(Arg::Value(e)) if e.ty == CTy::F64) {
                    match op {
                        Intr::Add => return bin("+"),
                        Intr::Sub => return bin("-"),
                        Intr::Mul => return bin("*"),
                        Intr::Div => return bin("/"),
                        Intr::Neg => return format!("(-{})", a[0]),
                        Intr::Abs => return format!("__builtin_fabs({})", a[0]),
                        _ => {}
                    }
                }
                match ops::cpp(*op, unsigned) {
                    Some(ops::Cpp::Infix(o)) => return bin(o),
                    Some(ops::Cpp::Call(f)) => return format!("{f}({})", a.join(", ")),
                    None => {}
                }
                match op {
                    Intr::ToU64 => format!("((uint64_t)({}))", a[0]),
                    Intr::ToF64 => format!("((double)({}))", a[0]),
                    Intr::F64Len => format!("cpl_f64_len({}, {})", a[0], a[1]),
                    Intr::F64Text => format!("cpl_f64_text({}, {})", a[0], a[1]),
                    Intr::FromU64 => format!("cpl_int_of_u64({})", a[0]),
                    Intr::Popcount => format!("cpl_popcount((uint64_t)({}))", a[0]),
                    Intr::Ctz => format!("cpl_ctz((uint64_t)({}))", a[0]),
                    Intr::Clz => format!("cpl_clz((uint64_t)({}))", a[0]),
                    Intr::BitWidth => format!("cpl_bit_width((uint64_t)({}))", a[0]),
                    Intr::BitCeil => format!("cpl_bit_ceil((int64_t)({}))", a[0]),
                    Intr::Neg => format!("cpl_neg({})", a[0]),
                    Intr::Abs => format!("cpl_abs({})", a[0]),
                    Intr::Below => format!("((uint64_t)({}) < (uint64_t)({}))", a[0], a[1]),
                    Intr::Not => format!("(!{})", a[0]),
                    Intr::CharOf => format!("cpl_char_of({})", a[0]),
                    Intr::CodeOf => format!("((int64_t)({}).code)", a[0]),
                    Intr::RawEmpty => format!("{}{{}}", self.names[&e.ty]),
                    Intr::RawNew => format!("cpl_raw_new({}, {})", a[0], a[1]),
                    Intr::RawGet => format!("cpl_get({}, {})", a[0], a[1]),
                    Intr::RawSet => format!("cpl_set({}, {}, {})", a[0], a[1], a[2]),
                    Intr::RawTake => format!("cpl_take({}, {})", a[0], a[1]),
                    Intr::RawCap => format!("cpl_raw_cap<{}>({})", self.names[&e.ty], a[0]),
                    Intr::CheckKey => format!("cpl_check_key({})", a[0]),
                    Intr::CheckVariant => format!("cpl_check_variant({})", a[0]),
                    Intr::CheckRange => format!("cpl_check_range({})", a[0]),
                    Intr::ReadInt => "cpl_read()".to_string(),
                    Intr::InputLeft => "cpl_input_left()".to_string(),
                    Intr::ReadByte => "cpl_read_byte()".to_string(),
                    Intr::PeekByte => format!("cpl_peek_byte({})", a[0]),
                    Intr::CheckToken => format!("cpl_check_token({})", a[0]),
                    Intr::CheckInput => format!("cpl_check_input({})", a[0]),
                    Intr::RandWord => "cpl_rand_word()".to_string(),
                    Intr::Out => format!("cpl_out({}, {})", a[0], a[1]),
                    Intr::Unset => format!("{}{{}}", self.names[&e.ty]),
                    // `ops::cpp` spells these, for operands of every kind.
                    Intr::Add
                    | Intr::Sub
                    | Intr::Mul
                    | Intr::Div
                    | Intr::Rem
                    | Intr::Pow
                    | Intr::Shl
                    | Intr::Shr
                    | Intr::AddWrap
                    | Intr::SubWrap
                    | Intr::MulWrap
                    | Intr::PowWrap
                    | Intr::ShlWrap
                    | Intr::BitAnd
                    | Intr::BitOr
                    | Intr::BitXor
                    | Intr::Eq
                    | Intr::Ne
                    | Intr::Lt
                    | Intr::Le
                    | Intr::Gt
                    | Intr::Ge => "/* not an operation */".to_string(),
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
                    EK::Intr(op, _, site) if op.checked() && self.line(*site) != 0 => {
                        put_line!(s, "{pad}cpl_at = {};", self.line(*site));
                        None
                    }
                    EK::Call(_, _, site) => Some(self.line(*site)),
                    _ => None,
                },
                Stmt::Region { site, .. } => Some(self.line(*site)),
                Stmt::Yield(_) | Stmt::LendOut(_) => Some(0),
                _ => None,
            };
            match around {
                Some(0) => put_line!(s, "{pad}{{ int cpl_was = cpl_at;"),
                Some(line) => put_line!(s, "{pad}{{ const int cpl_was = cpl_at = {line};"),
                None => {}
            }
            match st {
                Stmt::Let(l, e) => put_line!(s, "{pad}l{l} = {};", self.expr(e)),
                Stmt::Assign(p, e) => put_line!(s, "{pad}{} = {};", self.place(p), self.expr(e)),
                Stmt::Expr(e) => put_line!(s, "{pad}(void)({});", self.expr(e)),
                Stmt::Fact(_) => {}
                Stmt::If(c, a, b2) => {
                    put_line!(s, "{pad}if ({}) {{", self.expr(c));
                    self.block(a, depth + 1, s, label, ctx);
                    if !b2.is_empty() {
                        put_line!(s, "{pad}}} else {{");
                        self.block(b2, depth + 1, s, label, ctx);
                    }
                    put_line!(s, "{pad}}}");
                }
                Stmt::Loop(body, step, _) => {
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
                    if self.line(*site) != 0 {
                        format!("cpl_at = {}; ", self.line(*site))
                    } else {
                        String::new()
                    },
                    kind.code().0
                ),
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
                Stmt::If(_, a, b2) | Stmt::Loop(a, b2, _) => {
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

#[cfg(test)]
#[path = "../../cpl-cli/tests/support/cxx.rs"]
mod cxx;

/// The C++ runtime's helpers against `ops`, row by row, on the edges of each operand's range.
#[cfg(test)]
mod tests {
    use super::*;
    use cpl_next_ir::ops::Fault;
    use std::fmt::Display;

    fn expect<T: Display>(r: Result<T, Fault>) -> String {
        match r {
            Ok(v) => format!("{v}\n"),
            Err(f) => format!("trap: {}: line 0: {}\n", f.code(), f.message()),
        }
    }

    #[test]
    fn runtime_agrees_with_ops() {
        let ints = [
            -i64::MAX,
            -64,
            -3,
            -2,
            -1,
            0,
            1,
            2,
            3,
            62,
            63,
            64,
            1 << 32,
            i64::MAX,
        ];
        let words = [0, 1, 2, 3, 63, 64, 65, 1 << 32, 1 << 63, u64::MAX];
        let ops = [
            Intr::Add,
            Intr::Sub,
            Intr::Mul,
            Intr::Div,
            Intr::Rem,
            Intr::Pow,
            Intr::Shl,
            Intr::Shr,
            Intr::BitAnd,
            Intr::BitOr,
            Intr::BitXor,
        ];
        let (mut cases, mut want) = (Vec::new(), String::new());
        let mut case = |c: String, w: String| {
            cases.push(c);
            want.push_str(&w);
        };
        let call = |op: Intr, unsigned: bool, a: &str, b: &str| match ops::cpp(op, unsigned) {
            Some(ops::Cpp::Call(f)) => format!("{f}({a}, {b})"),
            Some(ops::Cpp::Infix(o)) => format!("({a} {o} {b})"),
            None => panic!("{op:?} has no C++"),
        };
        for a in ints {
            case(
                format!("printf(\"%lld\\n\", (long long)cpl_neg(INT64_C({a})));"),
                expect(ops::neg(a)),
            );
            case(
                format!("printf(\"%zu\\n\", cpl_raw_new(INT64_C({a}), INT64_C(0)).size());"),
                expect(ops::storage_size(a)),
            );
            case(
                format!("printf(\"%d\\n\", (int)cpl_get(cpl_raw<int64_t>(3, 7), INT64_C({a})));"),
                expect(ops::cell(a, 3, false).map(|_| 7)),
            );
            case(
                format!("cpl_raw<int64_t> r(3); cpl_set(r, INT64_C({a}), INT64_C(7)); printf(\"7\\n\");"),
                expect(ops::cell(a, 3, true).map(|_| 7)),
            );
        }
        let wrapping = [
            Intr::AddWrap,
            Intr::SubWrap,
            Intr::MulWrap,
            Intr::PowWrap,
            Intr::ShlWrap,
        ];
        for op in wrapping {
            for a in ints {
                for b in ints {
                    let c = call(
                        op,
                        false,
                        &format!("INT64_C({a})"),
                        &format!("INT64_C({b})"),
                    );
                    case(
                        format!("printf(\"%lld\\n\", (long long){c});"),
                        expect(ops::int(op, a, b)),
                    );
                }
            }
        }
        for op in ops {
            for a in ints {
                for b in ints {
                    let c = call(
                        op,
                        false,
                        &format!("INT64_C({a})"),
                        &format!("INT64_C({b})"),
                    );
                    case(
                        format!("printf(\"%lld\\n\", (long long){c});"),
                        expect(ops::int(op, a, b)),
                    );
                }
            }
            let shift = matches!(op, Intr::Shl | Intr::Shr);
            for a in words {
                // A shift's amount is an `Int`, given to `ops::word` as its bits.
                let amounts: Vec<u64> = if shift {
                    ints.iter().map(|b| *b as u64).collect()
                } else {
                    words.to_vec()
                };
                for b in amounts {
                    let bs = if shift {
                        format!("INT64_C({})", b as i64)
                    } else {
                        format!("UINT64_C({b})")
                    };
                    let c = call(op, true, &format!("UINT64_C({a})"), &bs);
                    case(
                        format!("printf(\"%llu\\n\", (unsigned long long){c});"),
                        expect(ops::word(op, a, b)),
                    );
                }
            }
        }
        for a in words {
            case(
                format!("printf(\"%lld\\n\", (long long)cpl_int_of_u64(UINT64_C({a})));"),
                expect(ops::int_of_u64(a)),
            );
            for (op, f) in [
                (Intr::Popcount, "cpl_popcount"),
                (Intr::Ctz, "cpl_ctz"),
                (Intr::Clz, "cpl_clz"),
                (Intr::BitWidth, "cpl_bit_width"),
                (Intr::BitCeil, "cpl_bit_ceil"),
            ] {
                case(
                    format!("printf(\"%lld\\n\", (long long){f}(UINT64_C({a})));"),
                    expect(ops::bit_count(op, a)),
                );
            }
        }
        for (op, f) in [
            (Intr::CheckKey, "cpl_check_key"),
            (Intr::CheckRange, "cpl_check_range"),
            (Intr::CheckVariant, "cpl_check_variant"),
            (Intr::CheckToken, "cpl_check_token"),
            (Intr::CheckInput, "cpl_check_input"),
        ] {
            for ok in [true, false] {
                case(
                    format!("{f}({ok}); printf(\"0\\n\");"),
                    expect(ops::check(op, ok).map(|_| 0)),
                );
            }
        }
        let long = "9".repeat(40);
        for t in [
            "0",
            "-0",
            "+5",
            "007",
            "-",
            "+",
            "1-",
            "--1",
            "+-1",
            "12a",
            "9223372036854775807",
            "9223372036854775808",
            "-9223372036854775807",
            "-9223372036854775808",
            "+9223372036854775808",
            &long,
        ] {
            let want = match ops::read_int(Some(t)) {
                Ok(v) => format!("{v}\n"),
                Err(f) => {
                    let msg = cpl_next_ir::run::Trap::at_token(f, Some(t)).msg;
                    format!("trap: {}: line 0: {msg}\n", f.code())
                }
            };
            case(
                format!("printf(\"%lld\\n\", (long long)cpl_int_token(std::string({t:?})));"),
                want,
            );
        }
        for c in 0..=u8::MAX {
            case(
                format!("printf(\"%d\\n\", (int)cpl_is_space({c}));"),
                format!("{}\n", i32::from(ops::is_space(c))),
            );
        }
        let floats = [
            0.0,
            -0.0,
            1.5,
            -2.25,
            1e300,
            -1e-300,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
        ];
        for x in floats {
            let b = x.to_bits();
            for d in [0, 3, 10] {
                case(
                    format!("double x; uint64_t b = UINT64_C({b}); memcpy(&x, &b, 8); auto t = cpl_f64_text(x, {d}); printf(\"[\"); cpl_out(t, (int64_t)t.size()); printf(\" %lld]\\n\", (long long)cpl_f64_len(x, {d}));"),
                    {
                        let t = ops::f64_text(x, d);
                        format!("[{t} {}]\n", t.len())
                    },
                );
            }
        }
        let mut shown = |v: ops::Scalar, cpp: &str| {
            let mut out = String::new();
            ops::show(v, &mut out);
            case(
                format!("printf(\"[\"); fflush(stdout); cpl_ev({cpp}); fflush(stderr); printf(\"]\\n\");"),
                format!("[ {out}]\n"),
            );
        };
        for b in [false, true] {
            shown(ops::Scalar::Bool(b), &b.to_string());
        }
        for c in *b"aZ0~!" {
            shown(ops::Scalar::Char(c), &format!("cpl_char{{{c}}}"));
        }
        for a in [-i64::MAX, -1, 0, 7, i64::MAX] {
            shown(ops::Scalar::Int(a), &format!("INT64_C({a})"));
        }
        for a in [0, 1 << 63, u64::MAX] {
            shown(ops::Scalar::U64(a), &format!("UINT64_C({a})"));
        }
        let mut cpp = runtime();
        cpp.push_str("#include <sys/wait.h>\nint main() {\n");
        for c in &cases {
            put_line!(
                cpp,
                "    fflush(stdout); if (fork() == 0) {{ dup2(1, 2); {{ {c} }} fflush(stdout); _exit(0); }} wait(nullptr);"
            );
        }
        cpp.push_str("}\n");
        let flags = ["-std=c++17", "-O1", "-w", "-x", "c++"];
        let (bin, _) = cxx::build(cpp.as_bytes(), &flags).unwrap();
        let out = std::process::Command::new(bin).output().unwrap();
        let got = String::from_utf8_lossy(&out.stdout);
        for ((c, g), w) in cases
            .iter()
            .zip(got.split_inclusive('\n'))
            .zip(want.split_inclusive('\n'))
        {
            assert_eq!(g, w, "{c}");
        }
        assert_eq!(got.lines().count(), cases.len());
    }
}
