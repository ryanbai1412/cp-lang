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

struct Solver {
    int n;
    bool ans = true;
    vector<int> vis;
    vector<vector<int>> graph;

    Solver(int n) : n(n), graph(n), vis(n, -1) {}

    void add_edge(int u, int v) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    void dfs(int u, int set) {
        if (vis[u] != -1) {
            ans &= vis[u] == set;
            return;
        }
        vis[u] = set;
        for (auto& v : graph[u]) {
            dfs(v, !set);
        }
    }

    bool solve() {
        for (int i = 0; i < n; i++) {
            if (vis[i] == -1) {
                dfs(i, 0);
            }
        }
        return ans;
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    int edges[m][3];
    for (auto& [u, v, w] : edges) {
        cin >> u >> v >> w;
        u--;
        v--;
    }
    auto check = [&](int x) -> bool {
        Solver solve(n);
        for (auto& [u, v, w]: edges) {
            if (w > x) {
                solve.add_edge(u, v);
            }
        }
        return solve.solve();
    };
    int l = 0, r = 1e9;
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