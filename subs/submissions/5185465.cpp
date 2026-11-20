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

constexpr int maxn = 305;

vector<pair<int, int>> graph[maxn];

struct Farthest {
    pair<int, int> ans {-1, -1};

    void dfs(int u, int p, int d) {
        ans = max(ans, {d, u});
        for (auto& [v, w] : graph[u]) {
            if (v == p) {
                continue;
            }
            dfs(v, u, d + w);
        }
    }

    static pair<int, int> solve(int u) {
        Farthest solver;
        solver.dfs(u, -1, 0);
        return solver.ans;
    }
};

int diameter() {
    return Farthest::solve(Farthest::solve(0).second).first;
}

struct DS1 {
    vector<int> v;

    void insert(int x) {
        v.push_back(x);
    }

    void build() {
        v.push_back(0);
        sort(begin(v), end(v));
    }

    int query_max_excluding(int x) const {
        return x == v.back() ? v[sz(v) - 2] : v.back();
    }
};

struct Solver {
    bool valid[maxn] {};
    int kv, diam, far[maxn] {}, par[maxn] {}, ans = 1e9;

    Solver(int kv, int diam): kv(kv), diam(diam) {}

    void pdfs(int u, int p, int d) {
        par[u] = p;
        valid[u] = d == diam;
        for (auto& [v, w] : graph[u]) {
            if (v == p) {
                continue;
            }
            pdfs(v, u, d + w);
            valid[u] |= valid[v];
            far[u] = max(far[u], w + far[v]);
        }
    }

    void dfs2(int u, int cans, int clen) {
        if (clen > kv || !valid[u]) {
            return;
        }
        dbg(u, cans, far[u], clen);
        ans = min(ans, max(cans, far[u]));
        DS1 ds;
        for (auto& [v, w] : graph[u]) {
            if (v == par[u]) {
                continue;
            }
            ds.insert(w + far[v]);
        }
        ds.build();
        for (auto& [v, w] : graph[u]) {
            if (v == par[u]) {
                continue;
            }
            dfs2(v, max(cans, ds.query_max_excluding(w + far[v])), clen + w);
        }
    }

    void dfs1(int u, int cans) {
        if (!valid[u]) {
            return;
        }
        dfs2(u, cans, 0);
        DS1 ds;
        ds.insert(cans);
        for (auto& [v, w] : graph[u]) {
            if (v == par[u]) {
                continue;
            }
            ds.insert(w + far[v]);
        }
        ds.build();
        for (auto& [v, w] : graph[u]) {
            if (v == par[u]) {
                continue;
            }
            dfs1(v, w + ds.query_max_excluding(w + far[v]));
        }
    }

    static int solve(int kv, int diam, int u) {
        Solver solver(kv, diam);
        solver.pdfs(u, -1, 0);
        solver.dfs1(u, 0);
        return solver.ans;
    }
};

void solve() {
    int n, kv;
    cin >> n >> kv;
    for (int i = 0; i < n - 1; i++) {
        int u, v, w;
        cin >> u >> v >> w;
        u--;
        v--;
        dbg(u, v, w);
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    int diam = diameter(), ans = 1e9;
    dbg(diam, Solver::solve(kv, diam, 7));
//    return;
    for (int i = 0; i < n; i++) {
        ans = min(ans, Solver::solve(kv, diam, i));
        dbg(ans);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}