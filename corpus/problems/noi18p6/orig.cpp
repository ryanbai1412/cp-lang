// noi18p6: count Hamiltonian cycles by a DP over the tree that merges children left to right.
// A state keeps the subtree's partial cycle as paths over its "slots": the subtree's top node
// (still owed its parent edge) and its first K and last K leaves (the only ones with ring edges
// still undecided); every other node already has degree 2. Ring edges between two leaves are
// decided at the merge that first joins them; wrap-around ring edges are decided at the root.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using u64 = unsigned long long;
const ll MOD = 998244353;

int n, K, A;
vector<vector<int>> ch;

struct St {
    bool top = false, closed = false;
    int cnt = 0, m = 0;  // cnt: leaves, capped at 2K + 1; m: slots
    int deg[16], par[16];  // par: other end of the slot's path, -1 if the slot has degree 2
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
    // keep the slots with keep[i]; a dropped slot must have degree 2
    bool keepOnly(const vector<bool>& keep) {
        int id[16], k = 0;
        for (int i = 0; i < m; i++) {
            if (!keep[i] && deg[i] != 2) return false;
            id[i] = keep[i] ? k++ : -1;
        }
        int d2[16], p2[16];
        for (int i = 0; i < m; i++)
            if (keep[i]) d2[id[i]] = deg[i], p2[id[i]] = par[i] < 0 ? -1 : id[par[i]];
        m = k;
        copy(d2, d2 + k, deg), copy(p2, p2 + k, par);
        return true;
    }
    u64 key() const {
        u64 x = top | closed << 1 | (u64)cnt << 2 | (u64)m << 6;
        for (int i = 0; i < m; i++) x = x << 5 | deg[i] << 3 | (par[i] < 0 ? 7 : par[i]);
        return x;
    }
};
using Map = unordered_map<u64, pair<St, ll>>;
void put(Map& M, const St& s, ll v) {
    auto [it, fresh] = M.try_emplace(s.key(), s, 0);
    it->second.second = (it->second.second + v) % MOD;
}

Map solve(int u) {
    Map cur;
    St s;
    if (u != 1 && ch[u].empty()) {
        s.cnt = 1, s.m = 1, s.deg[0] = 0, s.par[0] = 0;
        put(cur, s, 1);
        return cur;
    }
    s.top = true, s.m = 1, s.deg[0] = 0, s.par[0] = 0;
    put(cur, s, 1);
    for (int c : ch[u]) {
        Map sub = solve(c), nxt;
        for (auto& [k1, e1] : cur)
            for (auto& [k2, e2] : sub) {
                const St &a = e1.first, &b = e2.first;
                St t = a;
                int off = a.m;
                t.m = a.m + b.m, t.closed = a.closed || b.closed;
                if (a.closed && b.closed) continue;
                for (int i = 0; i < b.m; i++) t.deg[off + i] = b.deg[i], t.par[off + i] = b.par[i] < 0 ? -1 : b.par[i] + off;
                int l1 = a.m - 1, l2 = b.m - b.top;  // leaf slots of each side
                int bl = off + b.top;  // first leaf slot of b
                vector<int> opts;
                if (b.top) {
                    if (b.deg[0] == 0) continue;
                    opts = {b.deg[0] == 1};
                } else opts = {0, 1};
                vector<pair<int, int>> jn;
                for (int p = 1; p <= min(K, l1); p++)
                    for (int q = 1; q <= min(K, l2); q++)
                        if (p + q - 1 <= K) jn.push_back({a.m - p, bl + q - 1});
                int cnt = min(2 * K + 1, a.cnt + b.cnt);
                int L = l1 + l2;
                vector<bool> keep(t.m, true);
                if (b.top) keep[off] = false;
                if (cnt > 2 * K) {
                    // leaf order: a's leaf slots 1..l1 then b's bl..bl+l2-1; keep the first K and last K
                    for (int i = 0; i < L; i++) {
                        int slot = i < l1 ? 1 + i : bl + (i - l1);
                        if (i >= K && i < L - K) keep[slot] = false;
                    }
                }
                ll v = e1.second * e2.second % MOD;
                for (int use : opts)
                    for (int mask = 0; mask < 1 << jn.size(); mask++) {
                        St w = t;
                        bool ok = !use || w.add(0, off);
                        for (size_t j = 0; ok && j < jn.size(); j++)
                            if (mask >> j & 1) ok = w.add(jn[j].first, jn[j].second);
                        if (!ok || !w.keepOnly(keep)) continue;
                        w.cnt = cnt;
                        put(nxt, w, v);
                    }
            }
        cur.swap(nxt);
    }
    return cur;
}

int main() {
    scanf("%d %d", &n, &K);
    ch.assign(n + 1, {});
    for (int i = 2; i <= n; i++) {
        int f;
        scanf("%d", &f);
        ch[f].push_back(i);
    }
    for (int i = 2; i <= n; i++) A += ch[i].empty();
    Map res = solve(1);
    ll ans = 0;
    for (auto& [k, e] : res) {
        const St& s = e.first;
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
            if (ok) ans = (ans + e.second) % MOD;
        }
    }
    printf("%lld\n", ans);
}
