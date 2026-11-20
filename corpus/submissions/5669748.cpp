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

const int maxn = 3e5, logn = 19;

int t = 1, ans, vis1[maxn], vis2[maxn], tin[maxn], tout[maxn], depth[maxn],
    lift[logn][maxn];
vector<int> graph[maxn], graph1[maxn], graph2[maxn];

void dfs(int u, int p = -1) {
    lift[0][u] = p;
    tin[u] = t++;
    for (auto& v : graph[u]) {
        if (v != p) {
            depth[v] = depth[u] + 1;
            dfs(v, u);
        }
    }
    tout[u] = t - 1;
}

bool anc(int u, int v) {
    return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int lca(int u, int v) {
    if (anc(u, v)) {
        return u;
    }
    for (int i = logn - 1; i >= 0; i--) {
        if (lift[i][u] != -1 && !anc(lift[i][u], v)) {
            u = lift[i][u];
        }
    }
    return lift[0][u];
}

void dfs1(int u) {
    vis1[u] = t;
    for (auto& v : graph1[u]) {
        if (vis1[v] != t) {
            dfs1(v);
        }
    }
}

void dfs2(int u) {
    vis2[u] = t;
    ans += vis1[u] == t;
    for (auto& v : graph2[u]) {
        if (vis2[v] != t) {
            dfs2(v);
        }
    }
}

void solve() {
    int n, m, q, k;
    cin >> n >> m >> q >> k;
    if (n <= 1000) {
        auto push = [&](int u, int v) -> void {
            graph1[u].push_back(v);
            graph2[v].push_back(u);
        };
        auto pop = [&](int u, int v) -> void {
            graph1[u].pop_back();
            graph2[v].pop_back();
        };
        for (int i = 0; i < m; i++) {
            int u, v;
            cin >> u >> v;
            u--;
            v--;
            push(u, v);
        }
        while (q--) {
            t++;
            int u, v;
            cin >> u >> v;
            u--;
            v--;
            pair<int, int> extra[k];
            for (auto& [u, v] : extra) {
                cin >> u >> v;
                u--;
                v--;
                push(u, v);
            }
            ans = 0;
            dfs1(u);
            dfs2(v);
            cout << ans << endl;
            for (auto& [u, v] : extra) {
                pop(u, v);
            }
        }
    } else {
        int in[n] {};
        for (int i = 0; i < m; i++) {
            int u, v;
            cin >> u >> v;
            u--;
            v--;
            graph[u].push_back(v);
            in[v]++;
        }
        int root = -1;
        for (int i = 0; i < n; i++) {
            if (!in[i]) {
                root = i;
            }
        }
        assert(root != -1);
        depth[root] = 0;
        dfs(root);
        for (int i = 1; i < logn; i++) {
            for (int j = 0; j < maxn; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
        while (q--) {
            int u, v;
            cin >> u >> v;
            u--;
            v--;
            if (k == 0) {
                if (anc(u, v)) {
                    cout << depth[v] - depth[u] + 1 << endl;
                } else {
                    cout << 0 << endl;
                }
            } else if (k == 1) {
                int a, b;
                cin >> a >> b;
                a--;
                b--;
                auto answer = [&](int x) -> void { cout << x << endl; };
                if (anc(u, v)) {
                    if (anc(u, a) && anc(b, v)) {
                        if (anc(b, u)) {
                            int l = lca(a, v);
                            int ans = depth[a] - depth[b] + 1;
                            ans += depth[v] - depth[l];
                            answer(ans);
                        } else if (anc(u, b)) {
                            if (anc(b, a)) {
                                int l = lca(v, a);
                                int ans = depth[v] - depth[u] + 1;
                                ans += depth[a] - depth[l];
                                answer(ans);
                            } else {
                                int l = lca(a, b);
                                int ans = depth[v] - depth[u] + 1;
                                ans += depth[a] - depth[l];
                                answer(ans);
                            }
                        } else {
                            assert(false);
                        }
                    } else {
                        answer(depth[v] - depth[u] + 1);
                    }
                } else {
                    if (anc(u, a) && anc(b, v)) {
                        if (anc(b, a)) {
                            int l = lca(u, v);
                            answer(depth[a] - depth[b] + 1 + depth[v] -
                                   depth[l]);
                        } else {
                            answer(depth[a] - depth[u] + 1 + depth[v] -
                                   depth[b] + 1);
                        }
                    } else {
                        answer(0);
                    }
                }
            } else {
                cout << n << endl;
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}