#include <algorithm>
#include <cstdint>
#include <iostream>
#include <utility>
#include <vector>
using namespace std;
using ll = long long;

struct RollbackDsu {
    vector<int> p, sz;
    vector<pair<int, int>> hist;
    int comps;
    RollbackDsu(int n) : p(n), sz(n, 1), comps(n) {
        for (int i = 0; i < n; i++) p[i] = i;
    }
    int find(int x) const {
        while (p[x] != x) x = p[x];
        return x;
    }
    int snapshot() const { return (int)hist.size(); }
    void unite(int a, int b) {
        a = find(a); b = find(b);
        if (a == b) return;
        if (sz[a] < sz[b]) swap(a, b);
        hist.push_back({b, sz[a]});
        p[b] = a;
        sz[a] += sz[b];
        comps--;
    }
    void rollback(int snap) {
        while ((int)hist.size() > snap) {
            auto [b, old_sz] = hist.back();
            hist.pop_back();
            int a = p[b];
            p[b] = b;
            sz[a] = old_sz;
            comps++;
        }
    }
};

struct TimeTree {
    int size, q;
    vector<vector<pair<int, int>>> tree;
    TimeTree(int q_) : q(q_) {
        size = 1;
        while (size < max(1, q)) size <<= 1;
        tree.resize(2 * size);
    }
    void add_interval(int l, int r, pair<int, int> e) {
        for (l += size, r += size; l < r; l >>= 1, r >>= 1) {
            if (l & 1) tree[l++].push_back(e);
            if (r & 1) tree[--r].push_back(e);
        }
    }
    void dfs(int u, int l, int r, RollbackDsu& dsu, vector<int>& ans) const {
        int snap = dsu.snapshot();
        for (auto [a, b] : tree[u]) dsu.unite(a, b);
        if (r - l == 1) {
            if (l < q) ans[l] = dsu.comps;
        } else {
            int mid = (l + r) / 2;
            dfs(2 * u, l, mid, dsu, ans);
            if (mid < q) dfs(2 * u + 1, mid, r, dsu, ans);
        }
        dsu.rollback(snap);
    }
};

int main() {
    const int n = 30000, m = 60000, q = 30000;
    ll seed = 123456789;
    auto next_rng = [&]() -> ll {
        seed = (seed * 1103515245LL + 12345LL) % 2147483648LL;
        return seed;
    };
    TimeTree tt(q);
    for (int i = 0; i < m; i++) {
        int a = (int)(next_rng() % q), b = (int)(next_rng() % q);
        int lo = min(a, b), hi = max(a, b) + 1;
        int u = (int)(next_rng() % n), v = (int)(next_rng() % n);
        tt.add_interval(lo, hi, {u, v});
    }
    RollbackDsu dsu(n);
    vector<int> ans(q);
    tt.dfs(1, 0, tt.size, dsu, ans);
    ll acc = 0;
    for (int x : ans) acc += x;
    cout << acc << "\n";
}
