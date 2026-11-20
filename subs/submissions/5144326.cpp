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

namespace kactl {
#define rep(i, a, b) for (int i = a; i < (b); ++i)
#define all(x) begin(x), end(x)
using ll = long;
using pii = pair<int, int>;
using vi = vector<int>;

struct Dinic {
    struct Edge {
        int to, rev;
        ll c, oc;
        ll flow() { return max(oc - c, ll(0)); } // if you need flows
    };
    vi lvl, ptr, q;
    vector<vector<Edge>> adj;
    Dinic(int n) : lvl(n), ptr(n), q(n), adj(n) {}
    void addEdge(int a, int b, ll c, ll rcap = 0) {
        adj[a].push_back({b, sz(adj[b]), c, c});
        adj[b].push_back({a, sz(adj[a]) - 1, rcap, rcap});
    }
    ll dfs(int v, int t, ll f) {
        if (v == t || !f) return f;
        for (int& i = ptr[v]; i < sz(adj[v]); i++) {
            Edge& e = adj[v][i];
            if (lvl[e.to] == lvl[v] + 1)
                if (ll p = dfs(e.to, t, min(f, e.c))) {
                    e.c -= p, adj[e.to][e.rev].c += p;
                    return p;
                }
        }
        return 0;
    }
    ll calc(int s, int t) {
        ll flow = 0; q[0] = s;
        rep(L,0,31) do { // 'int L=30' maybe faster for random data
            lvl = ptr = vi(sz(q));
            int qi = 0, qe = lvl[s] = 1;
            while (qi < qe && !lvl[t]) {
                int v = q[qi++];
                for (Edge e : adj[v])
                    if (!lvl[e.to] && e.c >> (30 - L))
                        q[qe++] = e.to, lvl[e.to] = lvl[v] + 1;
            }
            while (ll p = dfs(s, t, LLONG_MAX)) flow += p;
        } while (lvl[t]);
        return flow;
    }
    bool leftOfMinCut(int a) { return lvl[a] != 0; }
};

#undef rep
#undef all
}  // namespace kactl

constexpr int maxn = 605;

int arr[maxn], dfs_vis[maxn];
bool cyc;
vector<int> graph[maxn];

void dfs(int u) {
    if (dfs_vis[u]) {
        if (dfs_vis[u] == 1) {
            cyc = true;
        }
        return;
    }
    dfs_vis[u] = 1;
    for (auto& v : graph[u]) {
        dfs(v);
    }
    dfs_vis[u] = 2;
}

void solve() {
    int n, m;
    cin >> n >> m;
    auto id = [&](int x, int y) -> int {
        return x * m + y;
    };
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            if (j + 1 < m) {
                graph[id(i, j)].push_back(id(i, j + 1));
            }
            int kv;
            cin >> arr[id(i, j)] >> kv;
            for (int k = 0; k < kv; k++) {
                int x, y;
                cin >> x >> y;
                graph[id(x, y)].push_back(id(i, j));
            }
        }
    }
    n *= m;
    for (int i = 0; i < n; i++) {
        for (auto& a : graph[i]) {
            dbg(i, a);
        }
    }
    bool bad[n] {}, cvis[n][n];
    for (int i = 0; i < n; i++) {
        memset(dfs_vis, 0, sizeof(dfs_vis));
        cyc = false;
        dfs(i);
        if (cyc) {
            dbg("CYC", i);
            bad[i] = true;
            continue;
        }
        copy(dfs_vis, dfs_vis + n, cvis[i]);
    }
    kactl::Dinic flow(n * m + 2);
    long csum = 0;
    int source = n * m, sink = n * m + 1;
    for (int i = 0; i < n; i++) {
        if (bad[i]) {
            continue;
        }
        if (arr[i] < 0) {
            flow.addEdge(i, sink, -arr[i]);
        } else {
            csum += arr[i];
            flow.addEdge(source, i, arr[i]);
        }
        for (auto& v : graph[i]) {
            flow.addEdge(i, v, 1e9);
        }
    }
    cout << csum - flow.calc(source, sink) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}