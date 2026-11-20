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
        p[u] += p[v];
        p[v] = u;
        return true;
    }
};

constexpr int dirs[4][2] = {{0, 1}, {1, 0}, {0, -1}, {-1, 0}};

int sign(int x) {
    if (x < 0) {
        return -1;
    } else if (x == 0) {
        return 0;
    } else {
        return 1;
    }
}

void solve() {
    int n;
    cin >> n;
    int arr[n][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    int m;
    cin >> m;
    int graph[n][4];
    memset(graph, -1, sizeof(graph));
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        int cx = arr[v][0] - arr[u][0], cy = arr[v][1] - arr[u][1];
        cx = sign(cx);
        cy = sign(cy);
        int cdir = -1;
        for (int j = 0; j < 4; j++) {
            if (dirs[j][0] == cx && dirs[j][1] == cy) {
                cdir = j;
                break;
            }
        }
        assert(cdir != -1);
        graph[u][cdir] = 2 * i;
        graph[v][cdir ^ 2] = (2 * i) ^ 1;
    }
    DSU dsu(2 * m);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < 4; j++) {
            if (graph[i][j] == -1) {
                continue;
            }
            for (int k = (j + 1) % 4; ; k = (k + 1) % 4) {
                if (graph[i][k] == -1) {
                    continue;
                }
                dsu.merge(graph[i][j], graph[i][k] ^ 1);
                break;
            }
        }
    }
    vector<int> adj[2 * m];
    for (int i = 0; i < 2 * m; i += 2) {
        int u = dsu.find(i), v = dsu.find(i ^ 1);
        dbg(u, v);
        adj[u].push_back(v);
        adj[v].push_back(u);
    }
    queue<int> q;
    int dist[2 * m];
    memset(dist, -1, sizeof(dist));
    auto push = [&](int u, int d) -> void {
        dbg(u, d);
        if (dist[u] == -1) {
            dist[u] = d;
            q.push(u);
        }
    };
    auto bfs = [&]() -> void {
        while (sz(q)) {
            int u = q.front();
            q.pop();
            for (auto& v : adj[u]) {
                push(v, dist[u] + 1);
            }
        }
    };
    vector<array<int, 3>> ord;
    for (int i = 0; i < n; i++) {
        auto& [x, y] = arr[i];
        ord.push_back({x, y, i});
    }
    sort(begin(ord), end(ord));
    for (auto& [_x, _y, i] : ord) {
        int ei = graph[i][1];
        if (ei == -1) {
            ei = graph[i][0];
            if (ei == -1) {
                continue;
            }
            ei ^= 1;
        }
        push(dsu.find(ei), 0);
        bfs();
    }
    vector<int> ans;
    for (int i = 0; i < 2 * m; i += 2) {
        int u = dsu.find(i), v = dsu.find(i ^ 1);
        if (dist[u] == dist[v]) {
            ans.push_back(i / 2);
        }
    }
    cout << sz(ans) << endl;
    for (auto& a : ans) {
        cout << a + 1 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}