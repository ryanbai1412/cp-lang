// NOI 2006 Tour Guide reference (output-only; longest simple path, scored against the best
// known). Exact for n <= 20 (DP over visited sets) and for forests (a diameter per tree);
// otherwise the longest of many deterministic randomized Warnsdorff walks grown from both
// ends with Posa rotations at the tail. checker.py accepts any valid path at least as long.
#include <bits/stdc++.h>
using namespace std;

int n, m;
vector<vector<int>> adj;
vector<int> best;

void exact() {
    vector<vector<int>> dp(1 << n, vector<int>(n, -1));
    for (int v = 0; v < n; v++) dp[1 << v][v] = v;
    int bm = 1, bv = 0;
    for (int s = 1; s < (1 << n); s++)
        for (int v = 0; v < n; v++) {
            if (dp[s][v] < 0) continue;
            if (__builtin_popcount(s) > __builtin_popcount(bm)) bm = s, bv = v;
            for (int w : adj[v])
                if (!(s >> w & 1) && dp[s | 1 << w][w] < 0) dp[s | 1 << w][w] = v;
        }
    best.clear();
    for (int s = bm, v = bv;;) {
        best.push_back(v);
        int p = dp[s][v];
        if (p == v) break;
        s ^= 1 << v, v = p;
    }
}

bool forest() {
    vector<int> comp(n, -1);
    int edges = 0;
    for (int v = 0; v < n; v++) edges += adj[v].size();
    edges /= 2;
    int comps = 0;
    for (int s = 0; s < n; s++) {
        if (comp[s] >= 0) continue;
        vector<int> st{s};
        comp[s] = s;
        while (!st.empty()) {
            int v = st.back();
            st.pop_back();
            for (int w : adj[v])
                if (comp[w] < 0) comp[w] = s, st.push_back(w);
        }
        comps++;
    }
    if (edges != n - comps) return false;
    auto far = [&](int s, vector<int>& par) {
        vector<int> dist(n, -1), q{s};
        dist[s] = 0, par[s] = -1;
        int last = s;
        for (size_t i = 0; i < q.size(); i++) {
            int v = q[i];
            last = v;
            for (int w : adj[v])
                if (dist[w] < 0) dist[w] = dist[v] + 1, par[w] = v, q.push_back(w);
        }
        return last;
    };
    vector<int> par(n);
    for (int s = 0; s < n; s++) {
        if (comp[s] != s) continue;
        int a = far(s, par), b = far(a, par);
        vector<int> path;
        for (int v = b; v >= 0; v = par[v]) path.push_back(v);
        if (path.size() > best.size()) best = path;
    }
    return true;
}

void heuristic() {
    mt19937 rng(2006);
    vector<int> on(n, 0), pos(n, -1);
    int stamp = 0;
    long long budget = 60'000'000;
    while (budget > 0) {
        stamp++;
        int s = rng() % n;
        vector<int> path{s};
        on[s] = stamp;
        pos[s] = 0;
        auto freeDeg = [&](int v) {
            int c = 0;
            for (int w : adj[v]) c += on[w] != stamp;
            budget -= adj[v].size();
            return c;
        };
        bool wild = rng() % 2;
        auto pick = [&](int v) {
            int bw = -1, bd = INT_MAX, ties = 0;
            if (wild && rng() % 4 == 0) {
                for (int w : adj[v])
                    if (on[w] != stamp && rng() % ++ties == 0) bw = w;
                return bw;
            }
            for (int w : adj[v]) {
                if (on[w] == stamp) continue;
                int d = freeDeg(w);
                if (d < bd) bd = d, bw = w, ties = 1;
                else if (d == bd && rng() % ++ties == 0) bw = w;
            }
            return bw;
        };
        for (int round = 0; round < 4; round++) {
            int rotations = 0;
            while (true) {
                int v = path.back();
                int w = pick(v);
                if (w >= 0) {
                    on[w] = stamp, pos[w] = path.size();
                    path.push_back(w);
                    continue;
                }
                if (rotations++ > 2 * n || path.size() < 3) break;
                // Posa rotation: tail v links to path[i]; reverse path[i+1..] to get a new tail.
                vector<int> cand;
                for (int x : adj[v])
                    if (on[x] == stamp && pos[x] + 2 < (int)path.size()) cand.push_back(pos[x]);
                budget -= path.size();
                if (cand.empty()) break;
                int i = cand[rng() % cand.size()];
                reverse(path.begin() + i + 1, path.end());
                for (size_t j = i + 1; j < path.size(); j++) pos[path[j]] = j;
            }
            reverse(path.begin(), path.end());
            for (size_t j = 0; j < path.size(); j++) pos[path[j]] = j;
        }
        if (path.size() > best.size()) best.assign(path.begin(), path.end());
        if (best.size() == (size_t)n) break;
    }
}

int main() {
    int t;
    cin >> t >> n >> m;
    adj.assign(n, {});
    for (int i = 0; i < m; i++) {
        int a, b;
        cin >> a >> b;
        if (a == b) continue;
        adj[a - 1].push_back(b - 1), adj[b - 1].push_back(a - 1);
    }
    for (auto& a : adj) sort(a.begin(), a.end()), a.erase(unique(a.begin(), a.end()), a.end());
    if (n <= 20) exact();
    else if (!forest()) heuristic();
    printf("%zu\n", best.size());
    for (int v : best) printf("%d\n", v + 1);
}
