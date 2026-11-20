#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

namespace s1 {

int count_hamil(const vector<vector<bool>>& graph) {
    int n = sz(graph);

    int dp[1 << n][n] {};
    for (int i = 0; i < n; i++) {
        dp[(1 << n) - 1][i] = graph[i][0];
    }

    for (int mask = (1 << n) - 2; mask >= 0; mask--) {
        for (int u = 0; u < n; u++) {
            for (int v = 0; v < n; v++) {
                if (on(mask, u) && !on(mask, v) && graph[u][v]) {
                    dp[mask][u] += dp[mask | (1 << v)][v];
                }
            }
        }
    }

    return dp[1][0] / 2;
}

bool solve(const vector<pair<int, int>>& edges, int kv) {
    int n = sz(edges) + 1;

    vector<vector<int>> graph(n);
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
    }

    vector<int> leaves;
    {
        vector<int> q;
        q.push_back(0);

        while (sz(q)) {
            int u = q.back();
            q.pop_back();

            if (!sz(graph[u])) {
                leaves.push_back(u);
            } else {
                for (auto& v : graph[u]) {
                    q.push_back(v);
                }
            }
        }
    }

    vector ngraph(n, vector(n, false));
    for (auto& [u, v] : edges) {
        ngraph[u][v] = ngraph[v][u] = true;
    }
    int m = sz(leaves);
    for (int i = 0; i < m; i++) {
        for (int j = 1; j <= kv; j++) {
            int u = leaves[i], v = leaves[(i + j) % m];
            ngraph[u][v] = ngraph[v][u] = true;
        }
    }

    cout << count_hamil(ngraph) << endl;
    return true;
}

}  // namespace s1

void solve() {
    int n, kv;
    cin >> n >> kv;

    vector<pair<int, int>> edges;
    for (int i = 1; i < n; i++) {
        int p;
        cin >> p;
        p--;
        edges.emplace_back(p, i);
    }

    s1::solve(edges, kv);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}