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

using ll = long long;
using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

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

template <typename Cb>
struct Cmp {
    Cb cb;

    Cmp(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

vector<vector<pair<int, long>>> edges_to_adj(
    int n,
    const vector<tuple<int, int, long>>& edges) {
    vector<vector<pair<int, long>>> graph(n);

    for (auto& [u, v, w] : edges) {
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }

    return graph;
}

struct DistDFS {
    vector<long> dist;
    vector<vector<pair<int, long>>> graph;

    DistDFS(int root, int n, const vector<tuple<int, int, long>>& edges)
        : dist(n), graph(edges_to_adj(n, edges)) {
        dfs(root, -1, 0);
    }

    void dfs(int u, int p, long d) {
        dist[u] = d;

        for (auto& [v, w] : graph[u]) {
            if (v == p) {
                continue;
            }

            dfs(v, u, d + w);
        }
    }
};

struct PathDFS {
    int n;
    vector<int> path;
    vector<char> on_path;
    vector<vector<int>> path_subs;
    vector<vector<pair<int, long>>> graph;

    PathDFS(int u0, int u1, int n, const vector<tuple<int, int, long>>& edges)
        : n(n), on_path(n), graph(edges_to_adj(n, edges)) {
        pdfs(u0, -1, u1);

        for (auto& a : path) {
            on_path[a] = true;
        }

        for (auto& a : path) {
            path_subs.emplace_back();
            dfs(a, -1, path_subs.back());
        }
    }

    vector<int> st;

    void pdfs(int u, int p, int targ) {
        st.push_back(u);

        if (u == targ) {
            path = st;
        }

        for (auto& [v, _w] : graph[u]) {
            if (v == p) {
                continue;
            }

            pdfs(v, u, targ);
        }

        st.pop_back();
    }

    void dfs(int u, int p, vector<int>& out) {
        out.push_back(u);

        for (auto& [v, _w] : graph[u]) {
            if (v == p || on_path[v]) {
                continue;
            }

            dfs(v, u, out);
        }
    }
};

template <typename T>
vector<int> coord_comp(const vector<T>& arr) {
    vector<pair<T, int>> v;
    for (int i = 0; i < sz(arr); i++) {
        v.emplace_back(arr[i], i);
    }
    sort(begin(v), end(v));

    vector<int> comp(sz(arr));
    for (int i = 0; i < sz(v); i++) {
        comp[v[i].second] = i;
    }

    return comp;
}

struct MArr {
    vector<long> vals;
    vector<int> comp;

    MArr(const vector<long>& vals) : vals(vals), comp(coord_comp(vals)) {}
};

struct Node {
    long sum, last0, last1;
    int cnt;

    Node operator+(const Node& n) const {
        if (n.last1 == -1) {
            return {sum + n.sum, last0, last1, cnt + n.cnt};
        } else if (n.last0 == -1) {
            return {sum + n.sum, last1, n.last1, cnt + n.cnt};
        } else {
            return {sum + n.sum, n.last0, n.last1, cnt + n.cnt};
        }
    }

    Node operator-(const Node& n) const {
        return {sum - n.sum, -1, -1, cnt - n.cnt};
    }

    static Node c_from(long x) {
        return {x, -1, x, 1};
    }

    static constexpr Node c_def() {
        return {0, -1, -1, 0};
    }
};

template <typename T>
constexpr T default_value() {
    return {};
}

template <>
constexpr Node default_value<Node>() {
    return Node::c_def();
}

template <typename T>
struct ST {
    int n;
    vector<T> v;

    ST(int _n) : n(1 << (__lg(_n) + 1)), v(2 * n, default_value<T>()) {}

    T query_point(int ind) const {
        return v[ind + n];
    }

    void update(int ind, const T& x) {
        ind += n;
        v[ind] = x;
        ind >>= 1;
        for (; ind; ind >>= 1) {
            v[ind] = v[ind * 2] + v[ind * 2 + 1];
        }
    }

    T query_pref(int ind) const {
        if (ind < 0) {
            return default_value<T>();
        } else if (ind >= n - 1) {
            return v[1];
        }

        T ans = default_value<T>();

        ind += n + 1;
        for (; ind > 1; ind >>= 1) {
            if (ind & 1) {
                ans = v[ind ^ 1] + ans;
            }
        }
        return ans;
    }

    T query_all() const {
        return v[1];
    }

    template <typename Cb>
    pair<int, T> bsearch(int o, int l, int r, const T& pref, const Cb& cb)
        const {
        if (l == r) {
            return {l - 1, pref};
        }

        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (!cb(pref + v[lc])) {
            return bsearch(lc, l, mid, pref, cb);
        } else {
            return bsearch(rc, mid + 1, r, pref + v[lc], cb);
        }
    }

    template <typename Cb>
    pair<int, T> bsearch(const Cb& cb) const {
        if (cb(v[1])) {
            return {n, v[1]};
        }
        return bsearch(1, 0, n - 1, default_value<T>(), cb);
    }
};

struct VEB {
    static constexpr int MAXD = 3, MAXN = 1 << (6 * MAXD);

    u64 v[MAXD][MAXN >> 6] {};

    void toggle(int ind) {
        auto set = [&](u64& mask, int bit, bool b) -> void {
            if (b) {
                mask |= u64(1) << bit;
            } else {
                mask &= ~(u64(1) << bit);
            }
        };

        v[MAXD - 1][ind >> 6] ^= u64(1) << (ind & 63);
        ind >>= 6;

        for (int i = MAXD - 2; i >= 0; i--) {
            set(v[i][ind >> 6], ind & 63, !!v[i + 1][ind]);
            ind >>= 6;
        }
    }

    optional<int> query_pred(int ind) {
        if (ind <= 0) {
            return {};
        }
        for (int i = MAXD - 1; i >= 0; i--) {
            u64 cur = v[i][ind >> 6] & ((u64(1) << (ind & 63)) - 1);
            if (!cur) {
                ind >>= 6;
                continue;
            }

            ind = (ind & (~63)) | int(__lg(cur));
            for (int j = i + 1; j < MAXD; j++) {
                ind = (ind << 6) | int(__lg(v[j][ind]));
            }
            return ind;
        }
        return {};
    }

    optional<int> query_succ(int ind) {
        for (int i = MAXD - 1; i >= 0; i--) {
            u64 cur = v[i][ind >> 6] >> 1 >> (ind & 63);
            if (!cur) {
                ind >>= 6;
                continue;
            }

            ind = (ind & (~63)) | (__builtin_ctzll(cur) + (ind & 63) + 1);
            for (int j = i + 1; j < MAXD; j++) {
                ind = (ind << 6) | __builtin_ctzll(v[j][ind]);
            }
            return ind;
        }
        return {};
    }
};

struct DS {
    int n;
    MArr arr[2];
    ST<Node> v_st[2];
    ST<long> v_stl;
    VEB v_inds[2];
    vector<int> comp, ord0;

    DS(const vector<long>& v0, const vector<long>& v1)
        : n(sz(v0)), arr {v0, v1}, v_st {n, n}, v_stl(2 * n) {
        vector<long> vals;
        vals.insert(vals.end(), begin(v0), end(v0));
        for (auto& a : v1) {
            vals.push_back(2 * a);
        }

        comp = coord_comp(vals);

        vector<int> ord(2 * n);
        for (int i = 0; i < 2 * n; i++) {
            ord[comp[i]] = i;
        }

        int last0 = -1;
        for (auto& a : ord) {
            if (a < n) {
                last0 = arr[0].comp[a];
            }
            ord0.push_back(last0);
        }
    }

    template <int IND>
    void insert(int x) {
        if constexpr (!IND) {
            v_inds[IND].toggle(arr[IND].comp[x]);
        }
        v_st[IND].update(arr[IND].comp[x], Node::c_from(arr[IND].vals[x]));
        int ox = x + IND * n;
        v_stl.update(comp[ox], arr[IND].vals[x]);
    }

    template <int IND>
    void erase(int x) {
        if constexpr (!IND) {
            v_inds[IND].toggle(arr[IND].comp[x]);
        }
        v_st[IND].update(arr[IND].comp[x], Node::c_def());
        int ox = x + IND * n;
        v_stl.update(comp[ox], 0);
    }

    int query(long kv) {
        int ans = -1e9;

        auto upd_ans_q0_q1 = [&](const Node& q0, const Node& q1) -> void {
            if (q0.sum + q1.sum <= kv) {
                ans = max(ans, q0.cnt * 2 + q1.cnt);
            }
        };
        auto upd_ans_q0 = [&](const Node& q0) -> void {
            auto q1 = v_st[1]
                          .bsearch([&](const Node& o) -> bool {
                              return q0.sum + o.sum <= kv;
                          })
                          .second;

            upd_ans_q0_q1(q0, q1);
        };

        int l = ({
            int ind =
                v_stl.bsearch([&](long x) -> bool { return x <= kv; }).first;

            int l;
            if (ind < 0) {
                l = -1;
            } else if (ind >= 2 * n) {
                l = n;
            } else {
                l = ord0[ind];
            }

            l;
        });

        Node last_q0 = v_st[0].query_pref(l);

        upd_ans_q0(last_q0);
        upd_ans_q0(Node::c_def());
        {
            auto [u, q0] = v_st[0].bsearch(
                [&](const Node& o) -> bool { return o.sum <= kv; });
            upd_ans_q0(q0);
            if (u - 1 >= 0) {
                upd_ans_q0(q0 - v_st[0].query_point(u - 1));
            }
        }
        {
            auto [u, q0] = v_st[0].bsearch([&](const Node& o) -> bool {
                return o.sum <= kv - v_st[1].query_all().sum;
            });
            upd_ans_q0(q0);
            if (u + 1 < v_st[0].n) {
                upd_ans_q0(q0 + v_st[0].query_point(u + 1));
            }
        }

        {
            int u = l;
            Node q0 = last_q0;

            for (int it = 0; it < 2; it++) {
                auto n_u = v_inds[0].query_succ(u);
                if (!n_u) {
                    break;
                }

                u = n_u.value();
                q0 = q0 + v_st[0].query_point(u);
                upd_ans_q0(q0);
            }
        }
        {
            int u = l;
            Node q0 = last_q0;

            for (int it = 0; it < 2; it++) {
                auto n_u = v_inds[0].query_pred(u);
                if (!n_u) {
                    break;
                }

                u = n_u.value();
                q0 = q0 - v_st[0].query_point(u);
                upd_ans_q0(q0);
            }
        }

        return ans;
    }
};

int solve_disjoint(long kv,
                   const vector<long>& dist0,
                   const vector<long>& dist1) {
    vector<long> dists;
    dists.insert(dists.end(), begin(dist0), end(dist0));
    dists.insert(dists.end(), begin(dist1), end(dist1));

    sort(begin(dists), end(dists));

    long sum = 0;
    int i;
    for (i = 0; i < sz(dists); i++) {
        sum += dists[i];
        if (sum > kv) {
            break;
        }
    }

    return i;
}

int solve(int n, int u0, int u1, long kv, vector<tuple<int, int, long>> edges) {
    auto dist0 = DistDFS(u0, n, edges).dist, dist1 = DistDFS(u1, n, edges).dist;
    auto path_dfs = PathDFS(u0, u1, n, edges);
    auto path = path_dfs.path_subs;
    int m = sz(path);

    vector<long> cost1(n), cost2(n), delta(n);

    for (int i = 0; i < n; i++) {
        cost1[i] = min(dist0[i], dist1[i]);
        cost2[i] = max(dist0[i], dist1[i]);
        delta[i] = cost2[i] - cost1[i];
    }

    map<long, vector<int>> mp;

    for (int i = 0; i < m; i++) {
        mp[delta[path[i][0]]].push_back(i);
    }

    DS ds(cost2, cost1);

    int ans = solve_disjoint(kv, dist0, dist1);
    dbg(ans);

    int ans_add = 0;
    long min_cost = 0;
    for (auto& a : path_dfs.path) {
        ans_add++;
        min_cost += cost1[a];
    }

    auto move_vals = [&](vector<int> nodes, bool undo) -> void {
        sort(begin(nodes), end(nodes),
             Cmp([&](int u) -> long { return cost1[u]; }));

        for (auto& u : nodes) {
            if (path_dfs.on_path[u]) {
                ans_add++;
                min_cost += delta[u];
            } else {
                ds.erase<1>(u);
                ds.insert<0>(u);
            }

            ans = max(ans, ans_add + ds.query(kv - min_cost));
            dbg(ans, ans_add, min_cost);
        }

        if (!undo) {
            return;
        }

        for (auto& u : nodes) {
            if (path_dfs.on_path[u]) {
                ans_add--;
                min_cost -= delta[u];
            } else {
                ds.insert<1>(u);
                ds.erase<0>(u);
            }
        }
    };

    for (int i = 0; i < n; i++) {
        if (path_dfs.on_path[i]) {
            continue;
        }
        ds.insert<1>(i);
    }

    for (auto& [_k, vals] : mp) {
        dbg(vals);
        if (sz(vals) == 1) {
            move_vals(path[vals[0]], false);
            continue;
        }

        assert(sz(vals) == 2);
        dbg(ans_add, min_cost);
        move_vals(path[vals[0]], true);
        dbg(ans_add, min_cost);
        move_vals(path[vals[1]], false);
        move_vals(path[vals[0]], false);
    }

    return ans;
}

int max_score(int n,
              int u0,
              int u1,
              ll kv,
              vector<int> edges_u,
              vector<int> edges_v,
              vector<int> edges_w) {
    vector<tuple<int, int, long>> edges;

    for (int i = 0; i < n - 1; i++) {
        edges.emplace_back(edges_u[i], edges_v[i], edges_w[i]);
    }

    return solve(n, u0, u1, kv, edges);
}