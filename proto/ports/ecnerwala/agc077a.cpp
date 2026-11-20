// agc077a.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // AtCoder AGC 077 A: recursively split mismatched AB intervals via binary search
//     // on balanced B-count prefixes and emit pre-/post-reversal operations.
//     struct Seg:
//         l: Int
//         r: Int
//         d: Int
//
//     fn seg_len(s: Seg) -> Int = (s.r - s.l) * s.d + 1
//     fn seg_flip(s: Seg) -> Seg = Seg(s.r, s.l, -s.d)
//     fn seg_with(s: Seg, k: Int) -> Seg = Seg(s.l, s.l + s.d * (k - 1), s.d)
//     fn seg_without(s: Seg, k: Int) -> Seg = Seg(s.l + s.d * k, s.r, s.d)
//     fn seg_step(s: Seg) -> Seg = Seg(s.l + s.d, s.r, s.d)
//
//     @!multitest
//     let n: Int = read()
//     let xs: Str = read()
//     let ys: Str = read()
//
//     let cnt_x = count(xs[i] == 'B' for i in 0..n)
//     let cnt_y = count(ys[i] == 'B' for i in 0..n)
//
//     if cnt_x != cnt_y:
//         print("No")
//     elif xs == ys:
//         print("Yes")
//         print(0)
//     else:
//         var l = 0
//         var r = n - 1
//         while xs[l] == ys[l]: l += 1
//         while xs[r] == ys[r]: r -= 1
//         if xs[l] != 'A' or xs[r] != 'B':
//             print("No")
//         else:
//             var pref_x: [0..=n]Int = 0
//             var pref_y: [0..=n]Int = 0
//             for i in 0..n:
//                 pref_x[i + 1] = pref_x[i] + Int(xs[i] == 'B')
//                 pref_y[i + 1] = pref_y[i] + Int(ys[i] == 'B')
//
//             fn get_px(s: Seg) -> Int = if s.d == 1: pref_x[s.r + 1] - pref_x[s.l] else: pref_x[s.l + 1] - pref_x[s.r]
//             fn get_py(s: Seg) -> Int = if s.d == 1: pref_y[s.r + 1] - pref_y[s.l] else: pref_y[s.l + 1] - pref_y[s.r]
//
//             var pre_ops: List<(Int, Int)> = []
//             var post_ops: List<(Int, Int)> = []
//
//             fn rec(var x: Seg, var pre: Seg, var post: Seg, var y: Seg):
//                 while seg_len(x) > 0 and xs[x.l] == ys[y.l]:
//                     x = seg_step(x)
//                     y = seg_step(y)
//                     pre = seg_step(pre)
//                     post = seg_step(post)
//                 if seg_len(x) == 0: return
//                 let dir = Int(xs[x.l] == 'B') - Int(ys[y.l] == 'B')
//                 var mi = 1
//                 var ma = seg_len(x)
//                 while ma - mi > 1:
//                     let md = (mi + ma) / 2
//                     let vx = get_px(seg_with(x, md))
//                     let vy = get_py(seg_with(y, md))
//                     if (vx - vy) * dir > 0: mi = md
//                     else: ma = md
//                 rec(seg_without(x, ma), seg_without(pre, ma), seg_without(post, ma), seg_without(y, ma))
//                 x = seg_with(x, ma)
//                 pre = seg_with(pre, ma)
//                 post = seg_with(post, ma)
//                 y = seg_with(y, ma)
//                 if xs[x.l] == 'B':
//                     x = seg_flip(x)
//                     pre = seg_flip(pre)
//                     post = seg_flip(post)
//                     y = seg_flip(y)
//                 if pre.d == 1:
//                     pre_ops.push((pre.l, pre.r))
//                     x = seg_flip(x)
//                 else:
//                     post_ops.push((post.l, post.r))
//                     y = seg_flip(y)
//                 rec(x, pre, post, y)
//
//             rec(Seg(l + 1, r - 1, 1), Seg(l + 1, r - 1, 1), Seg(r - 1, l + 1, -1), Seg(r - 1, l + 1, -1))
//             pre_ops.push((l, r))
//             for op in post_ops.rev: pre_ops.push(op)
//             print("Yes")
//             print(pre_ops.len)
//             for (a, b) in pre_ops: print(a + 1, b + 1)

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
#include <functional>

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
inline std::string read_str() { skip_space(); size_t start = in_at; while (in_at < in.size() && !is_space(in[in_at])) in_at++; return in.substr(start, in_at - start); }
// The same for a divisor known to be positive.
inline ll divp(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    return a / b + ((a % b) >> 63);
}
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

struct Seg {
    ll l = 0;
    ll r = 0;
    ll d = 0;
    auto cpl_tie() const { return std::tie(l, r, d); }
};

ll n = 0;
std::string xs;
std::string ys;
ll cnt_x = 0;
ll cnt_y = 0;

ll seg_len(const Seg& s);
Seg seg_flip(const Seg& s);
Seg seg_with(const Seg& s, ll k);
Seg seg_without(const Seg& s, ll k);
Seg seg_step(const Seg& s);

ll seg_len(const Seg& s) {
    return (s.r - s.l) * s.d + 1;
}

Seg seg_flip(const Seg& s) {
    return Seg{s.r, s.l, -s.d};
}

Seg seg_with(const Seg& s, ll k) {
    return Seg{s.l, s.l + s.d * (k - 1), s.d};
}

Seg seg_without(const Seg& s, ll k) {
    return Seg{s.l + s.d * k, s.r, s.d};
}

Seg seg_step(const Seg& s) {
    return Seg{s.l + s.d, s.r, s.d};
}

bool setup() {
    return true;
}

void solve() {
    n = cpl::read_int();
    xs = cpl::read_str();
    ys = cpl::read_str();
    ll t0 = 0;
    for (ll i = 0; i < n; i++) {
        if ((cpl::u8)xs[i] == 'B') {
            t0 += 1;
        }
    }
    cnt_x = t0;
    ll t1 = 0;
    for (ll i_2 = 0; i_2 < n; i_2++) {
        if ((cpl::u8)ys[i_2] == 'B') {
            t1 += 1;
        }
    }
    cnt_y = t1;
    if (cnt_x != cnt_y) {
        cpl::put_str(std::string("No")); cpl::put_line();
    } else if (xs == ys) {
        cpl::put_str(std::string("Yes")); cpl::put_line();
        cpl::put_int(0); cpl::put_line();
    } else {
        ll l = 0;
        ll r = n - 1;
        while ((cpl::u8)xs[l] == (cpl::u8)ys[l]) {
            l += 1;
        }
        while ((cpl::u8)xs[r] == (cpl::u8)ys[r]) {
            r -= 1;
        }
        if ((cpl::u8)xs[l] != 'A' || (cpl::u8)xs[r] != 'B') {
            cpl::put_str(std::string("No")); cpl::put_line();
        } else {
            std::vector<ll> pref_x(n + 1, 0);
            std::vector<ll> pref_y(n + 1, 0);
            for (ll i_3 = 0; i_3 < n; i_3++) {
                pref_x[i_3 + 1] = pref_x[i_3] + ((cpl::u8)xs[i_3] == 'B' ? 1 : 0);
                pref_y[i_3 + 1] = pref_y[i_3] + ((cpl::u8)ys[i_3] == 'B' ? 1 : 0);
            }
            std::function<ll(Seg)> get_px = [&pref_x_2 = pref_x](const Seg& s) mutable -> ll {
                if (s.d == 1) {
                    return pref_x_2[s.r + 1] - pref_x_2[s.l];
                } else {
                    return pref_x_2[s.l + 1] - pref_x_2[s.r];
                }
            };
            std::function<ll(Seg)> get_py = [&pref_y_2 = pref_y](const Seg& s) mutable -> ll {
                if (s.d == 1) {
                    return pref_y_2[s.r + 1] - pref_y_2[s.l];
                } else {
                    return pref_y_2[s.l + 1] - pref_y_2[s.r];
                }
            };
            std::vector<std::tuple<ll, ll>> pre_ops = std::vector<std::tuple<ll, ll>>();
            std::vector<std::tuple<ll, ll>> post_ops = std::vector<std::tuple<ll, ll>>();
            std::function<void(Seg, Seg, Seg, Seg)> rec = [&get_px_2 = get_px, &get_py_2 = get_py, &rec_2 = rec, &pre_ops_2 = pre_ops, &post_ops_2 = post_ops](const Seg& x, const Seg& pre, const Seg& post, const Seg& y) mutable -> void {
                Seg x_2 = x;
                Seg pre_2 = pre;
                Seg post_2 = post;
                Seg y_2 = y;
                while (::prog::seg_len(x_2) > 0 && (cpl::u8)xs[x_2.l] == (cpl::u8)ys[y_2.l]) {
                    x_2 = ::prog::seg_step(x_2);
                    y_2 = ::prog::seg_step(y_2);
                    pre_2 = ::prog::seg_step(pre_2);
                    post_2 = ::prog::seg_step(post_2);
                }
                if (::prog::seg_len(x_2) == 0) {
                    return;
                }
                ll dir = ((ll)((cpl::u8)xs[x_2.l] == 'B' ? 1 : 0)) - ((cpl::u8)ys[y_2.l] == 'B' ? 1 : 0);
                ll mi = 1;
                ll ma = ::prog::seg_len(x_2);
                while (ma - mi > 1) {
                    ll md = cpl::divp(mi + ma, 2);
                    ll vx = get_px_2(::prog::seg_with(x_2, md));
                    ll vy = get_py_2(::prog::seg_with(y_2, md));
                    if ((vx - vy) * dir > 0) {
                        mi = md;
                    } else {
                        ma = md;
                    }
                }
                rec_2(::prog::seg_without(x_2, ma), ::prog::seg_without(pre_2, ma), ::prog::seg_without(post_2, ma), ::prog::seg_without(y_2, ma));
                x_2 = ::prog::seg_with(x_2, ma);
                pre_2 = ::prog::seg_with(pre_2, ma);
                post_2 = ::prog::seg_with(post_2, ma);
                y_2 = ::prog::seg_with(y_2, ma);
                if ((cpl::u8)xs[x_2.l] == 'B') {
                    x_2 = ::prog::seg_flip(x_2);
                    pre_2 = ::prog::seg_flip(pre_2);
                    post_2 = ::prog::seg_flip(post_2);
                    y_2 = ::prog::seg_flip(y_2);
                }
                if (pre_2.d == 1) {
                    pre_ops_2.push_back(std::tuple<ll, ll>{pre_2.l, pre_2.r});
                    x_2 = ::prog::seg_flip(x_2);
                } else {
                    post_ops_2.push_back(std::tuple<ll, ll>{post_2.l, post_2.r});
                    y_2 = ::prog::seg_flip(y_2);
                }
                rec_2(x_2, pre_2, post_2, y_2);
            };
            rec(Seg{l + 1, r - 1, 1}, Seg{l + 1, r - 1, 1}, Seg{r - 1, l + 1, -1}, Seg{r - 1, l + 1, -1});
            pre_ops.push_back(std::tuple<ll, ll>{l, r});
            {
                const std::vector<std::tuple<ll, ll>>& t3 = post_ops;
                for (auto t4 = t3.rbegin(); t4 != t3.rend(); ++t4) {
                    const std::tuple<ll, ll>& op = *t4;
                    pre_ops.push_back(op);
                }
            }
            cpl::put_str(std::string("Yes")); cpl::put_line();
            cpl::put_int((ll)pre_ops.size()); cpl::put_line();
            for (const std::tuple<ll, ll>& t2 : pre_ops) {
                ll a = std::get<0>(t2);
                ll b = std::get<1>(t2);
                cpl::put_int(a + 1); cpl::put_char(' '); cpl::put_int(b + 1); cpl::put_line();
            }
        }
    }
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
