#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

using P = pair<int, int>;

P& operator+=(P& a, const P& b) {
    a.first += b.first;
    a.second += b.second;
    return a;
}

P operator+(P a, const P& b) {
    return a += b;
}

constexpr P DIRS[8] {{0, -1}, {1, -1}, {1, 0},  {1, 1},
                     {0, 1},  {-1, 1}, {-1, 0}, {-1, -1}};

long dijkstra(const vector<tuple<int, int, long>>& edges,
              const vector<int>& srcs,
              const vector<int>& dests) {
    int n = 0;
    for (auto& [u, v, w] : edges) {
        n = max({n, u, v});
    }
    n++;

    vector<pair<int, long>> graph[n];
    for (auto& [u, v, w] : edges) {
        graph[u].emplace_back(v, w);
    }

    long dist[n];
    memset(dist, 0x3f, sizeof(dist));
    min_pq<pair<long, int>> pq;

    auto push = [&](int u, long d) -> void {
        if (d >= dist[u]) {
            return;
        }
        dist[u] = d;
        pq.emplace(d, u);
    };

    for (auto& a : srcs) {
        push(a, 0);
    }

    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        if (dist[u] != d) {
            continue;
        }

        for (auto& [v, w] : graph[u]) {
            push(v, d + w);
        }
    }

    long ans = 1e18;
    for (auto& a : dests) {
        ans = min(ans, dist[a]);
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    n++;

    int arr[n][n][4] {};

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n - 1; j++) {
            cin >> arr[i][j][2];
        }
    }
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n; j++) {
            cin >> arr[i][j][1];
        }
    }
    for (int i = 0; i < n; i++) {
        for (int j = 1; j < n; j++) {
            cin >> arr[i][j][0];
        }
    }
    for (int i = 1; i < n; i++) {
        for (int j = 0; j < n; j++) {
            cin >> arr[i][j][3];
        }
    }

    struct State {
        P pos;
        int dir;
    };

    vector<tuple<State, State, long>> edges;

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            P p {i, j};

            for (int d = 0; d < 8; d++) {
                long cadd = 0;

                for (int dd = 1; dd < 7; dd++) {
                    int cd = (d + dd) % 8;

                    if (dd > 1) {
                        edges.emplace_back(State {p, d},
                                           State {p + DIRS[cd], cd ^ 4}, cadd);
                    }

                    if (cd % 2 == 0) {
                        cadd += arr[i][j][cd >> 1];
                    }
                }
            }
        }
    }

    {
        vector<tuple<State, State, long>> n_edges;

        P sink {n - 1, n - 1};

        for (auto& [u, v, w] : edges) {
            if (u.pos == sink || v.pos == sink) {
                continue;
            }

            n_edges.emplace_back(u, v, w);
        }

        edges = n_edges;
    }

    vector<State> srcs, sinks;
    for (int i = 0; i < n; i++) {
        srcs.push_back({{i, 0}, 0});
        srcs.push_back({{n - 1, i}, 2});
        sinks.push_back({{-1, i}, 2});
        sinks.push_back({{i, n}, 0});
    }

    auto id = [&](const State& s) -> int {
        return ((s.pos.first + 1) * (n + 1) + (s.pos.second + 1)) * 8 + s.dir;
    };

    vector<tuple<int, int, long>> c_edges;
    for (auto& [u, v, w] : edges) {
        c_edges.emplace_back(id(u), id(v), w);
    }

    vector<int> c_srcs, c_sinks;
    for (auto& a : srcs) {
        c_srcs.push_back(id(a));
    }
    for (auto& a : sinks) {
        c_sinks.push_back(id(a));
    }

    cout << dijkstra(c_edges, c_srcs, c_sinks) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}