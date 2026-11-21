struct Dinic {
    int n; vector<vector<int>> head; vector<int> to; vector<long long> lft, cap0;
    Dinic(int n) : n(n), head(n) {}
    int add(int u, int v, long long c) { int a = to.size(); head[u].push_back(a); head[v].push_back(a ^ 1);
        to.push_back(v); to.push_back(u); lft.push_back(c); lft.push_back(0); cap0.push_back(c); cap0.push_back(0); return a; }
    int add_both(int u, int v, long long c) { int a = add(u, v, c); lft[a ^ 1] = c; cap0[a ^ 1] = c; return a; }
    long long flow(int a) { return cap0[a] - lft[a]; }
    void reset() { lft = cap0; }
    void close(int a) { lft[a] = 0; lft[a ^ 1] = 0; }
    vector<int> levels(int s) { vector<int> level(n, -1); level[s] = 0; vector<int> q{s};
        for (size_t at = 0; at < q.size(); at++) { int u = q[at];
            for (int a : head[u]) if (lft[a] > 0 && level[to[a]] < 0) { level[to[a]] = level[u] + 1; q.push_back(to[a]); } }
        return level; }
    long long augment(int s, int t, const vector<int>& level, vector<int>& next, long long most) {
        vector<int> path; int u = s;
        while (u != t) {
            if (next[u] < (int)head[u].size()) { int a = head[u][next[u]]; int v = to[a];
                if (lft[a] > 0 && level[v] == level[u] + 1) { path.push_back(a); u = v; } else next[u]++; }
            else { if (path.empty()) return 0; int a = path.back(); path.pop_back(); u = to[a ^ 1]; next[u]++; }
        }
        long long amt = most; for (int a : path) amt = min(amt, lft[a]);
        for (int a : path) { lft[a] -= amt; lft[a ^ 1] += amt; }
        return amt; }
    long long max_flow(int s, int t, long long limit = LLONG_MAX) { long long total = 0;
        while (total < limit) { auto level = levels(s); if (level[t] < 0) break; vector<int> next(n, 0);
            while (total < limit) { long long p = augment(s, t, level, next, limit - total); if (!p) break; total += p; } }
        return total; }
    vector<bool> min_cut(int s) { auto level = levels(s); vector<bool> r(n); for (int i = 0; i < n; i++) r[i] = level[i] >= 0; return r; }
};
