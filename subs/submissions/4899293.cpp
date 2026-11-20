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
        p[v] += p[u];
        p[u] = v;
        return true;
    }
};

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    int edges[m][3];
    for (auto& [u, v, w] : edges) {
        cin >> u >> v >> w;
        u--;
        v--;
    }
    pair<long, int> ans {long(1e18), -1};
    int perm[m] {};
    fill(perm, perm + n - 1, 1);
    reverse(perm, perm + m);
    do {
        DSU dsu(n);
        long cans = 0;
        int mx = 0, bcnt = 0;
        for (int i = 0; i < m; i++) {
            auto& [u, v, w] = edges[i];
            if (perm[i]) {
                if (!dsu.merge(u, v)) {
                    goto loop;
                }
                cans += w;
                mx = max(mx, w);
                bcnt += i >= n - 1;
            }
        }
        ans = min(ans, make_pair(cans - min(kv, mx), bcnt));
    loop:;
    } while (next_permutation(perm, perm + m));
    dbg(ans.first);
    cout << ans.second << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}