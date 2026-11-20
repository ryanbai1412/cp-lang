// ucup4s25e.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // UCup 4 Stage 25 (Zagreb) E: divide-and-conquer Minkowski sum over prefix-sum
//     // upper convex hulls to find maximum subarray sum per length, then answer offline
//     // linear queries on the resulting upper hull.
//     struct Pt:
//         x: Int
//         y: Int
//
//     fn cross(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x
//     fn cross3(a: Pt, b: Pt, c: Pt) -> Int = cross(Pt(b.x - a.x, b.y - a.y), Pt(c.x - a.x, c.y - a.y))
//     fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y
//
//     fn upper_hull(pts: List<Pt>) -> List<Pt>:
//         var res: List<Pt> = []
//         for p in pts:
//             while res.len >= 2:
//                 let top = res.pop()!
//                 if cross3(res.last!, top, p) < 0:
//                     res.push(top)
//                     break
//             res.push(p)
//         res
//
//     @!multitest
//     let (n, q): (Int, Int) = read()
//     let a: [0..n]Int = read()
//
//     var pref: [0..=n]Pt = Pt(0, 0)
//     for i in 0..n:
//         pref[i + 1] = Pt(i + 1, pref[i].y + a[i])
//
//     var maxs: [0..n]Pt = Pt(0, 0)
//     for i in 0..n: maxs[i] = pref[i + 1]
//
//     fn try_max(p: Pt):
//         maxs[p.x - 1].y max= p.y
//
//     fn solve_dc(lo: Int, hi: Int):
//         if hi - lo <= 1: return
//         let md = (lo + hi) / 2
//         solve_dc(lo, md)
//         solve_dc(md, hi)
//         let los = upper_hull([Pt(-pref[i].x, -pref[i].y) for i in (lo..md).rev])
//         let his = upper_hull([pref[i] for i in md..hi])
//         var i = 0
//         var j = 0
//         try_max(Pt(los[0].x + his[0].x, los[0].y + his[0].y))
//         while i + 1 < los.len or j + 1 < his.len:
//             let step_i = i + 1 < los.len and (
//                 j + 1 == his.len or
//                 cross(
//                     Pt(los[i + 1].x - los[i].x, los[i + 1].y - los[i].y),
//                     Pt(his[j + 1].x - his[j].x, his[j + 1].y - his[j].y)
//                 ) < 0
//             )
//             if step_i: i += 1
//             else: j += 1
//             try_max(Pt(los[i].x + his[j].x, los[i].y + his[j].y))
//
//     solve_dc(0, n + 1)
//     let hull = upper_hull([maxs[i] for i in 0..n])
//
//     let qs_in: [0..q]Int = read()
//     let queries = sorted([(qs_in[idx], idx) for idx in 0..q])
//     var ans: [0..q]Int = 0
//     var hi_idx = 0
//     for (v, q_id) in queries:
//         let cnd = Pt(v, 1)
//         while hi_idx + 1 < hull.len and dot(Pt(hull[hi_idx + 1].x - hull[hi_idx].x, hull[hi_idx + 1].y - hull[hi_idx].y), cnd) >= 0:
//             hi_idx += 1
//         ans[q_id] = dot(hull[hi_idx], cnd)
//     print(ans)

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <map>
#include <optional>
#include <set>
#include <string>
#include <tuple>
#include <vector>

using ll = long long;

// What the program below leans on.
namespace cpl {
using u8 = unsigned char;
// Output is collected here and written once at the end.
std::string out;
inline void flush() { fwrite(out.data(), 1, out.size(), stdout); out.clear(); }
inline void put_char(u8 c) { out.push_back((char)c); }
inline void put_str(const std::string& s) { out += s; }
inline void put_str(const char* s) { out += s; }
inline void put_line() { out.push_back('\n'); if (out.size() >= (1u << 16)) flush(); }
inline void put_int(ll x) { char b[24]; out.append(b, (size_t)snprintf(b, sizeof b, "%lld", x)); }
[[noreturn]] inline void fail(const char* why) { flush(); fputs(why, stderr); fputc('\n', stderr); exit(3); }
// All of the input, read once.
std::string in;
size_t in_at = 0;
inline void read_all() { char b[1 << 16]; size_t n; while ((n = fread(b, 1, sizeof b, stdin)) > 0) in.append(b, n); }
inline bool is_space(char c) { return c == ' ' || c == '\n' || c == '\t' || c == '\r' || c == '\f'; }
// An interactive program reads a line when it needs one: the judge
// answers only after it has seen the query.
bool live = false;
inline bool more() { if (!live) return false; char b[1 << 12]; bool got = false; while (fgets(b, sizeof b, stdin)) { got = true; in += b; if (in.back() == '\n') break; } return got; }
inline void skip_space() { for (;;) { while (in_at < in.size() && is_space(in[in_at])) in_at++; if (in_at < in.size() || !more()) return; } }
inline ll read_int() {
    skip_space();
    bool negative = false;
    if (in_at < in.size() && (in[in_at] == '-' || in[in_at] == '+')) negative = in[in_at++] == '-';
    ll x = 0;  // built downward, so the smallest Int reads without overflow
    while (in_at < in.size() && in[in_at] >= '0' && in[in_at] <= '9') x = x * 10 - (in[in_at++] - '0');
    return negative ? x : -x;
}
// The same for a divisor known to be positive.
inline ll divp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    return a / b + ((a % b) >> 63);
}
template <class O> decltype(auto) have(O& o) {
    if (!o.has_value()) fail("`!` on `none`: there is no value here");
    return *o;
}
template <class O, std::enable_if_t<!std::is_lvalue_reference<O>::value, int> = 0> auto have(O&& o) {
    if (!o.has_value()) fail("`!` on `none`: there is no value here");
    return std::move(*o);
}
template <class T> std::optional<T> pop(std::vector<T>& v) { if (v.empty()) return std::nullopt; T x = std::move(v.back()); v.pop_back(); return x; }
template <class T> std::optional<T> last(const std::vector<T>& v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> std::vector<T> sorted(std::vector<T> v) { std::sort(v.begin(), v.end()); return v; }
template <class T> std::vector<T> sorted(const std::set<T>& s) { return std::vector<T>(s.begin(), s.end()); }
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
}  // namespace cpl

// The program.
namespace prog {
// A struct compares field by field, in order.
template <class T> auto operator==(const T& a, const T& b) -> decltype(a.cpl_tie() == b.cpl_tie()) { return a.cpl_tie() == b.cpl_tie(); }
template <class T> auto operator!=(const T& a, const T& b) -> decltype(a.cpl_tie() != b.cpl_tie()) { return a.cpl_tie() != b.cpl_tie(); }
template <class T> auto operator<(const T& a, const T& b) -> decltype(a.cpl_tie() < b.cpl_tie()) { return a.cpl_tie() < b.cpl_tie(); }
template <class T> auto operator<=(const T& a, const T& b) -> decltype(a.cpl_tie() <= b.cpl_tie()) { return a.cpl_tie() <= b.cpl_tie(); }
template <class T> auto operator>(const T& a, const T& b) -> decltype(a.cpl_tie() > b.cpl_tie()) { return a.cpl_tie() > b.cpl_tie(); }
template <class T> auto operator>=(const T& a, const T& b) -> decltype(a.cpl_tie() >= b.cpl_tie()) { return a.cpl_tie() >= b.cpl_tie(); }

struct Pt {
    ll x = 0;
    ll y = 0;
    auto cpl_tie() const { return std::tie(x, y); }
};

ll n = 0;
ll q = 0;
std::vector<ll> a;
std::vector<Pt> pref;
std::vector<Pt> maxs;
std::vector<Pt> hull;
std::vector<ll> qs_in;
std::vector<std::tuple<ll, ll>> queries;
std::vector<ll> ans;
ll hi_idx = 0;

ll cross(const Pt& a_2, const Pt& b);
ll cross3(const Pt& a_2, const Pt& b, const Pt& c);
ll dot(const Pt& a_2, const Pt& b);
std::vector<Pt> upper_hull(const std::vector<Pt>& pts);
void try_max(const Pt& p);
void solve_dc(ll lo, ll hi);
// Prints a `[Int]Int`.
void show_1(const std::vector<ll>& v, int) {
    for (size_t i = 0; i < v.size(); i++) {
        if (i > 0) cpl::put_char(' ');
        cpl::put_int(v[i]);
    }
}


ll cross(const Pt& a_2, const Pt& b) {
    return a_2.x * b.y - a_2.y * b.x;
}

ll cross3(const Pt& a_2, const Pt& b, const Pt& c) {
    return ::prog::cross(Pt{b.x - a_2.x, b.y - a_2.y}, Pt{c.x - a_2.x, c.y - a_2.y});
}

ll dot(const Pt& a_2, const Pt& b) {
    return a_2.x * b.x + a_2.y * b.y;
}

std::vector<Pt> upper_hull(const std::vector<Pt>& pts) {
    std::vector<Pt> res = std::vector<Pt>();
    for (const Pt& p : pts) {
        while ((ll)res.size() >= 2) {
            Pt top = cpl::have(cpl::pop(res));
            if (::prog::cross3(cpl::have(cpl::last(res)), top, p) < 0) {
                res.push_back(top);
                break;
            }
        }
        res.push_back(p);
    }
    return res;
}

void try_max(const Pt& p) {
    maxs[p.x - 1].y = std::max<ll>(maxs[p.x - 1].y, p.y);
}

void solve_dc(ll lo, ll hi) {
    if (hi - lo <= 1) {
        return;
    }
    ll md = cpl::divp(lo + hi, 2);
    solve_dc(lo, md);
    solve_dc(md, hi);
    std::vector<Pt> los = ::prog::upper_hull([&]() -> std::vector<Pt> {
        std::vector<Pt> t0 = std::vector<Pt>();
        for (ll i = md - 1, i_end = lo; i >= i_end; i--) {
            t0.push_back(Pt{-pref[i].x, -pref[i].y});
        }
        return t0;
    }());
    std::vector<Pt> his = ::prog::upper_hull([&]() -> std::vector<Pt> {
        std::vector<Pt> t1 = std::vector<Pt>();
        for (ll i_2 = md, i_2_end = hi; i_2 < i_2_end; i_2++) {
            t1.push_back(pref[i_2]);
        }
        return t1;
    }());
    ll i_3 = 0;
    ll j = 0;
    ::prog::try_max(Pt{cpl::at(los, 0).x + cpl::at(his, 0).x, cpl::at(los, 0).y + cpl::at(his, 0).y});
    while (i_3 + 1 < (ll)los.size() || j + 1 < (ll)his.size()) {
        bool step_i = i_3 + 1 < (ll)los.size() && (j + 1 == (ll)his.size() || ::prog::cross(Pt{cpl::at(los, i_3 + 1).x - cpl::at(los, i_3).x, cpl::at(los, i_3 + 1).y - cpl::at(los, i_3).y}, Pt{cpl::at(his, j + 1).x - cpl::at(his, j).x, cpl::at(his, j + 1).y - cpl::at(his, j).y}) < 0);
        if (step_i) {
            i_3 += 1;
        } else {
            j += 1;
        }
        ::prog::try_max(Pt{cpl::at(los, i_3).x + cpl::at(his, j).x, cpl::at(los, i_3).y + cpl::at(his, j).y});
    }
}

bool setup() {
    return true;
}

void solve() {
    std::tuple<ll, ll> t0 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
    n = std::get<0>(t0);
    q = std::get<1>(t0);
    a.assign(n, ll());
    for (auto& t4 : a) t4 = cpl::read_int();
    pref = std::vector<Pt>(n + 1, Pt{0, 0});
    for (ll i = 0; i < n; i++) {
        pref[i + 1] = Pt{i + 1, pref[i].y + a[i]};
    }
    maxs = std::vector<Pt>(n, Pt{0, 0});
    for (ll i_2 = 0; i_2 < n; i_2++) {
        maxs[i_2] = pref[i_2 + 1];
    }
    solve_dc(0, n + 1);
    hull = ::prog::upper_hull([&]() -> std::vector<Pt> {
        std::vector<Pt> t1 = std::vector<Pt>();
        for (ll i_3 = 0; i_3 < n; i_3++) {
            t1.push_back(maxs[i_3]);
        }
        return t1;
    }());
    qs_in.assign(q, ll());
    for (auto& t5 : qs_in) t5 = cpl::read_int();
    queries = cpl::sorted([&]() -> std::vector<std::tuple<ll, ll>> {
        std::vector<std::tuple<ll, ll>> t2 = std::vector<std::tuple<ll, ll>>();
        for (ll idx = 0; idx < q; idx++) {
            t2.push_back(std::tuple<ll, ll>{qs_in[idx], idx});
        }
        return t2;
    }());
    ans = std::vector<ll>(q, 0);
    hi_idx = 0;
    for (const std::tuple<ll, ll>& t3 : queries) {
        ll v = std::get<0>(t3);
        ll q_id = std::get<1>(t3);
        Pt cnd = Pt{v, 1};
        while (hi_idx + 1 < (ll)hull.size() && ::prog::dot(Pt{cpl::at(hull, hi_idx + 1).x - cpl::at(hull, hi_idx).x, cpl::at(hull, hi_idx + 1).y - cpl::at(hull, hi_idx).y}, cnd) >= 0) {
            hi_idx += 1;
        }
        ans[q_id] = ::prog::dot(cpl::at(hull, hi_idx), cnd);
    }
    show_1(ans, 6); cpl::put_line();
}
}  // namespace prog

// Deep recursion: the program runs on a 256 MB stack of its own.
#if defined(__x86_64__) && defined(__linux__) && !defined(__SANITIZE_ADDRESS__)
extern "C" void cpl_switch_stack(void (*run)(), void* top);
__asm__(".text\n.globl cpl_switch_stack\n.type cpl_switch_stack,@function\ncpl_switch_stack:\n"
        "  pushq %rbp\n  movq %rsp, %rbp\n  movq %rsi, %rsp\n  callq *%rdi\n"
        "  movq %rbp, %rsp\n  popq %rbp\n  ret\n.size cpl_switch_stack,.-cpl_switch_stack\n");
static void cpl_on_big_stack(void (*run)()) {
    const size_t size = (size_t)256 << 20;
    char* stack = (char*)std::malloc(size);
    if (!stack) return run();
    cpl_switch_stack(run, stack + size);
}
#else
static void cpl_on_big_stack(void (*run)()) { run(); }
#endif

static void cpl_main() {
    cpl::read_all();
    if (prog::setup()) for (ll tests = cpl::read_int(); tests > 0; tests--) prog::solve();
    cpl::flush();
}

int main() {
    cpl_on_big_stack(cpl_main);
    return 0;
}
