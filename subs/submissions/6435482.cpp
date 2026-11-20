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

void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

template <typename T>
T sorted(T arr) {
    sort(begin(arr), end(arr));
    return arr;
}

mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());

long rint(long l, long r) {
    return cowng() % (r - l + 1) + l;
}

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

template <typename C>
struct ST {
    using I = typename C::I;

    int n;
    vector<I> v, arr;

    ST(const vector<I>& arr) : n(sz(arr)), v(4 * n), arr(arr) {
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
        v[o] = C::merge(v[lc], v[rc]);
    }

    void update(int o, int l, int r, int ind, const I& x) {
        if (l == r) {
            v[o] = x;
            return;
        }

        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = C::merge(v[lc], v[rc]);
    }

    void update(int ind, const I& x) {
        update(1, 0, n - 1, ind, x);
    }

    I query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }

        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return C::merge(query(lc, l, mid, ql, qr),
                                query(rc, mid + 1, r, ql, qr));
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    I query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
    }

    I query2(int l, int r) const {
        return query(max(0, l), min(n - 1, r));
    }
};

template <typename T>
struct MinOp {
    using I = T;

    static I merge(I a, I b) {
        return min(a, b);
    }
};

template <typename T>
struct MaxOp {
    using I = T;

    static I merge(I a, I b) {
        return max(a, b);
    }
};

vector<vector<int>> to_adj_list(const vector<pair<int, int>>& edges) {
    int n = sz(edges) + 1;

    vector<vector<int>> graph(n);
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    return graph;
}

void pdfs(vector<vector<int>>& graph, int u) {
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(graph, v);
    }
}

namespace s1 {

bool solve(int rt, int ka, int kb, const vector<pair<int, int>>& edges) {
    int n = sz(edges) + 1;

    vector<vector<int>> graph(n);
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    if (sz(graph[rt]) != 1) {
        return false;
    }
    for (auto& a : graph) {
        if (sz(a) > 2) {
            return false;
        }
    }

    vector<int> arr;
    {
        auto cgraph = graph;
        pdfs(cgraph, rt);
        int u = rt;
        while (true) {
            arr.push_back(u);
            if (sz(cgraph[u])) {
                assert(sz(cgraph[u]) == 1);
                u = cgraph[u][0];
            } else {
                break;
            }
        }
        assert(sz(arr) == n);
    }

    ST<MinOp<int>> st_arr(arr);
    vector<int> dp_a(n), dp_b(n);
    ST<MinOp<int>> st_dpa(dp_a);

    for (int i = n - 1; i >= 0; i--) {
        {
            if (i + ka >= n) {
                dp_a[i] = st_arr.query2(i, i + kb);
            } else {
                dp_a[i] = dp_b[i + ka];
            }
            st_dpa.update(i, dp_a[i]);
        }
        {
            dp_b[i] = 1e9;
            // jump backwards
            if (i > 0) {
                dp_b[i] = min(dp_b[i], st_arr.query2(i - kb, i - 1));
            }
            dp_b[i] = min(dp_b[i], st_dpa.query2(i, i + kb));
        }
    }
    dbg(dp_a, dp_b);

    cout << dp_a[0] + 1 << endl;

    return true;
}

}  // namespace s1

struct Tree {
    int n;
    vector<vector<int>> graph;
    vector<int> par, tin, tout, depth, ord;

    Tree() {}
    Tree(const vector<pair<int, int>>& edges, int rt)
        : n(sz(edges) + 1),
          graph(to_adj_list(edges)),
          par(n),
          tin(n),
          tout(n),
          depth(n) {
        dfs(rt, -1, 0);
    }

    void dfs(int u, int p, int d) {
        par[u] = p;
        tin[u] = sz(ord);
        depth[u] = d;
        ord.push_back(u);

        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }

            dfs(v, u, d + 1);
        }

        tout[u] = sz(ord);
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }
};

struct BLift {
    int n, logn;
    vector<vector<int>> lift;

    BLift() {}
    BLift(const Tree& tree)
        : n(tree.n), logn(__lg(n + 5)), lift(logn + 1, vector<int>(n)) {
        lift[0] = tree.par;
        for (int i = 1; i <= logn; i++) {
            for (int j = 0; j < n; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
    }

    int anc_k(int u, int kv) const {
        for (int i = 0; i <= logn; i++) {
            if (on(kv, i) && u != -1) {
                u = lift[i][u];
            }
        }
        return u;
    }

    int imm_ch(const Tree& tree, int u, int ch) const {
        assert(tree.anc(u, ch) && !tree.anc(ch, u));

        for (int i = logn; i >= 0; i--) {
            int nch = lift[i][ch];
            if (nch != -1 && !tree.anc(nch, u)) {
                ch = nch;
            }
        }

        return ch;
    }
};

namespace s2 {

vector<int> opt_neq(const vector<int>& cv, bool want_min) {
    int n = sz(cv);
    assert(n >= 2);

    multiset<int> ms;
    for (auto& a : cv) {
        ms.insert(a);
    }

    vector<int> ans(n);
    for (int i = 0; i < n; i++) {
        ms.erase(ms.find(cv[i]));
        if (want_min) {
            ans[i] = *begin(ms);
        } else {
            ans[i] = *rbegin(ms);
        }
        ms.insert(cv[i]);
    }

    return ans;
}

int n, rt, ka, kb;
Tree tree;
BLift blift;
vector<vector<int>> graph_ch, dist;
vector<int> ch_id, anc_ka1, memo_a, memo_a2, memo_b, dp_b_end, dp_b_end2;

int dp_b(int u);

int dp_a(int u) {
    if (memo_a[u] != -1) {
        return memo_a[u];
    }

    int m = sz(graph_ch[u]);
    vector<int> chs(m, -1);

    for (int v = 0; v < n; v++) {
        if (!(dist[u][v] == ka && tree.anc(u, v))) {
            continue;
        }

        int u_ch = ch_id[anc_ka1[v]];
        if (u == 5) {
            dbg(v, u_ch);
        }
        chs[u_ch] = max(chs[u_ch], dp_b(v));
    }

    if (!m) {
        memo_a[u] = -1;
    } else {
        memo_a[u] = *max_element(begin(chs), end(chs));
    }
    if (m == 1) {
        memo_a2[graph_ch[u][0]] = -1;
    } else if (m > 1) {
        auto nopt = opt_neq(chs, false);
        for (int i = 0; i < m; i++) {
            memo_a2[graph_ch[u][i]] = nopt[i];
        }
    }

    if (memo_a[u] == -1) {
        memo_a[u] = dp_b_end[u];
    }
    for (auto& v : graph_ch[u]) {
        if (memo_a2[v] == -1) {
            memo_a2[v] = dp_b_end2[v];
        }
    }

    assert(memo_a[u] != -1);
    return memo_a[u];
}

int dp_a2(int u) {
    assert(u != rt);
    dp_a(tree.par[u]);
    assert(memo_a2[u] != -1);
    return memo_a2[u];
}

int dp_b(int u) {
    if (memo_b[u] != -1) {
        return memo_b[u];
    }

    int ans = dp_a(u);
    for (int v = 0; v < n; v++) {
        if (!(dist[u][v] <= kb)) {
            continue;
        }

        if (tree.anc(v, u)) {
            if (dist[u][v] == kb) {
                continue;
            }
            ans = min(ans, dp_a2(v));
        } else {
            ans = min(ans, dp_a(v));
        }
    }

    return memo_b[u] = ans;
}

void solve(int _rt, int _ka, int _kb, const vector<pair<int, int>>& edges) {
    n = sz(edges) + 1;
    rt = _rt;
    ka = _ka;
    kb = _kb;

    tree = Tree(edges, rt);
    blift = BLift(tree);

    graph_ch = to_adj_list(edges);
    pdfs(graph_ch, rt);

    ch_id.resize(n);
    for (auto& a : graph_ch) {
        for (int i = 0; i < sz(a); i++) {
            ch_id[a[i]] = i;
        }
    }

    dist.resize(n);
    {
        for (int i = 0; i < n; i++) {
            dist[i] = Tree(edges, i).depth;
        }
    }

    anc_ka1.resize(n);
    for (int i = 0; i < n; i++) {
        anc_ka1[i] = blift.anc_k(i, ka - 1);
    }

    dp_b_end.resize(n);
    dp_b_end2.resize(n);
    {
        for (int u = 0; u < n; u++) {
            int m = sz(graph_ch[u]);
            vector<int> chs(m, 1e9);
            chs.push_back(u);

            for (int v = 0; v < n; v++) {
                if (!(dist[u][v] <= kb && tree.anc(u, v) && u != v)) {
                    continue;
                }

                // int u_ch = ch_id[anc_ka1[v]];
                int u_ch = ch_id[blift.imm_ch(tree, u, v)];
                chs[u_ch] = min(chs[u_ch], v);
            }

            dp_b_end[u] = *min_element(begin(chs), end(chs));
            if (m) {
                auto nopt = opt_neq(chs, true);
                for (int i = 0; i < m; i++) {
                    dp_b_end2[graph_ch[u][i]] = nopt[i];
                }
            }
        }
    }

    memo_a.resize(n, -1);
    memo_a2.resize(n, -1);
    memo_b.resize(n, -1);

    // dbg(dp_a(1));
    // return;

    cout << dp_a(rt) + 1 << endl;
}

}  // namespace s2

void solve() {
    int n, rt, ka, kb;
    cin >> n >> rt >> ka >> kb;
    rt--;
    if (ka <= kb) {
        cout << 1 << endl;
        return;
    }

    vector<pair<int, int>> edges(n - 1);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }

    bool solved = s1::solve(rt, ka, kb, edges);
    if (!solved) {
        s2::solve(rt, ka, kb, edges);
    }
}

int main() {
    init_io();
    solve();
}