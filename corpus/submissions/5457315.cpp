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
    int n;
    vector<Node> v, arr;
    vector<int> lazy;

    ST1() {}
    ST1(const vector<Node>& arr)
        : n(sz(arr)), v(4 * n), arr(arr), lazy(4 * n, -1) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void maintain(int o, int l, int r) {
        if (lazy[o] != -1) {
            v[o] = Node::c_from(lazy[o]);
            return;
        }
        if (l == r) {
            v[o] = arr[l];
        } else {
            v[o] = v[o * 2] + v[o * 2 + 1];
        }
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] = x;
            maintain(o, l, r);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (lazy[o] != -1) {
            lazy[lc] = lazy[rc] = lazy[o];
            lazy[o] = -1;
        }
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        } else {
            maintain(lc, l, mid);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        } else {
            maintain(rc, mid + 1, r);
        }
        maintain(o, l, r);
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    Node query(int o, int l, int r, int ql, int qr) const {
        if (lazy[o] != -1) {
            return Node::c_from(lazy[o]);
        }
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr) +
                       query(rc, mid + 1, r, ql, qr);
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    Node query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
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
vector<int> graph[maxn], st;
ST1 st1;
ST2 st2;

void dfs1(int u, int p, int d) {
    par[u] = lift[0][u] = p;
    siz[u] = 1;
    depth[u] = d;
    st.push_back(u);
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        dfs1(v, u, d + 1);
        siz[u] += siz[v];
        if (siz[v] > siz[graph[u][0]]) {
            swap(v, graph[u][0]);
        }
    }
}

void dfs2(int u) {
    tin[u] = cc++;
    if (sz(graph[u])) {
        up[graph[u][0]] = up[u];
    }
    for (auto& v : graph[u]) {
        dfs2(v);
    }
    tout[u] = cc;
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
        dbg(lv, tin[v]);
        ans = Node::merge(st1.query(lv, tin[v]), ans);
        if (lv == tin[u]) {
            break;
        }
        v = par[up[v]];
    }
    return ans.value();
}

void update_st1(int u, int ind) {
    while (u >= 0) {
        int lu = tin[up[u]];
        st1.update(lu, tin[u], ind);
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