// cf2183d2.cp: C++17 written by cpl 0.2.2 from the CP Lang source below.
//
//     // Codeforces 2183 D2: compute the minimum number of operations by depth, then assign
//     // vertices bottom-up into slots so no parent shares a slot with its child.
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
//
//     var resp: List<List<Int>> = List.filled(ans, [])
//     var last_depth: [0..ans]Int = n + 1
//     var forbidden_loc: [0..n]?Int = none
//
//     for d in (0..n).rev:
//         if by_depth[d].is_empty: continue
//         var forbiddens: List<(Int, ?Int, Int)> = []
//         for (cur, prv) in by_depth[d]:
//             if let fl = forbidden_loc[cur]:
//                 forbiddens.push((cur, prv, fl))
//         if forbiddens.len == 1:
//             let (cur, prv, _) = forbiddens[0]
//             var loc = 0
//             while last_depth[loc] == d + 1: loc += 1
//             resp[loc].push(cur + 1)
//             last_depth[loc] = d
//             if let p = prv: forbidden_loc[p] = loc
//         else:
//             let k = forbiddens.len
//             for z in 0..k:
//                 let (cur, prv, _) = forbiddens[z]
//                 let loc = forbiddens[(z + 1) % k].2
//                 resp[loc].push(cur + 1)
//                 last_depth[loc] = d
//                 if let p = prv: forbidden_loc[p] = loc
//         var loc = 0
//         for (cur, prv) in by_depth[d]:
//             if forbidden_loc[cur] == none:
//                 while last_depth[loc] == d: loc += 1
//                 resp[loc].push(cur + 1)
//                 last_depth[loc] = d
//                 if let p = prv: forbidden_loc[p] = loc
//
//     print(ans)
//     for group in resp:
//         print(group.len, group)

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
inline ll emod(ll a, ll b) {
#ifdef LOCAL
    if (b == 0) { fputs("division by zero\n", stderr); exit(3); }
#endif
    if (b == -1) return 0;
    ll r = a % b, s = b >> 63; return r + ((r >> 63) & ((b ^ s) - s));
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
std::vector<std::vector<ll>> resp;
std::vector<ll> last_depth;
std::vector<std::optional<ll>> forbidden_loc;

void dfs(ll cur, const std::optional<ll>& prv, ll depth);
// Prints a `List<Int>`.
void show_1(const std::vector<ll>& v, int) {
    for (size_t i = 0; i < v.size(); i++) {
        if (i > 0) cpl::put_char(' ');
        cpl::put_int(v[i]);
    }
}


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
    resp = std::vector<std::vector<ll>>((size_t)(ans), std::vector<ll>());
    last_depth = std::vector<ll>(ans, n + 1);
    forbidden_loc = std::vector<std::optional<ll>>(n, std::optional<ll>());
    for (ll d_2 = n - 1; d_2 >= 0; d_2--) {
        if ((ll)cpl::at(by_depth, d_2).size() == 0) {
            continue;
        }
        std::vector<std::tuple<ll, std::optional<ll>, ll>> forbiddens = std::vector<std::tuple<ll, std::optional<ll>, ll>>();
        for (const std::tuple<ll, std::optional<ll>>& t4 : cpl::at(by_depth, d_2)) {
            ll cur = std::get<0>(t4);
            std::optional<ll> prv = std::get<1>(t4);
            std::optional<ll> t5 = forbidden_loc[cur];
            if (t5.has_value()) {
                ll fl = *t5;
                forbiddens.push_back(std::tuple<ll, std::optional<ll>, ll>{cur, prv, fl});
            }
        }
        if ((ll)forbiddens.size() == 1) {
            std::tuple<ll, std::optional<ll>, ll> t6 = cpl::at(forbiddens, 0);
            ll cur_2 = std::get<0>(t6);
            std::optional<ll> prv_2 = std::get<1>(t6);
            std::get<2>(t6);
            ll loc = 0;
            while (last_depth[loc] == d_2 + 1) {
                loc += 1;
            }
            cpl::at(resp, loc).push_back(cur_2 + 1);
            last_depth[loc] = d_2;
            std::optional<ll> t7 = prv_2;
            if (t7.has_value()) {
                ll p_2 = *t7;
                forbidden_loc[p_2] = std::optional<ll>(loc);
            }
        } else {
            ll k = (ll)forbiddens.size();
            for (ll z = 0; z < k; z++) {
                std::tuple<ll, std::optional<ll>, ll> t8 = cpl::at(forbiddens, z);
                ll cur_3 = std::get<0>(t8);
                std::optional<ll> prv_3 = std::get<1>(t8);
                std::get<2>(t8);
                ll loc_2 = std::get<2>(cpl::at(forbiddens, cpl::emod(z + 1, k)));
                cpl::at(resp, loc_2).push_back(cur_3 + 1);
                last_depth[loc_2] = d_2;
                std::optional<ll> t9 = prv_3;
                if (t9.has_value()) {
                    ll p_3 = *t9;
                    forbidden_loc[p_3] = std::optional<ll>(loc_2);
                }
            }
        }
        ll loc_3 = 0;
        for (const std::tuple<ll, std::optional<ll>>& t10 : cpl::at(by_depth, d_2)) {
            ll cur_4 = std::get<0>(t10);
            std::optional<ll> prv_4 = std::get<1>(t10);
            if (forbidden_loc[cur_4] == std::optional<ll>()) {
                while (last_depth[loc_3] == d_2) {
                    loc_3 += 1;
                }
                cpl::at(resp, loc_3).push_back(cur_4 + 1);
                last_depth[loc_3] = d_2;
                std::optional<ll> t11 = prv_4;
                if (t11.has_value()) {
                    ll p_4 = *t11;
                    forbidden_loc[p_4] = std::optional<ll>(loc_3);
                }
            }
        }
    }
    cpl::put_int(ans); cpl::put_line();
    for (const std::vector<ll>& group : resp) {
        cpl::put_int((ll)group.size()); cpl::put_char(' '); show_1(group, 6); cpl::put_line();
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
