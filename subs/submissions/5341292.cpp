#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct DSU {
    vector<int> p;

    DSU(int n) : p(n, -1) {}

    int find(int u) {
        return p[u] < 0 ? u : (p[u] = find(p[u]));
    }

    bool merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            return false;
        }
        if (p[u] < p[v]) {
            swap(u, v);
        }
        p[u] += p[v];
        p[v] = u;
        return true;
    }
};

int mst(int n, vector<array<int, 3>> edges) {
    DSU dsu(n);
    sort(begin(edges), end(edges), [&](auto a, auto b) -> bool {
        return a[2] < b[2];
    });
    int ans = 0;
    for (auto& [u, v, w] : edges) {
        if (dsu.merge(u, v)) {
            ans += w;
        }
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    map<pair<int, int>, vector<pair<int, int>>> mp;
    auto add_edge = [&](int u, int v, int i, int w) -> void {
        if (u > v) {
            swap(u, v);
        }
        mp[{u, v}].emplace_back(i, w);
    };
    for (int i = 0; i < n; i++) {
        int m;
        cin >> m;
        int vs[m], ws[m];
        for (auto& a : vs) {
            cin >> a;
            a--;
        }
        for (auto& a : ws) {
            cin >> a;
        }
        for (int j = 0; j < m; j++) {
            add_edge(vs[j], vs[(j + 1) % m], i, ws[j]);
        }
    }
    vector<array<int, 3>> edges;
    for (auto& [_k, v] : mp) {
        assert(sz(v) == 1 || sz(v) == 2);
        if (sz(v) == 2) {
            auto [u1, w1] = v[0];
            auto [u2, w2] = v[1];
            assert(w1 == w2);
            edges.push_back({u1, u2, w1});
        }
    }
    int ans = mst(n, edges);
    for (auto& [_k, v] : mp) {
        if (sz(v) == 1) {
            auto [u, w] = v[0];
            edges.push_back({n, u, w});
        }
    }
    ans = min(ans, mst(n + 1, edges));
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}