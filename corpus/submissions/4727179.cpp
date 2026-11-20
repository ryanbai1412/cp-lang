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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using vi = vector<int>;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

struct ST {
    int n;
    vector<int> v;

    ST(int n) : n(n), v(4 * n, 1e9) {}

    void update(int o, int l, int r, int ind, int x) {
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
        v[o] = min(v[lc], v[rc]);
    }

    void update(int ind, int x) {
        update(1, 0, n - 1, ind, x);
    }

    int query(int o, int l, int r, int ql, int qr) {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 1e9;
        if (ql <= mid) {
            ans = min(ans, query(lc, l, mid, ql, qr));
        }
        if (mid < qr) {
            ans = min(ans, query(rc, mid + 1, r, ql, qr));
        }
        return ans;
    }

    int query(int l, int r) {
        return query(1, 0, n - 1, l, r);
    }
};

struct DSU {
    vector<int> p, pc;

    DSU(int n) : p(n, -1), pc(n, - 1) {}

    int find(int u) {
        return pc[u] < 0 ? u : (pc[u] = find(pc[u]));
    }

    void merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u != v) {
            p[u] = pc[u] = v;
        }
    }
};

struct Tree {
    int n;
    vector<int> ord;
    vector<pair<int, int>> ranges;
    vector<vector<int>> lift, graph;

    Tree(const vector<int>& p)
        : n(sz(p)), ranges(n), lift(__lg(n) + 2, vector<int>(n)), graph(n) {
        vector<int> roots;
        for (int i = 0; i < n; i++) {
            if (p[i] >= 0) {
                graph[p[i]].push_back(i);
            } else {
                roots.push_back(i);
            }
        }
        for (auto& a : roots) {
            dfs(a);
        }
        lift[0] = p;
        for (int i = 0; i <= __lg(n); i++) {
            for (int j = 0; j < n; j++) {
                if (lift[i][j] == -1) {
                    lift[i + 1][j] = -1;
                } else {
                    lift[i + 1][j] = lift[i][lift[i][j]];
                }
            }
        }
    }

    void dfs(int u) {
        ranges[u].first = sz(ord);
        ord.push_back(u);
        for (auto& v : graph[u]) {
            dfs(v);
        }
        ranges[u].second = sz(ord);
    }

    template <typename T>
    pair<int, int> bsearch(int u, const T& t) {
        for (int i = __lg(n); i >= 0; i--) {
            int v = lift[i][u];
            if (v != -1 && t(v)) {
                u = v;
            }
        }
        return ranges[u];
    }
};

vector<int> solve(const vector<int>& arr1,
                  const vector<int>& arr2,
                  const vector<array<int, 4>>& queries) {
    int n = sz(arr1), q = sz(queries);
    pair<int, int> arr[n];
    for (int i = 0; i < n; i++) {
        arr[arr1[i]].first = i;
        arr[arr2[i]].second = i;
    }
    vector<int> upds[n], qs[n];
    for (auto& [x, y] : arr) {
        upds[x].push_back(y);
    }
    for (int qi = 0; qi < q; qi++) {
        auto& [l1, r1, l2, r2] = queries[qi];
        qs[l1].push_back(qi);
    }
    bool ans[q];
    ST st(n);
    for (int i = n - 1; i >= 0; i--) {
        for (auto& a : upds[i]) {
            st.update(a, i);
        }
        for (auto& a : qs[i]) {
            auto& [l1, r1, l2, r2] = queries[a];
            ans[a] = st.query(l2, r2 - 1) < r1;
        }
    }
    return vector<int>(ans, ans + q);
}

vi check_validity(int n, vi us, vi vs, vi qsrc, vi qdest, vi ql, vi qr) {
    int q = sz(qsrc);
    vector<int> graph[n];
    for (int i = 0; i < sz(us); i++) {
        graph[us[i]].push_back(vs[i]);
        graph[vs[i]].push_back(us[i]);
    }
    DSU inc(n), dec(n);
    for (int i = 0; i < n; i++) {
        for (auto& a : graph[i]) {
            if (a <= i) {
                inc.merge(a, i);
            }
        }
    }
    for (int i = n - 1; i >= 0; i--) {
        for (auto& a : graph[i]) {
            if (a >= i) {
                dec.merge(a, i);
            }
        }
    }
    Tree inct(inc.p), dect(dec.p);
    vector<array<int, 4>> queries(q);
    for (int qi = 0; qi < q; qi++) {
        int src = qsrc[qi], dest = qdest[qi], l = ql[qi], r = qr[qi];
        auto [il, ir] = inct.bsearch(dest, [&](int x) -> bool {
            return x <= r;
        });
        auto [dl, dr] = dect.bsearch(src, [&](int x) -> bool {
            return x >= l;
        });
        queries[qi] = {il, ir, dl, dr};
    }
    dbg("A");
    return solve(inct.ord, dect.ord, queries);
}