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

struct Node {
    int l, r, v;

    Node operator+(const Node& n) const {
        return {l, n.r, v + n.v + (r != n.l)};
    }

    Node reversed() const {
        return {r, l, v};
    }

    static Node c_from(int x) {
        return {x, x, 0};
    }

    static Node merge(const optional<Node>& a, const Node& b) {
        if (!a) {
            return b;
        }
        return a.value() + b;
    }

    static Node merge(const Node& a, const optional<Node>& b) {
        if (!b) {
            return a;
        }
        return a + b.value();
    }
};

struct ST1 {
    vector<Node> arr;

    ST1() {}
    ST1(const vector<Node>& arr) : arr(arr) {}

    void update(int l, int r, int x) {
        for (int i = l; i <= r; i++) {
            arr[i] = Node::c_from(x);
        }
    }

    Node query(int l, int r) const {
        Node ans = arr[l];
        for (int i = l + 1; i <= r; i++) {
            ans = ans + arr[i];
        }
        return ans;
    }
};

struct ST2 {
    vector<int> arr;

    ST2() {}
    ST2(const vector<int>& arr) : arr(arr) {}

    void update(int l, int r, int x) {
        for (int i = l; i <= r; i++) {
            arr[i] += x;
        }
    }

    int query(int l, int r) const {
        int ans = 1e9;
        for (int i = l; i <= r; i++) {
            ans = min(ans, arr[i]);
        }
        return ans;
    }
};

constexpr int maxn = 1e5 + 5, logn = 20;

int cc, tin[maxn], tout[maxn], par[maxn], siz[maxn], up[maxn], lift[logn][maxn],
    depth[maxn];
vector<int> graph[maxn], ord;
ST1 st1;
ST2 st2;

void dfs1(int u, int p, int d) {
    par[u] = lift[0][u] = p;
    tin[u] = cc++;
    siz[u] = 1;
    depth[u] = d;
    ord.push_back(u);
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        dfs1(v, u, d + 1);
        siz[u] += siz[v];
        if (siz[v] > siz[graph[u][0]]) {
            swap(v, graph[u][0]);
        }
    }
    tout[u] = cc;
}

void dfs2(int u) {
    if (sz(graph[u])) {
        up[graph[u][0]] = up[u];
    }
    for (auto& v : graph[u]) {
        dfs2(v);
    }
}

bool anc(int u, int v) {
    return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int lca(int u, int v) {
    if (anc(u, v)) {
        return u;
    }
    for (int i = logn - 1; i >= 0; i--) {
        int cu = lift[i][u];
        if (cu != -1 && !anc(cu, v)) {
            u = cu;
        }
    }
    return par[u];
}

Node query_anc(int u, int v) {
    assert(anc(u, v));
    optional<Node> ans;
    while (true) {
        int lv = max(tin[u], tin[up[v]]);
        ans = Node::merge(st1.query(lv, tin[v]), ans);
        if (lv == tin[u]) {
            break;
        }
        v = par[up[lv]];
    }
    return ans.value();
}

void update_st1(int u, int ind) {
    while (u >= 0) {
        int lu = tin[up[u]];
        st1.update(lu, u, ind);
        u = par[up[u]];
    }
}

void update(int u, int ind) {
    update_st1(u, ind);
}

void solve() {
    int n, q;
    cin >> n >> q;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    dfs1(0, -1, 0);
    iota(up, up + n, 0);
    dfs2(0);
    for (int i = 1; i < logn; i++) {
        for (int j = 0; j < n; j++) {
            if (lift[i - 1][j] == -1) {
                lift[i][j] = -1;
            } else {
                lift[i][j] = lift[i - 1][lift[i - 1][j]];
            }
        }
    }
    int cind = 0;
    {
        vector<Node> stv1;
        for (int i = 0; i < n; i++) {
            stv1.push_back(Node::c_from(cind++));
        }
        st1 = stv1;
    }
    {
        vector<int> stv2(n);
        for (int i = 0; i < n; i++) {
            stv2[tin[i]] = depth[i] + 1;
        }
        st2 = stv2;
    }
    while (q--) {
        int t;
        cin >> t;
        dbg(t);
        if (t == 1) {
            int u;
            cin >> u;
            u--;
            update(u, cind++);
        } else if (t == 2) {
            int u, v;
            cin >> u >> v;
            u--;
            v--;
            int l = lca(u, v);
            dbg(l, u, v);
            auto nu = query_anc(l, u).reversed(), nv = query_anc(l, v);
            dbg(nu.v, nv.v, nu.r, nv.l);
            cout << (nu + nv).v + 1 << endl;
        } else {
            assert(t == 3);
            int u;
            cin >> u;
            u--;
            cout << st2.query(tin[u], tout[u] - 1);
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}