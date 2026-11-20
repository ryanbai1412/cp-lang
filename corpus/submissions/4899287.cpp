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
    
    DSU(int n): p(n, -1) {}
    
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
        p[v] += p[u];
        p[u] = v;
        return true;
    }
};

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    array<int, 3> edges[m];
    for (auto& [w, u, v] : edges) {
        cin >> u >> v >> w;
        u--;
        v--;
    }
    auto mst = [&](const vector<array<int, 3>>& sedges) {
        DSU dsu(n);
        long ans = 0;
        int mx = 0;
        for (auto& [w, u, v] : sedges) {
            if (dsu.merge(u, v)) {
                ans += w;
                mx = max(mx, w);
            }
        }
        return ans - min(mx, kv);
    };
    long mans;
    {
        vector<array<int, 3>> sedges(edges, edges + m);
        sort(begin(sedges), end(sedges));
        mans = mst(sedges);
    }
    dbg(mans);
    auto check = [&](int x) -> bool {
        x = n - 1 - x;
        dbg(x);
        vector<array<int, 3>> sedges(edges, edges + m);
        sort(sedges.begin(), sedges.begin() + n - 1);
        sort(sedges.begin() + x, sedges.end());
        return mans == mst(sedges);
    };
    int l = 0, r = n - 1;
    while (l < r) {
        int mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    cout << l << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}