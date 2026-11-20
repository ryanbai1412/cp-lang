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

using ld = long double;
using i128 = __int128_t;

#define endl "\n"
// #define long int64_t
#define long i128
#define sz(x) int(std::size(x))

#ifdef DEBUG
ostream& operator<<(ostream& out, long x) {
    return out << int(x);
}
#endif

struct Flow {
    int n, sink;
    long cw, ans = 0;
    vector<bool> vis;
    vector<int> st;
    vector<vector<long>> graph;

    Flow(int n) : n(n), vis(n), graph(n, vector<long>(n)) {}

    void add_edge(int u, int v, long w) {
        graph[u][v] += w;
    }

    void dfs(int u) {
        if (vis[u]) {
            return;
        }
        vis[u] = true;
        st.push_back(u);
        if (u == sink) {
            ans += cw;
            for (int i = 0; i < sz(st) - 1; i++) {
                int cu = st[i], cv = st[i + 1];
                graph[cu][cv] -= cw;
                graph[cv][cu] += cw;
            }
            st.pop_back();
            return;
        }
        for (int v = 0; v < n; v++) {
            if (graph[u][v] >= cw) {
                dfs(v);
            }
        }
        st.pop_back();
    }

    long solve(int src, int _sink) {
        sink = _sink;
        cw = 1e30;
        ans = 0;
        while (cw) {
            while (true) {
                fill(begin(vis), end(vis), false);
                dfs(src);
                if (!vis[sink]) {
                    break;
                }
            }
            cw /= 2;
        }
        return ans;
    }
};

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

long max_matching(const vector<vector<long>>& graph) {
    int n = sz(graph);
    long madd = 0;
    for (auto& a : graph) {
        for (auto& b : a) {
            madd = min(madd, b);
        }
    }
    Flow flow(2 * n + 2);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            flow.add_edge(i, j + n, graph[i][j] - madd);
        }
    }
    int src = 2 * n, sink = src + 1;
    for (int i = 0; i < n; i++) {
        flow.add_edge(src, i, 1e30);
    }
    for (int i = 0; i < n; i++) {
        flow.add_edge(i + n, sink, 1e30);
    }
    dbg(madd, n);
    return flow.solve(src, sink) + madd * n;
    //    int n = sz(graph);
//    long dp[1 << n];
//    fill(dp, dp + (1 << n), -8e18);
//    dp[(1 << n) - 1] = 0;
//    for (int i = (1 << n) - 2; i >= 0; i--) {
//        int u = __builtin_popcount(i);
//        for (int j = 0; j < n; j++) {
//            if (!on(i, j)) {
//                dp[i] = max(dp[i], graph[u][j] + dp[i | (1 << j)]);
//            }
//        }
//    }
//    return dp[0];
}

void solve() {
    {
        Flow flow(6);
        flow.add_edge(0, 2, 3);
        flow.add_edge(0, 3, 2);
        flow.add_edge(1, 2, 1);
        flow.add_edge(1, 3, 0);
        flow.add_edge(4, 0, 10);
        flow.add_edge(4, 1, 10);
        flow.add_edge(2, 5, 10);
        flow.add_edge(3, 5, 10);
        dbg(int(flow.solve(4, 5)));
    }
    dbg(int(max_matching({{-1, -2}, {-3, -4}})));
    return;
    int n;
    cin >> n;
    vector<vector<int>> arra(n, vector<int>(n)), arrb(n, vector<int>(n));
    for (auto& a : arra) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    for (auto& a : arrb) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    auto check = [&](long x) -> bool {
        vector<vector<long>> graph(n, vector<long>(n));
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                graph[i][j] = long(1e7) * arra[i][j] - x * arrb[i][j];
            }
        }
        return max_matching(graph) > 0;
    };
    long l = 0, r = 1e13;
    while (l < r) {
        long mid = (l + r + 1) / 2;
        if (check(mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    cout << ld(l) / 1e7 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(6);
    solve();
}