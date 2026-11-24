// NOI 2005 Party (output-only; scored against the organizers' network). A degree-bounded
// maximum spanning tree is NP-hard, so: greedy Kruskal under the bounds on noisy weights,
// joining leftover components by exchanges, then single-edge swaps (add a non-tree edge,
// drop the lightest path edge the bounds allow) until none helps, restarted until a fixed
// work budget runs out; then simulated annealing on the same swaps, with over-full people
// allowed at a penalty, from the best tree. checker.py accepts any valid network at least as comfortable.
#include <bits/stdc++.h>
using namespace std;

int n, m;
vector<int> U, V, K;
vector<long long> C;
long long budget = 150'000'000;
mt19937 rng(2005);

struct Tree {
    vector<char> in;
    vector<int> deg;
    vector<vector<int>> adj;  // edge ids
    vector<int> par, pe, depth;
    void add(int e) { in[e] = 1, deg[U[e]]++, deg[V[e]]++, adj[U[e]].push_back(e), adj[V[e]].push_back(e); }
    void del(int e) {
        in[e] = 0, deg[U[e]]--, deg[V[e]]--;
        for (int x : {U[e], V[e]}) adj[x].erase(find(adj[x].begin(), adj[x].end(), e));
    }
    // roots every component; returns component ids
    vector<int> root() {
        par.assign(n, -1), pe.assign(n, -1), depth.assign(n, -1);
        vector<int> comp(n, -1), st;
        for (int r = 0, c = 0; r < n; r++)
            if (comp[r] < 0) {
                comp[r] = c, depth[r] = 0, st = {r};
                while (!st.empty()) {
                    int x = st.back();
                    st.pop_back();
                    for (int e : adj[x]) {
                        int y = U[e] ^ V[e] ^ x;
                        if (comp[y] < 0) comp[y] = c, par[y] = x, pe[y] = e, depth[y] = depth[x] + 1, st.push_back(y);
                    }
                }
                budget -= n, c++;
            }
        return comp;
    }
};

bool connect(Tree& t) {
    for (int round = 0; round < 20 * n; round++) {
        auto comp = t.root();
        if (*max_element(comp.begin(), comp.end()) == 0) return true;
        int both = -1, one = -1, ties = 0;
        for (int e = 0; e < m; e++) {
            if (comp[U[e]] == comp[V[e]]) continue;
            bool fu = t.deg[U[e]] < K[U[e]], fv = t.deg[V[e]] < K[V[e]];
            if (fu && fv && (both < 0 || C[e] > C[both])) both = e;
            if (fu != fv && rng() % ++ties == 0) one = e;
        }
        budget -= m;
        if (both >= 0) { t.add(both); continue; }
        if (one < 0) return false;
        int full = t.deg[U[one]] < K[U[one]] ? V[one] : U[one];
        int drop = t.adj[full][rng() % t.adj[full].size()];
        t.del(drop), t.add(one);
    }
    return false;
}

void improve(Tree& t) {
    for (bool changed = true; changed && budget > 0;) {
        changed = false;
        t.root();
        vector<int> order(m);
        iota(order.begin(), order.end(), 0);
        shuffle(order.begin(), order.end(), rng);
        for (int e : order) {
            if (t.in[e]) continue;
            int u = U[e], v = V[e];
            bool fu = t.deg[u] < K[u], fv = t.deg[v] < K[v];
            if (!fu && !fv) continue;
            int best = -1;
            for (int a = u, b = v; a != b;) {
                int& x = t.depth[a] >= t.depth[b] ? a : b;
                int f = t.pe[x];
                bool ok = (fu || U[f] == u || V[f] == u) && (fv || U[f] == v || V[f] == v);
                if (ok && C[f] < C[e] && (best < 0 || C[f] < C[best])) best = f;
                x = t.par[x], budget--;
            }
            if (best >= 0) {
                t.del(best), t.add(e), changed = true;
                t.root();
            }
        }
    }
}


// annealing over spanning trees: swap a non-tree edge in for an edge of its cycle; a person
// over their bound costs `pen` per extra contact; the best tree within the bounds is kept
long long anneal(vector<char>& in, long long steps) {
    vector<vector<int>> adj(n);
    vector<int> deg(n), par(n), pe(n), seen(n, 0), st;
    long long w = 0, over = 0, pen = 2 * *max_element(C.begin(), C.end()) + 1;
    for (int e = 0; e < m; e++)
        if (in[e]) adj[U[e]].push_back(e), adj[V[e]].push_back(e), deg[U[e]]++, deg[V[e]]++, w += C[e];
    for (int i = 0; i < n; i++) over += max(0, deg[i] - K[i]);
    long long bestw = over ? LLONG_MIN : w;
    vector<char> best = in;
    double t0 = 0.05 * pen, t1 = 0.5;
    for (long long it = 0, stamp = 0; it < steps; it++) {
        int e = rng() % m;
        if (in[e]) continue;
        int u = U[e], v = V[e];
        seen[u] = ++stamp, st = {u};
        while (!st.empty() && seen[v] != stamp) {
            int x = st.back();
            st.pop_back();
            for (int f : adj[x]) {
                int y = U[f] ^ V[f] ^ x;
                if (seen[y] != stamp) seen[y] = stamp, par[y] = x, pe[y] = f, st.push_back(y);
            }
        }
        int len = 0;
        for (int x = v; x != u; x = par[x]) len++;
        int pick = rng() % len, f = -1;
        for (int x = v; pick >= 0; x = par[x], pick--) f = pe[x];
        auto ex = [&](int x, int dd) { return max(0, deg[x] + dd - K[x]) - max(0, deg[x] - K[x]); };
        long long dover = 0;
        map<int, int> dd{{u, 1}, {v, 1}};
        dd[U[f]]--, dd[V[f]]--;
        for (auto [x, d] : dd) dover += ex(x, d);
        long long delta = C[e] - C[f] - pen * dover;
        double t = t0 * pow(t1 / t0, (double)it / steps);
        if (delta < 0 && exp(delta / t) * 4294967296.0 <= rng()) continue;
        for (auto [x, d] : dd) deg[x] += d;
        over += dover, w += C[e] - C[f], in[e] = 1, in[f] = 0;
        adj[u].push_back(e), adj[v].push_back(e);
        for (int x : {U[f], V[f]}) adj[x].erase(find(adj[x].begin(), adj[x].end(), f));
        if (!over && w > bestw) bestw = w, best = in;
    }
    in = best;
    return bestw;
}

int main() {
    int T;
    cin >> T >> n >> m;
    K.resize(n);
    for (auto& k : K) cin >> k;
    U.resize(m), V.resize(m), C.resize(m);
    for (int e = 0; e < m; e++) cin >> U[e] >> V[e] >> C[e], U[e]--, V[e]--;
    long long bestw = LLONG_MIN;
    vector<int> best;
    for (int start = 0; start == 0 || (budget > 0 && start < 200); start++) {
        Tree t{vector<char>(m), vector<int>(n), vector<vector<int>>(n)};
        vector<pair<double, int>> order(m);
        for (int e = 0; e < m; e++) order[e] = {C[e] * (start ? 1 + 0.3 * (rng() / 4294967296.0) : 1.0), e};
        sort(order.rbegin(), order.rend());
        vector<int> dsu(n);
        iota(dsu.begin(), dsu.end(), 0);
        auto find = [&](int x) { while (dsu[x] != x) x = dsu[x] = dsu[dsu[x]]; return x; };
        for (auto [w, e] : order)
            if (t.deg[U[e]] < K[U[e]] && t.deg[V[e]] < K[V[e]] && find(U[e]) != find(V[e]))
                dsu[find(U[e])] = find(V[e]), t.add(e);
        if (!connect(t)) continue;
        improve(t);
        long long w = 0;
        for (int e = 0; e < m; e++) if (t.in[e]) w += C[e];
        if (w > bestw) {
            bestw = w, best.clear();
            for (int e = 0; e < m; e++) if (t.in[e]) best.push_back(e + 1);
        }
    }
    vector<char> in(m);
    if (best.empty()) {
        vector<pair<long long, int>> order(m);
        for (int e = 0; e < m; e++) order[e] = {C[e], e};
        sort(order.rbegin(), order.rend());
        vector<int> dsu(n);
        iota(dsu.begin(), dsu.end(), 0);
        auto find = [&](int x) { while (dsu[x] != x) x = dsu[x] = dsu[dsu[x]]; return x; };
        for (auto [w, e] : order)
            if (find(U[e]) != find(V[e])) dsu[find(U[e])] = find(V[e]), in[e] = 1;
    } else
        for (int e : best) in[e - 1] = 1;
    if (n > 2 && m >= n) {
        long long w = anneal(in, 300'000'000LL / n);
        if (w > bestw) {
            bestw = w, best.clear();
            for (int e = 0; e < m; e++) if (in[e]) best.push_back(e + 1);
        }
    }
    printf("%lld\n", bestw);
    for (int e : best) printf("%d\n", e);
}
