// Highest-label push-relabel with the gap heuristic, the same steps as
// FlowGraph.push_relabel: arcs in insertion order, arc a's reverse is a ^ 1.
vector<vector<int>> head;
vector<int> to; vector<long long> lft, cap0;
void add(int u, int v, long long c) {
    head[u].push_back(to.size()); to.push_back(v); lft.push_back(c); cap0.push_back(c);
    head[v].push_back(to.size()); to.push_back(u); lft.push_back(0); cap0.push_back(0);
}
long long push_relabel(int s, int t) {
    int n = head.size();
    vector<long long> h(n, 0), ex(n, 0), nx(n, 0);
    vector<vector<int>> active(2 * n); vector<long long> cnt(2 * n, 0);
    auto push = [&](int a, long long f) {
        int v = to[a];
        if (ex[v] == 0 && f > 0) active[h[v]].push_back(v);
        lft[a] -= f; lft[a ^ 1] += f; ex[v] += f; ex[to[a ^ 1]] -= f;
    };
    h[s] = n; ex[t] = 1; cnt[0] = n - 1;
    for (int a : head[s]) push(a, lft[a]);
    for (long long top = 0;;) {
        while (active[top].empty()) { if (top == 0) return -ex[s]; top--; }
        int u = active[top].back(); active[top].pop_back();
        while (ex[u] > 0) {
            if (nx[u] == (long long)head[u].size()) {
                h[u] = LLONG_MAX;
                for (size_t k = 0; k < head[u].size(); k++) { int a = head[u][k];
                    if (lft[a] > 0 && h[to[a]] < h[u] - 1) { h[u] = h[to[a]] + 1; nx[u] = k; } }
                cnt[h[u]]++; cnt[top]--;
                if (cnt[top] == 0 && top < n)
                    for (int v = 0; v < n; v++) if (top < h[v] && h[v] < n) { cnt[h[v]]--; h[v] = n + 1; }
                top = h[u];
            } else { int a = head[u][nx[u]];
                if (lft[a] > 0 && h[u] == h[to[a]] + 1) push(a, min(ex[u], lft[a])); else nx[u]++; }
        }
    }
}
