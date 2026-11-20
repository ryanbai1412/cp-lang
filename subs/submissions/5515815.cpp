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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

struct Dijkstra {
    vector<vector<pair<int, long>>> graph;

    Dijkstra() {}

    int new_node() {
        graph.emplace_back();
        return sz(graph) - 1;
    }

    vector<int> new_node(int n) {
        vector<int> arr;
        for (int i = 0; i < n; i++) {
            arr.push_back(new_node());
        }
        return arr;
    }

    void add_edge(int u, int v, long w) {
        graph[u].emplace_back(v, w);
    }

    vector<long> query(int src) const {
        vector<long> dist(sz(graph), 1e18);
        min_pq<pair<long, int>> pq;
        auto push = [&](int u, long d) -> void {
            if (d >= dist[u]) {
                return;
            }
            dist[u] = d;
            pq.emplace(d, u);
        };
        push(src, 0);
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
        return dist;
    }
};

struct Tree {
    int n;
    vector<int> depth;
    vector<vector<int>> graph;

    Tree(int n, int root, const vector<pair<int, int>>& edges)
        : n(n), depth(n), graph(n) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        dfs(root, -1, 0);
    }

    void dfs(int u, int p, int d) {
        depth[u] = d;
        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            dfs(v, u, d + 1);
        }
    }
};

constexpr int maxn = 5e4 + 5;

// nodes - (u, depth, in, out)
// edges - (par, child)
void construct_tree(Dijkstra& graph,
                    vector<array<int, 4>> nodes,
                    vector<pair<int, int>> edges) {
    static int nmap[maxn];
    int n = sz(nodes);
    for (int i = 0; i < n; i++) {
        nmap[nodes[i][0]] = i;
    }
    vector<int> tup = graph.new_node(n), tdown = graph.new_node(n);
    for (auto& [u, v] : edges) {
        graph.add_edge(tup[nmap[v]], tup[nmap[u]], 0);
        graph.add_edge(tdown[nmap[u]], tdown[nmap[v]], 0);
    }
    for (int i = 0; i < n; i++) {
        graph.add_edge(nodes[i][2], tup[i], 0);
        graph.add_edge(tdown[i], nodes[i][3], 0);
    }
    for (int i = 0; i < n; i++) {
        graph.add_edge(nodes[i][2], tdown[i], nodes[i][1]);
        graph.add_edge(tup[i], nodes[i][3], nodes[i][1]);
    }
    map<int, vector<int>> mch;
    for (auto& [u, v] : edges) {
        mch[u].push_back(v);
    }
    for (auto& [u, ch] : mch) {
        int m = sz(ch);
        vector<int> pref = graph.new_node(m), suff = graph.new_node(m);
        for (int i = 0; i < m; i++) {
            graph.add_edge(pref[i], tdown[nmap[ch[i]]], 0);
            graph.add_edge(suff[i], tdown[nmap[ch[i]]], 0);
        }
        for (int i = 0; i < m - 1; i++) { \
            graph.add_edge(pref[i + 1], pref[i], 0);
            graph.add_edge(suff[i], suff[i + 1], 0);
        }
        for (int i = 0; i < m; i++) {
            if (i) {
                graph.add_edge(tup[nmap[ch[i]]], pref[i - 1],
                               nodes[nmap[u]][1]);
            }
            if (i + 1 < m) {
                graph.add_edge(tup[nmap[ch[i]]], suff[i + 1],
                               nodes[nmap[u]][1]);
            }
        }
    }
}

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    assert(n * kv <= int(2e5));
    int edges[m][4];
    for (auto& [u, v, w, t] : edges) {
        cin >> u >> v >> w >> t;
        u--;
        v--;
        t--;
    }
    vector<pair<int, int>> trie_edges(kv - 1);
    for (auto& [u, v] : trie_edges) {
        int _w;
        cin >> u >> v >> _w;
        u--;
        v--;
    }
    Tree trie(kv, 0, trie_edges);
    Dijkstra graph;
    int nodes[n][kv][2];
    for (auto& a : nodes) {
        for (auto& b : a) {
            for (auto& c : b) {
                c = graph.new_node();
            }
        }
    }
    for (auto& [u, v, w, t] : edges) {
        graph.add_edge(nodes[u][t][1], nodes[v][t][0], w);
    }
    for (int i = 0; i < n; i++) {
        vector<array<int, 4>> cnodes(kv);
        for (int j = 0; j < kv; j++) {
            cnodes[j] = {j, trie.depth[j], nodes[i][j][0], nodes[i][j][1]};
        }
        construct_tree(graph, cnodes, trie_edges);
    }
    int src = graph.new_node();
    for (int i = 0; i < kv; i++) {
        graph.add_edge(src, nodes[0][i][1], 0);
    }
    vector<long> dist = graph.query(src);
    long ans[n];
    memset(ans, 0x3f, sizeof(ans));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < kv; j++) {
            ans[i] = min(ans[i], dist[nodes[i][j][0]]);
        }
    }
    for (int i = 1; i < n; i++) {
        cout << ans[i] << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}