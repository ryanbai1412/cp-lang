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

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(const Cb& cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

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

vector<array<int, 3>> mst(int n, const vector<array<int, 3>>& arr) {
    vector<array<int, 3>> out;
    DSU dsu(n);
    for (auto& [u, v, w] : arr) {
        if (dsu.merge(u, v)) {
            out.push_back({u, v, w});
        }
    }
    return out;
}

vector<vector<int>> fw_mn(vector<vector<int>> graph) {
    int n = sz(graph);
    for (int i = 0; i < n; i++) {
        graph[i][i] = 1e9;
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            for (int k = 0; k < n; k++) {
                graph[j][k] = max(graph[j][k], min(graph[j][i], graph[i][k]));
            }
        }
    }
    return graph;
}

vector<vector<int>> fw_mx(vector<vector<int>> graph) {
    int n = sz(graph);
    for (int i = 0; i < n; i++) {
        graph[i][i] = -1;
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            for (int k = 0; k < n; k++) {
                graph[j][k] = min(graph[j][k], max(graph[j][i], graph[i][k]));
            }
        }
    }
    return graph;
}

void solve() {
    int n, kv;
    cin >> n >> kv;
    vector<vector<int>> arr_mn, arr_mx;
    auto read = [&](vector<vector<int>>& arr) -> void {
        arr.resize(n, vector<int>(n));
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < i; j++) {
                cin >> arr[i][j];
                arr[j][i] = arr[i][j];
            }
        }
    };
    read(arr_mx);
    read(arr_mn);

    for (auto& a : arr_mx) {
        for (auto& b : a) {
            b = kv - b;
        }
    }

    for (int i = 0; i < n; i++) {
        arr_mn[i][i] = 1e9;
        arr_mx[i][i] = -1;
    }

    if (fw_mn(arr_mn) != arr_mn || fw_mx(arr_mx) != arr_mx) {
        cout << "NO" << endl;
        return;
    }

    vector<array<int, 3>> edges;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < i; j++) {
            if (arr_mn[i][j] >= arr_mx[i][j]) {
                edges.push_back({i, j, arr_mn[i][j]});
                edges.push_back({i, j, arr_mx[i][j]});
            }
        }
    }

    vector<vector<int>> cg_mn(n, vector<int>(n, -1)),
        cg_mx(n, vector<int>(n, 1e9));
    for (auto& [u, v, w] : edges) {
        cg_mn[u][v] = max(cg_mn[u][v], w);
        cg_mx[u][v] = min(cg_mx[u][v], w);
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < i; j++) {
            cg_mn[j][i] = cg_mn[i][j];
            cg_mx[j][i] = cg_mx[i][j];
        }
    }

    sort(begin(edges), end(edges),
         CmpByKey([&](const auto& a) -> int { return a[2]; }));
    auto e1 = mst(n, edges);
    reverse(begin(edges), end(edges));
    auto e2 = mst(n, edges);

    edges = e1;
    edges.insert(edges.end(), begin(e2), end(e2));

    if (fw_mn(cg_mn) != arr_mn || fw_mx(cg_mx) != arr_mx) {
        cout << "NO" << endl;
        return;
    }

    cout << sz(edges) << endl;
    for (auto& [u, v, w] : edges) {
        cout << u << " " << v << " " << w << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}