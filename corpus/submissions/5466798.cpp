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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

#ifdef DEBUG
ostream& operator<<(ostream& out, long x) {
    return out << int(x);
}
#endif

constexpr long INF = 1e30;

struct Flow {
    int n;
    vector<vector<long>> graph;

    Flow(int n) : n(n), graph(n, vector<long>(n, INF)) {}

    void add_edge(int u, int v, long w) {
        graph[u][v] = w;
    }

    long solve(int src, int sink, int kv) {
        long ans = 0;
        for (int _iter = 0; _iter < kv; _iter++) {
            long dist[n];
            int from[n];
            fill(dist, dist + n, INF);
            min_pq<pair<long, int>> pq;
            auto push = [&](int u, long d, int f) -> void {
                if (d >= dist[u]) {
                    return;
                }
                dist[u] = d;
                from[u] = f;
                pq.emplace(d, u);
            };
            push(src, 0, -1);
            while (sz(pq)) {
                auto [d, u] = pq.top();
                pq.pop();
                for (int v = 0; v < n; v++) {
                    push(v, d + graph[u][v], u);
                }
            }
            assert(dist[sink] < INF);
            ans += dist[sink];
            int u = sink;
            while (u != src) {
                int pu = from[u];
                swap(graph[pu][u], graph[u][pu]);
                graph[u][pu] *= -1;
                u = pu;
            }
        }
        return ans;
    }
};

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

long min_matching(const vector<vector<long>>& graph) {
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
        flow.add_edge(src, i, 0);
    }
    for (int i = 0; i < n; i++) {
        flow.add_edge(i + n, sink, 0);
    }
    return flow.solve(src, sink, n) + madd * n;
}

long max_matching(const vector<vector<long>>& graph) {
    int n = sz(graph);
    vector<vector<long>> ngraph(n, vector<long>(n));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            ngraph[i][j] = -graph[i][j];
        }
    }
    return -min_matching(ngraph);
}

void solve() {
    {
        Flow flow(6);
        flow.add_edge(0, 2, 3);
        flow.add_edge(0, 3, 2);
        flow.add_edge(1, 2, 1);
        flow.add_edge(1, 3, 0);
        flow.add_edge(4, 0, 0);
        flow.add_edge(4, 1, 0);
        flow.add_edge(2, 5, 0);
        flow.add_edge(3, 5, 0);
        dbg(int(flow.solve(4, 5, 2)));
    }
    dbg(int(max_matching({{5, 3}, {7, 2}})));
//    return;
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