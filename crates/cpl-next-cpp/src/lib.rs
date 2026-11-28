//! Contract 7: the C++ emitter. A printer over canonical IR: one C++ shape per
//! node kind, and no decision of its own.
use cpl_next_ir::ir::*;
use cpl_next_ir::layout::{Layouts, Shape};
use cpl_next_ir::ops;
use cpl_next_ir::prog::{CTy, Program};
use cpl_next_ir::run::Limits;
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
#include <new>
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
static char cpl_tb[1 << 16]; static size_t cpl_tn = 0;
static void cpl_tflush() { if (cpl_tn && write(2, cpl_tb, cpl_tn)) {} cpl_tn = 0; }
static void cpl_tput(const char* p, size_t n) { if (cpl_tn + n > sizeof cpl_tb) cpl_tflush(); if (n > sizeof cpl_tb) { if (write(2, p, n)) {} } else { memcpy(cpl_tb + cpl_tn, p, n); cpl_tn += n; } }
[[noreturn]] static void cpl_trap(const char* code, const char* msg) {
    fflush(stdout); cpl_tflush();
    fprintf(stderr, "trap: %s: line %d: %s\n", code, cpl_at, msg);
    exit(3);
}
"#;

/// The rest of the runtime, after the trap table `ops::cpp_trap_table` writes.
const RUNTIME: &str = r#"[[noreturn]] static void cpl_fail(int f) { cpl_trap(cpl_faults[f][0], cpl_faults[f][1]); }
static char cpl_altstack[1 << 16];
static void cpl_segv(int) {
    // Only `write` is safe here: the line goes after the trace through its buffer (`cpl_tput`), by hand.
    const char* code = cpl_faults[CPL_STACK_EXHAUSTED][0];
    const char* msg = cpl_faults[CPL_STACK_EXHAUSTED][1];
    cpl_tput("trap: ", 6); cpl_tput(code, strlen(code)); cpl_tput(": line ", 7);
    char d[12]; int n = 0;
    for (int v = cpl_at; v || !n; v /= 10) d[n++] = (char)('0' + v % 10);
    while (n) cpl_tput(&d[--n], 1);
    cpl_tput(": ", 2); cpl_tput(msg, strlen(msg)); cpl_tput("\n", 1);
    cpl_tflush();
    _exit(3);
}
#ifdef __SANITIZE_ADDRESS__
extern "C" const char* __asan_default_options() { return "handle_segv=0:handle_sigbus=0"; }
#endif
static void cpl_guard_stack() {
    stack_t ss; ss.ss_sp = cpl_altstack; ss.ss_size = sizeof cpl_altstack; ss.ss_flags = 0;
    sigaltstack(&ss, nullptr);
    struct sigaction sa; memset(&sa, 0, sizeof sa); sa.sa_handler = cpl_segv; sa.sa_flags = SA_ONSTACK;
    sigaction(SIGSEGV, &sa, nullptr); sigaction(SIGBUS, &sa, nullptr);
}
"#;

/// The runtime's storage and input, after the rows `ops::cpp_runtime` writes.
fn runtime(limits: &Limits) -> String {
    let (table, rows, rand) = (ops::cpp_trap_table(), ops::cpp_runtime(), ops::cpp_rand());
    let memory = format!("static const int64_t CPL_MEMORY = {}LL;\n", limits.memory);
    format!("{RUNTIME_HEAD}{memory}{table}{rand}{RUNTIME}{rows}{RUNTIME_STORAGE}")
}

const RUNTIME_STORAGE: &str = r#"template <class T> static inline cpl_raw<T> cpl_raw_new(int64_t n, const T& fill, int64_t cell) {
    return cpl_raw<T>(cpl_storage_size(n, cell, CPL_MEMORY), fill);
}
static inline cpl_raw<cpl_char> cpl_raw_text(const char* s, size_t n) {
    cpl_raw<cpl_char> r(n);
    for (size_t i = 0; i < n; i++) r[i].code = (uint8_t)s[i];
    return r;
}
static inline uint64_t cpl_f64_bits(double x) { uint64_t b; memcpy(&b, &x, 8); return b; }
template <class T> static inline T cpl_get(const cpl_raw<T>& r, int64_t i) {
    return r[cpl_read_cell(i, (int64_t)r.size())];
}
template <class T> static inline T cpl_take(cpl_raw<T>& r, int64_t i) {
    return std::move(r[cpl_read_cell(i, (int64_t)r.size())]);
}
template <class R> static inline R cpl_raw_cap(int64_t n, int64_t cell) {
    return R(cpl_storage_size(n, cell, CPL_MEMORY));
}
template <class T> static inline cpl_unit cpl_set(cpl_raw<T>& r, int64_t i, const T& x) {
    r[cpl_write_cell(i, (int64_t)r.size())] = x;
    return cpl_unit{};
}
static inline bool cpl_is_space(int c) { return c == ' ' || (c >= '\t' && c <= '\r'); }
// The input so far, read as the program asks for it (`ops::Input` keeps the same rules).
static std::string cpl_in;
static size_t cpl_in_at = 0; static bool cpl_in_ended = false;
// Whether byte `j` past the read position is in; waiting for it hands over the output (R10.4).
static __attribute__((noinline)) bool cpl_fill(size_t j) {
    static char b[1 << 16];
    for (ssize_t n; !cpl_in_ended && cpl_in_at + j >= cpl_in.size();) {
        fflush(stdout);
        cpl_in.erase(0, cpl_in_at); cpl_in_at = 0;
        do n = read(0, b, sizeof b); while (n < 0 && errno == EINTR);
        if (n > 0) cpl_in.append(b, (size_t)n); else cpl_in_ended = true;
    }
    return cpl_in_at + j < cpl_in.size();
}
static inline bool cpl_has(size_t j) { return __builtin_expect(cpl_in_at + j < cpl_in.size(), 1) || cpl_fill(j); }
static inline int64_t cpl_peek_byte(int64_t k) { return k >= 0 && cpl_has((size_t)k) ? (int64_t)(unsigned char)cpl_in[cpl_in_at + (size_t)k] : -1; }
static inline bool cpl_input_left() { int64_t k = 0; while (cpl_is_space((int)cpl_peek_byte(k))) k++; return cpl_peek_byte(k) >= 0; }
static inline int64_t cpl_read_byte() { return cpl_has(0) ? (int64_t)(unsigned char)cpl_in[cpl_in_at++] : -1; }
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
    size_t n, s, e, k;
    do {
        n = cpl_in.size();
        for (s = cpl_in_at; s < n && cpl_is_space(cpl_in[s]); s++) {}
        for (e = s; e < n && !cpl_is_space(cpl_in[e]); e++) {}
        for (k = e; k < n && (cpl_in[k] == ' ' || cpl_in[k] == '\t' || cpl_in[k] == '\r'); k++) {}
    } while (k == n && !cpl_in_ended && (cpl_fill(n - cpl_in_at), true));
    cpl_in_at = k == n ? n : cpl_in[k] == '\n' ? k + 1 : e;
    if (s == e) cpl_fail(CPL_BAD_INT_TOKEN);
    return cpl_int_token(cpl_in.substr(s, e - s));
}
// The caller's code handed to a generator or lender that calls itself: two words, not owning.
// Used only there; elsewhere the callee is a template and the code is inlined.
template <class T> struct cpl_body {
    void* env;
    int (*fn)(void*, T);
    template <class L> cpl_body(L& l) : env(&l), fn([](void* e, T x) -> int { return (*static_cast<L*>(e))(static_cast<T>(x)); }) {}
    int operator()(T x) const { return fn(env, static_cast<T>(x)); }
};
// While an event shows a value, what is written goes to the event instead of stdout.
static std::string* cpl_capture = nullptr;
static inline cpl_unit cpl_out(const cpl_raw<cpl_char>& r, int64_t n) { size_t k = (size_t)(n < 0 ? 0 : n < (int64_t)r.size() ? n : (int64_t)r.size()); if (cpl_capture) cpl_capture->append((const char*)r.data(), k); else fwrite(r.data(), 1, k, stdout); return {}; }
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
/// What a jump leaves: the innermost loop or region body, or the fn.
#[derive(Clone, Copy)]
enum Leave {
    Break,
    Continue,
    Return,
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
    /// The IR's layout table: field order, members and where the tag is.
    lays: Layouts,
    /// The bytes a cell of each storage type takes (`Layouts::cell_bytes`).
    cells: HashMap<CTy, i64>,
    /// The origins of the fn being printed (`Func::origins`).
    origins: std::cell::Cell<&'a [u32]>,
    /// The line of the user's file each origin is on: origin `o`'s is `lines[o - 1]`.
    lines: &'a [u32],
}

/// A union's C++ body: the tag, then one member's payload in storage shared by all of them, as
/// the layout table lays it out. A union whose members can hold a handle (`owns`) copies,
/// moves and destroys the member its tag names.
fn tagged(n: &str, ms: &[String], owns: bool) -> String {
    if ms.is_empty() {
        return "int64_t tag;".to_string();
    }
    let each = |f: &dyn Fn(usize) -> String| {
        let cases: Vec<String> = (0..ms.len())
            .map(|i| format!("case {i}: {} break;", f(i)))
            .collect();
        format!("switch (tag) {{ {} }}", cases.join(" "))
    };
    let mut b = String::new();
    for (i, m) in ms.iter().enumerate() {
        put!(b, "using M{i} = {m}; ");
    }
    let vs: Vec<String> = (0..ms.len()).map(|i| format!("M{i} v{i};")).collect();
    put!(
        b,
        "int64_t tag; union {{ {} }}; {n}() : tag(0), v0() {{}}",
        vs.join(" ")
    );
    let clear = if owns { "u.clear(); " } else { "" };
    for i in 0..ms.len() {
        put!(b, " static {n} m{i}(const M{i}& x) {{ {n} u; {clear}u.tag = {i}; new (&u.v{i}) M{i}(x); return u; }}");
    }
    if owns {
        let copy = each(&|i| format!("new (&v{i}) M{i}(o.v{i});"));
        let take = each(&|i| format!("new (&v{i}) M{i}(static_cast<M{i}&&>(o.v{i}));"));
        put!(b, " {n}(const {n}& o) : tag(o.tag) {{ {copy} }}");
        put!(b, " {n}({n}&& o) noexcept : tag(o.tag) {{ {take} }}");
        put!(b, " {n}& operator=(const {n}& o) {{ if (this != &o) {{ clear(); tag = o.tag; {copy} }} return *this; }}");
        put!(b, " {n}& operator=({n}&& o) noexcept {{ if (this != &o) {{ clear(); tag = o.tag; {take} }} return *this; }}");
        put!(
            b,
            " ~{n}() {{ clear(); }} void clear() {{ {} }}",
            each(&|i| format!("v{i}.~M{i}();"))
        );
    }
    b
}

/// The program as C++. A trap prints the line of `lines` its origin is on.
pub fn emit(p: &Program, funcs: &[Func<CTy>], lines: &[u32], limits: &Limits) -> String {
    let mut e = Emit {
        p,
        funcs,
        recursive: recursive_regions(funcs),
        names: HashMap::new(),
        order: Vec::new(),
        lays: Layouts::default(),
        cells: HashMap::new(),
        origins: std::cell::Cell::new(&[]),
        lines,
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
    let raws: Vec<CTy> = (e.names.keys())
        .filter(|t| matches!(t, CTy::Raw(_)))
        .cloned()
        .collect();
    for t in raws {
        if let CTy::Raw(elem) = &t {
            let l = e.lays.of(p, elem);
            let bytes = e.lays.cell_bytes(l);
            e.cells.insert(t, bytes);
        }
    }
    let mut s = runtime(limits);
    for t in &e.order {
        put_line!(s, "struct {};", e.names[t]);
    }
    // Fields and members in the order, and the tag where, the IR's layout table puts them.
    let mut lays = std::mem::take(&mut e.lays);
    for t in e.order.clone() {
        if !matches!(t, CTy::Struct(..) | CTy::Clo(_) | CTy::Fn(..)) {
            continue;
        }
        let n = e.names[&t].clone();
        let l = lays.of(p, &t) as usize;
        let part = |m: u32| e.names[&lays.ty[m as usize]].clone();
        match &lays.shape[l] {
            Shape::Union(ms) => {
                let ms: Vec<String> = ms.iter().map(|m| part(*m)).collect();
                put_line!(
                    s,
                    "struct {n} {{ {} }};",
                    tagged(&n, &ms, lays.flat[l].is_none())
                );
            }
            Shape::Struct(fs) => {
                let fs: Vec<String> = (fs.iter().enumerate())
                    .map(|(i, (_, f))| format!("{} f{i};", part(*f)))
                    .collect();
                put_line!(s, "struct {n} {{ {} }};", fs.join(" "));
            }
            // A struct, closure or fn value laid out as a word or nothing has no parts.
            Shape::Zero | Shape::Word | Shape::Handle => put_line!(s, "struct {n} {{ }};"),
        }
    }
    e.lays = lays;
    for (i, t) in p.globals.iter().enumerate() {
        put_line!(s, "static {} g{i}{{}};", e.names[t]);
    }
    for (i, f) in funcs.iter().enumerate() {
        put_line!(s, "{};", e.sig(i, f));
    }
    for (i, f) in funcs.iter().enumerate() {
        put_line!(s, "// {}", f.name);
        put_line!(s, "{} {{", e.sig(i, f));
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
        e.origins.set(&f.origins);
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
    // Spec 9.6: `main` runs on a thread with the judge's stack, the run's `Limits::stack`.
    put_line!(s, "static void* cpl_main(void*) {{ cpl_guard_stack(); f{}(); fflush(stdout); cpl_tflush(); return nullptr; }}", p.main);
    put_line!(
        s,
        "int main() {{ pthread_attr_t a; pthread_t t; pthread_attr_init(&a); pthread_attr_setstacksize(&a, (size_t){}ULL);\n    if (pthread_create(&t, &a, cpl_main, nullptr)) {{ cpl_main(nullptr); return 0; }}\n    pthread_join(t, nullptr); return 0; }}",
        limits.stack
    );
    s
}

impl Emit<'_> {
    fn line(&self, site: Site) -> u32 {
        match self.origins.get()[site as usize] {
            0 => 0,
            o => self.lines[o as usize - 1],
        }
    }
    /// The line of the first operation in `e` that can trap, or 0 if none can.
    fn trap_line(&self, e: &Expr<CTy>) -> u32 {
        let first = |xs: &mut dyn Iterator<Item = &Expr<CTy>>| {
            xs.map(|x| self.trap_line(x)).find(|&l| l != 0)
        };
        let inner = match &e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => first(&mut xs.iter()),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => first(&mut std::iter::once(&**x)),
            EK::Intr(_, args, _) => first(&mut args.iter().filter_map(|a| match a {
                Arg::Value(x) => Some(x),
                _ => None,
            })),
            _ => None,
        };
        match &e.kind {
            _ if inner.is_some() => inner.unwrap_or(0),
            EK::Intr(op, _, site) if op.checked() => self.line(*site),
            _ => 0,
        }
    }
    /// Definition order for C++: a type comes after every type it holds by value. Storage
    /// (`cpl_raw<T>`) and references need only T's declaration, which is what lets a
    /// function value reach itself through a list.
    fn by_value_order(&self) -> Vec<CTy> {
        fn visit(e: &Emit, t: &CTy, done: &mut HashSet<CTy>, out: &mut Vec<CTy>) {
            let t = e.p.rep(t);
            let parts: Vec<CTy> = match t {
                CTy::Struct(..) => e.p.parts(t),
                CTy::Clo(k) => e.p.captures(k),
                CTy::Fn(_, _, ms) => ms.iter().map(|k| CTy::Clo(Box::new(k.clone()))).collect(),
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
                    xs.iter().for_each(|a| {
                        if let Arg::Value(y) = a {
                            expr(e, y)
                        }
                    });
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
                Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
                    xs.iter().for_each(|x| expr(self, x))
                }
                Stmt::Break | Stmt::Continue | Stmt::LendOut(_) | Stmt::Drop(_) => {}
            }
        }
    }
    fn name(&mut self, t: &CTy) -> String {
        if let Some(n) = self.names.get(t) {
            return n.clone();
        }
        let rep = self.p.rep(t);
        if rep != t {
            let n = self.name(rep);
            self.names.insert(t.clone(), n.clone());
            return n;
        }
        let n = match t {
            CTy::Int | CTy::Range(..) | CTy::RangeTo(..) | CTy::Dom(_) | CTy::Error => {
                "int64_t".to_string()
            }
            CTy::Bool => "bool".to_string(),
            CTy::Char => "cpl_char".to_string(),
            CTy::U64 => "uint64_t".to_string(),
            CTy::I128 => "__int128".to_string(),
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
            CTy::Fn(_, _, ms) => {
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
    fn sig(&self, i: usize, f: &Func<CTy>) -> String {
        let ps: Vec<String> = f
            .params
            .iter()
            .enumerate()
            .map(|(k, (l, lent))| {
                let t = &self.names[&f.locals[*l]];
                if *lent {
                    format!("{t}& l{l}")
                } else if f.views.get(k).copied().unwrap_or(false) {
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
    /// A struct's field types, in the layout table's order (not a closure's or a union's).
    fn fields(&self, t: &CTy) -> Option<Vec<CTy>> {
        let t = self.p.rep(t);
        let l = (self.lays.get(t)).filter(|_| matches!(t, CTy::Struct(..)))?;
        match &self.lays.shape[l as usize] {
            Shape::Struct(fs) => Some(
                fs.iter()
                    .map(|(_, f)| self.lays.ty[*f as usize].clone())
                    .collect(),
            ),
            _ => None,
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
            EK::DomLen(_) | EK::DomLo(_) | EK::DomHeld(_) | EK::DomCounted(_) => {
                "/* unresolved domain value */".to_string()
            }
            EK::HasBound(..) => "/* undecided bound test */".to_string(),
            EK::Copy(p) => self.place(p),
            EK::Lent(p) => format!("(&{})", self.place(p)),
            EK::AddrOf(p) => format!("(&{})", self.place(p)),
            EK::Tag(p) => format!("{}.tag", self.place(p)),
            EK::MakeStruct(xs) if self.fields(&e.ty).is_some() => {
                // A lent slot filled from a lent place gets the pointer, not what it points at.
                let fs = self.fields(&e.ty).unwrap_or_default();
                let xs: Vec<String> = (xs.iter().zip(&fs))
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
                let first = match args.first() {
                    Some(Arg::Value(x)) => ops::Operand::of(&x.ty),
                    _ => ops::Operand::Other,
                };
                let mut text = ops::cpp_text(*op, first).replace("$*", &a.join(", "));
                for (k, x) in a.iter().enumerate().rev() {
                    text = text.replace(&format!("${k}"), x);
                }
                if let Some(cell) = self.cells.get(&e.ty) {
                    text = text.replace("$W", &format!("INT64_C({cell})"));
                }
                match self.names.get(&e.ty) {
                    Some(t) => text.replace("$T", t),
                    None => text,
                }
            }
        }
    }
    fn leave(&self, what: Leave, ctx: Ctx) -> String {
        match (what, ctx.inner) {
            (Leave::Break, Inner::Loop(_)) => "break;".to_string(),
            (Leave::Break, Inner::Region(_)) => "return 1;".to_string(),
            (Leave::Continue, Inner::Loop(l)) => format!("goto next{l};"),
            (Leave::Continue, Inner::Region(false)) => "return 0;".to_string(),
            (Leave::Continue, Inner::Region(true)) => "return 3;".to_string(),
            // A `return` that was carried out of a region body.
            (Leave::Return, _) if ctx.in_lambda => "return 2;".to_string(),
            (Leave::Return, _) if ctx.fn_region => "return cpl_code;".to_string(),
            (Leave::Return, _) => "return cpl_ret;".to_string(),
            (Leave::Break | Leave::Continue, Inner::None) => "/* nothing to leave */;".to_string(),
        }
    }
    fn block(&self, b: &[Stmt<CTy>], depth: usize, s: &mut String, label: &mut usize, ctx: Ctx) {
        let pad = "    ".repeat(depth);
        for st in b {
            // `cpl_at` is set where the interpreter sets `Machine::at`: at an operation that can
            // trap, and around a call, which may run other lines of the user's file.
            let around = match st {
                Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) => match &e.kind {
                    EK::Call(_, _, site) => Some(self.line(*site)),
                    _ => None,
                },
                Stmt::Region { site, .. } => Some(self.line(*site)),
                Stmt::Yield(_) | Stmt::LendOut(_) => Some(0),
                _ => None,
            };
            let read = match st {
                Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) | Stmt::Yield(e) => Some(e),
                Stmt::If(e, ..) | Stmt::Switch(e, _) | Stmt::Return(Some(e)) => Some(e),
                _ => None,
            };
            let line = read.map_or(0, |e| self.trap_line(e));
            if line != 0 && around.is_none() {
                put_line!(s, "{pad}cpl_at = {line};");
            }
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
                Stmt::Break => put_line!(s, "{pad}{}", self.leave(Leave::Break, ctx)),
                Stmt::Continue => put_line!(s, "{pad}{}", self.leave(Leave::Continue, ctx)),
                Stmt::Return(e) => {
                    if ctx.in_lambda || ctx.fn_region {
                        if let (Some(e), false) = (e, ctx.fn_region) {
                            put_line!(s, "{pad}cpl_ret = {};", self.expr(e));
                        }
                        put_line!(s, "{pad}{}", self.leave(Leave::Return, ctx));
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
                    put_line!(s, "{pad}    if (c == 2) {}", self.leave(Leave::Return, ctx));
                    if *lend && !matches!(ctx.inner, Inner::None) {
                        put_line!(s, "{pad}    if (c == 1) {}", self.leave(Leave::Break, ctx));
                        put_line!(s, "{pad}    if (c == 3) {}", self.leave(Leave::Continue, ctx));
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
                // Events go to the trace stream in order (`cpl_tput`).
                Stmt::Event(ev) => {
                    // The code that shows an event's value places no events of its own.
                    let label = format!("ev {}", ev.label(self.funcs));
                    put_line!(s, "{pad}if (!cpl_capture) {{ cpl_tput({label:?}, {});", label.len());
                    for v in &ev.vals {
                        let v = self.expr(v);
                        put_line!(s, "{pad}{{ std::string cpl_s; cpl_capture = &cpl_s; {v}; cpl_capture = nullptr; cpl_tput(\" \", 1); cpl_tput(cpl_s.data(), cpl_s.size()); }}");
                    }
                    put_line!(s, "{pad}cpl_tput(\"\\n\", 1); }}");
                }
                // A lender finishes its own code first; the caller leaves after it.
                Stmt::LendOut(p) => put_line!(s, "{pad}cpl_code = body({});", self.place(p)),
                // Ownership's release runs here; a local in `Func::drops` is released when its
                // scope ends, and a value moved out of it was left empty.
                Stmt::Drop(p) => put_line!(s, "{pad}{} = {{}};", self.place(p)),
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

    /// An `F64` as C's `%a` prints it, so a result compares bit for bit.
    fn hex(x: f64) -> String {
        if x.is_nan() {
            return if x.is_sign_negative() { "-nan" } else { "nan" }.to_string();
        }
        if x.is_infinite() {
            return if x < 0.0 { "-inf" } else { "inf" }.to_string();
        }
        let sign = if x.is_sign_negative() { "-" } else { "" };
        let bits = x.to_bits();
        let (e, m) = ((bits >> 52) & 0x7ff, bits & ((1 << 52) - 1));
        if e == 0 && m == 0 {
            return format!("{sign}0x0p+0");
        }
        let (lead, e) = if e == 0 {
            (0, -1022)
        } else {
            (1, e as i64 - 1023)
        };
        let mut digits = format!("{m:013x}");
        while digits.ends_with('0') {
            digits.pop();
        }
        let dot = if digits.is_empty() { "" } else { "." };
        format!("{sign}0x{lead}{dot}{digits}p{e:+}")
    }

    fn expect<T: Display>(r: Result<T, Fault>) -> String {
        match r {
            Ok(v) => format!("{v}\n"),
            Err(f) => format!("trap: {}: line 0: {}\n", f.code(), f.message()),
        }
    }

    #[test]
    fn runtime_agrees_with_ops() {
        let memory = Limits::default().memory as i64;
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
                format!(
                    "printf(\"%zu\\n\", cpl_raw_new(INT64_C({a}), INT64_C(0), INT64_C(8)).size());"
                ),
                expect(ops::storage_size(a, 8, memory)),
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
        let floats = [
            -1e300,
            -9223372036854775808.0,
            -2.5,
            -1.0,
            -0.5,
            0.0,
            0.5,
            1.5,
            2.0,
            9223372036854774784.0,
            9223372036854775808.0,
            f64::NAN,
            f64::INFINITY,
        ];
        for x in floats {
            let c = format!("{:?}", f64::from_bits(x.to_bits()));
            let lit = if x.is_nan() {
                "__builtin_nan(\"\")".to_string()
            } else if x.is_infinite() {
                "__builtin_inf()".to_string()
            } else {
                c
            };
            for r in [
                Rounding::Floor,
                Rounding::Ceil,
                Rounding::Round,
                Rounding::Trunc,
            ] {
                case(
                    format!(
                        "printf(\"%lld\\n\", (long long)cpl_int_of_f64({}({lit})));",
                        r.cpp()
                    ),
                    expect(ops::to_int(r, x)),
                );
            }
            for f in [
                FloatFn::Sqrt,
                FloatFn::Sin,
                FloatFn::Cos,
                FloatFn::Tan,
                FloatFn::Asin,
                FloatFn::Acos,
                FloatFn::Atan2,
                FloatFn::Hypot,
                FloatFn::Exp,
                FloatFn::Ln,
            ] {
                case(
                    format!("printf(\"%a\\n\", {}({lit}, 0.75));", f.cpp()).replace(
                        ", 0.75)",
                        if matches!(f, FloatFn::Atan2 | FloatFn::Hypot) {
                            ", 0.75)"
                        } else {
                            ")"
                        },
                    ),
                    format!("{}\n", hex(ops::float_fn(f, x, 0.75).unwrap())),
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
        let halves = |w: i128| format!("{} {}", (w as u128 >> 64) as u64, w as u64);
        let wide = |w: i128| {
            let (hi, lo) = ((w as u128 >> 64) as u64, w as u64);
            format!("(__int128)(((unsigned __int128)UINT64_C({hi}) << 64) | UINT64_C({lo}))")
        };
        let wides = [
            i128::MIN,
            -(1 << 100),
            -3,
            -2,
            -1,
            0,
            1,
            2,
            3,
            64,
            127,
            128,
            1 << 64,
            i128::MAX,
        ];
        let wide_ops = [
            Intr::Add,
            Intr::Sub,
            Intr::Mul,
            Intr::Div,
            Intr::Rem,
            Intr::Pow,
            Intr::BitAnd,
            Intr::BitOr,
            Intr::BitXor,
        ];
        for op in wide_ops {
            let Some(ops::Cpp::Call(f)) = ops::cpp_wide(op) else {
                panic!("{op:?} has no I128 C++ helper");
            };
            for a in wides {
                for b in wides {
                    case(
                        format!("__int128 r = {f}({}, {}); printf(\"%llu %llu\\n\", (unsigned long long)((unsigned __int128)r >> 64), (unsigned long long)r);", wide(a), wide(b)),
                        expect(ops::wide(op, a, b).map(halves)),
                    );
                }
            }
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
            case(
                format!("double x; uint64_t b = UINT64_C({b}); memcpy(&x, &b, 8); printf(\"%llu\\n\", (unsigned long long)cpl_f64_bits(x));"),
                format!("{b}\n"),
            );
        }
        let mut cpp = runtime(&Limits::default());
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
