// noi18p6: count Hamiltonian cycles by a DP over the tree that merges children left to right.
// A state keeps the subtree's partial cycle as paths over its "slots": the subtree's top node
// (still owed its parent edge) and its first K and last K leaves (the only ones with ring edges
// still undecided); every other node already has degree 2. Ring edges between two leaves are
// decided at the merge that first joins them; wrap-around ring edges are decided at the root.
// States that can no longer finish are dropped: a closed cycle before the root's last merge,
// or a boundary leaf needing more ring edges than the leaves within K outside it can give.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using u64 = unsigned long long;
const ll MOD = 998244353;

int n, K, A;
vector<vector<int>> ch;

struct St {
    bool top = false, closed = false;
    int8_t cnt = 0, m = 0;  // cnt: leaves, capped at 2K + 1; m: slots
    int8_t deg[16], par[16];  // par: other end of the slot's path, -1 if the slot has degree 2
    bool add(int a, int b) {
        if (deg[a] == 2 || deg[b] == 2) return false;
        if (par[a] == b && a != b) {
            if (closed) return false;
            closed = true, deg[a]++, deg[b]++, par[a] = par[b] = -1;
            return true;
        }
        int ea = par[a], eb = par[b];
        deg[a]++, deg[b]++;
        if (deg[a] == 2) par[a] = -1;
        if (deg[b] == 2) par[b] = -1;
        par[ea] = eb, par[eb] = ea;
        return true;
    }
    // keep the slots in `keep`; every dropped slot already has degree 2
    void keepOnly(unsigned keep) {
        int8_t id[16], k = 0;
        for (int i = 0; i < m; i++) id[i] = keep >> i & 1 ? k++ : -1;
        for (int i = 0; i < m; i++)
            if (keep >> i & 1) deg[id[i]] = deg[i], par[id[i]] = par[i] < 0 ? -1 : id[par[i]];
        m = k;
    }
    u64 key() const {
        u64 x = top | closed << 1 | (u64)cnt << 2 | (u64)m << 6;
        for (int i = 0; i < m; i++) x = x << 5 | deg[i] << 3 | (par[i] < 0 ? 7 : par[i]);
        return x;
    }
    // a boundary leaf can only still get ring edges from the leaves within K outside it
    bool dead(bool last) const {
        if (closed && !last) return true;
        if (cnt <= 2 * K) return false;
        int l = m - top;
        for (int i = 0; i < K; i++)
            if (2 - deg[top + i] > K - i || 2 - deg[top + l - 1 - i] > K - i) return true;
        return false;
    }
};

struct Map {
    unordered_map<u64, int> at;
    vector<pair<St, ll>> v;
    void put(const St& s, ll x) {
        auto [it, fresh] = at.try_emplace(s.key(), (int)v.size());
        if (fresh) v.push_back({s, 0});
        ll& y = v[it->second].second;
        y = (y + x) % MOD;
    }
};

struct Merge {
    Map* out;
    int nj;
    int ja[8], jb[8];
    unsigned keep, check[8];  // check[j]: dropped slots whose last join edge is j
    int8_t cnt;
    bool last;
    ll v;
    void rec(int j, const St& w) {
        if (j == nj) {
            St r = w;
            r.keepOnly(keep);
            r.cnt = cnt;
            if (!r.dead(last)) out->put(r, v);
            return;
        }
        if (ok(w, check[j])) rec(j + 1, w);
        St x = w;
        if (x.add(ja[j], jb[j]) && ok(x, check[j])) rec(j + 1, x);
    }
    static bool ok(const St& w, unsigned slots) {
        for (; slots; slots &= slots - 1)
            if (w.deg[__builtin_ctz(slots)] != 2) return false;
        return true;
    }
};

Map solve(int u) {
    Map cur;
    St s;
    s.deg[0] = 0, s.par[0] = 0, s.m = 1;
    if (u != 1 && ch[u].empty()) {
        s.cnt = 1;
        cur.put(s, 1);
        return cur;
    }
    s.top = true;
    cur.put(s, 1);
    for (size_t ci = 0; ci < ch[u].size(); ci++) {
        Map sub = solve(ch[u][ci]), nxt;
        Merge g;
        g.out = &nxt;
        g.last = u == 1 && ci + 1 == ch[u].size();
        for (auto& [a, va] : cur.v)
            for (auto& [b, vb] : sub.v) {
                if (a.closed && b.closed) continue;
                if (b.top && b.deg[0] == 0) continue;
                St t = a;
                int off = a.m;
                t.m = a.m + b.m, t.closed = a.closed || b.closed;
                for (int i = 0; i < b.m; i++)
                    t.deg[off + i] = b.deg[i], t.par[off + i] = b.par[i] < 0 ? -1 : b.par[i] + off;
                int l1 = a.m - 1, l2 = b.m - b.top;  // leaf slots of each side
                int bl = off + b.top;  // first leaf slot of b
                g.nj = 0;
                for (int p = 1; p <= min(K, l1); p++)
                    for (int q = 1; q <= min(K, l2); q++)
                        if (p + q - 1 <= K) g.ja[g.nj] = a.m - p, g.jb[g.nj] = bl + q - 1, g.nj++;
                g.cnt = min(2 * K + 1, a.cnt + b.cnt);
                unsigned keep = (1u << t.m) - 1;
                if (b.top) keep &= ~(1u << off);
                if (g.cnt > 2 * K) {
                    // leaf order: a's leaf slots 1..l1 then b's bl..bl+l2-1; keep the first K and last K
                    int L = l1 + l2;
                    for (int i = K; i < L - K; i++) keep &= ~(1u << (i < l1 ? 1 + i : bl + (i - l1)));
                }
                g.keep = keep;
                unsigned drop = ~keep & ((1u << t.m) - 1), early = drop;
                for (int j = g.nj - 1; j >= 0; j--) {
                    unsigned e = (1u << g.ja[j] | 1u << g.jb[j]) & early;
                    g.check[j] = e, early &= ~e;
                }
                g.v = va * vb % MOD;
                // the tree edge from u to b's top (or to b itself when b is a leaf)
                for (int use = 0; use < 2; use++) {
                    if (b.top && use != (b.deg[0] == 1)) continue;
                    St w = t;
                    if (use && !w.add(0, off)) continue;
                    if (Merge::ok(w, early)) g.rec(0, w);
                }
            }
        cur = move(nxt);
    }
    return cur;
}

int main() {
    if (scanf("%d %d", &n, &K) != 2) return 1;
    ch.assign(n + 1, {});
    for (int i = 2; i <= n; i++) {
        int f;
        if (scanf("%d", &f) != 1) return 1;
        ch[f].push_back(i);
    }
    for (int i = 2; i <= n; i++) A += ch[i].empty();
    Map res = solve(1);
    ll ans = 0;
    for (auto& [s, val] : res.v) {
        if (s.deg[0] != 2) continue;
        int l = s.m - 1;
        vector<int> pos(l);
        for (int i = 0; i < l; i++) pos[i] = (A <= 2 * K || i < K) ? i + 1 : A - (l - i) + 1;
        vector<pair<int, int>> wr;
        for (int i = 0; i < l; i++)
            for (int j = i + 1; j < l; j++)
                if (pos[j] - pos[i] > K && A - (pos[j] - pos[i]) <= K) wr.push_back({1 + i, 1 + j});
        for (int mask = 0; mask < 1 << wr.size(); mask++) {
            St w = s;
            bool ok = true;
            for (size_t j = 0; ok && j < wr.size(); j++)
                if (mask >> j & 1) ok = w.add(wr[j].first, wr[j].second);
            if (!ok || !w.closed) continue;
            for (int i = 0; i < w.m; i++) ok &= w.deg[i] == 2;
            if (ok) ans = (ans + val) % MOD;
        }
    }
    printf("%lld\n", ans);
}
