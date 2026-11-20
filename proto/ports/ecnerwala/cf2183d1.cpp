// cf2183d1.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2183 D1: bucket tree vertices by depth; answer is max over depths of
//     // layer size, plus 1 when every vertex in a non-root layer shares one parent.
//     @!multitest
//     let n: Int = read()
//     var adj: List<List<Int>> = List.filled(n, [])
//     for _ in 0..n - 1:
//         let (u, v): (Int, Int) = read()
//         adj[u - 1].push(v - 1)
//         adj[v - 1].push(u - 1)
//
//     var by_depth: List<List<(Int, ?Int)>> = List.filled(n, [])
//
//     fn dfs(cur: Int, prv: ?Int, depth: Int):
//         by_depth[depth].push((cur, prv))
//         for nxt in adj[cur]:
//             if some(nxt) == prv: continue
//             dfs(nxt, cur, depth + 1)
//
//     dfs(0, none, 0)
//     var ans = 0
//     for d in 0..n:
//         if by_depth[d].is_empty: break
//         let sz = by_depth[d].len
//         let p0 = by_depth[d][0].1
//         let same_parent = d > 0 and all(p == p0 for (_, p) in by_depth[d])
//         ans max= sz + Int(same_parent)
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
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail("index outside the list"); return v[(size_t)i]; }
}  // namespace cpl

// The program.
namespace prog {
ll n = 0;
std::vector<std::vector<ll>> adj;
std::vector<std::vector<std::tuple<ll, std::optional<ll>>>> by_depth;
ll ans = 0;

void dfs(ll cur, const std::optional<ll>& prv, ll depth);

void dfs(ll cur, const std::optional<ll>& prv, ll depth) {
    cpl::at(by_depth, depth).push_back(std::tuple<ll, std::optional<ll>>{cur, prv});
    for (ll nxt : cpl::at(adj, cur)) {
        if (std::optional<ll>(nxt) == prv) {
            continue;
        }
        ::prog::dfs(nxt, std::optional<ll>(cur), depth + 1);
    }
}

bool setup() {
    return true;
}

void solve() {
    n = cpl::read_int();
    adj = std::vector<std::vector<ll>>((size_t)(n), std::vector<ll>());
    for (ll t0 = 0, t0_end = n - 1; t0 < t0_end; t0++) {
        std::tuple<ll, ll> t1 = std::tuple<ll, ll>{cpl::read_int(), cpl::read_int()};
        ll u = std::get<0>(std::move(t1));
        ll v = std::get<1>(std::move(t1));
        cpl::at(adj, u - 1).push_back(v - 1);
        cpl::at(adj, v - 1).push_back(u - 1);
    }
    by_depth = std::vector<std::vector<std::tuple<ll, std::optional<ll>>>>((size_t)(n), std::vector<std::tuple<ll, std::optional<ll>>>());
    ::prog::dfs(0, std::optional<ll>(), 0);
    ans = 0;
    for (ll d = 0; d < n; d++) {
        if ((ll)cpl::at(by_depth, d).size() == 0) {
            break;
        }
        ll sz = (ll)cpl::at(by_depth, d).size();
        std::optional<ll> p0 = std::get<1>(cpl::at(cpl::at(by_depth, d), 0));
        bool same_parent = d > 0 && [&]() -> bool {
            bool t3 = true;
            for (const std::tuple<ll, std::optional<ll>>& t2 : cpl::at(by_depth, d)) {
                std::get<0>(t2);
                std::optional<ll> p = std::get<1>(t2);
                if (!(p == p0)) {
                    t3 = false;
                    break;
                }
            }
            return t3;
        }();
        ans = std::max<ll>(ans, sz + (same_parent ? 1 : 0));
    }
    cpl::put_int(ans); cpl::put_line();
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
