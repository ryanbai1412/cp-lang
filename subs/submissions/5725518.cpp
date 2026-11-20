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

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

struct ST1 {
    struct Node {
        int mx, ind;

        Node operator+(const Node& n) const {
            if (mx > n.mx) {
                return *this;
            }
            return n;
        }

        Node with_lazy(int x) const {
            return {mx + x, ind};
        }
    };

    int n;
    vector<Node> v, arr;
    vector<int> lazy;

    ST1(const vector<Node>& arr) : n(sz(arr)), v(4 * n), arr(arr), lazy(4 * n) {
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

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            v[o].mx += x;
            lazy[o] += x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = (v[lc] + v[rc]).with_lazy(lazy[o]);
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    Node query_all() const {
        return v[1];
    }
};

struct DS2 {
    int n, m;
    set<int> v, good;

    DS2(int n, int m) : n(n), m(m) {}

    bool check(set<int>::iterator it) const {
        if (it == v.begin()) {
            if (sz(v) == 1) {
                return true;
            }
            return *it + n - *v.rbegin() >= m;
        }
        int cur = *it, prv = *(--it);
        return cur - prv >= m;
    }

    void check_update(set<int>::iterator it) {
        if (it == v.end()) {
            return;
        }

        if (check(it)) {
            good.insert(*it);
        } else {
            good.erase(*it);
        }
    }

    void insert(int x) {
        auto [it, inserted] = v.insert(x);
        assert(inserted);
        check_update(it);
        check_update(++it);
        check_update(v.begin());
    }

    void erase(int x) {
        bool erased = !!v.erase(x);
        assert(erased);
        good.erase(x);
        check_update(v.lower_bound(x));
        check_update(v.begin());
    }
};

vector<int> solve_layers(int m, const vector<int>& arr) {
    int n = sz(arr);

    ST1 st_arr(({
        vector<ST1::Node> v(n);
        for (int i = 0; i < n; i++) {
            v[i] = {arr[i], i};
        }
        v;
    }));

    int ans_it = 0, ans_cnt = 0;
    vector<int> ans(n);

    DS2 m_imp(n, m);

    while (ans_cnt < n) {
        while (true) {
            auto it = st_arr.query_all();
            if (it.mx == m - 1) {
                m_imp.insert(it.ind);
                st_arr.update(it.ind, it.ind, -1e9);
            } else {
                break;
            }
        }

        vector<int> c_vals(begin(m_imp.good), end(m_imp.good));

        for (auto& cur : c_vals) {
            ans_cnt++;
            ans[cur] = ans_it;
            if (cur - m + 1 < 0) {
                st_arr.update(0, cur, +1);
                st_arr.update(cur - m + 1 + n, n - 1, +1);
            } else {
                st_arr.update(cur - m + 1, cur, +1);
            }

            m_imp.erase(cur);
        }

        ans_it++;
    }

    dbg(ans);
    return ans;
}

struct QConn {
    int n, logn, m;
    vector<int> layers;
    vector<vector<int>> lift;

    QConn() {}
    QConn(int m, const vector<int>& _layers)
        : n(sz(_layers)),
          logn(__lg(n) + 3),
          m(m),
          layers(_layers),
          lift(logn + 1, vector<int>(2 * n)) {
        auto tmp = layers;
        layers.insert(layers.end(), begin(tmp), end(tmp));

        map<int, int> q;
        for (int i = 2 * n - 1; i >= 0; i--) {
            if (i + m < 2 * n) {
                q.erase(layers[i + m]);
            }
            assert(!q.count(layers[i]));
            q[layers[i]] = i;

            auto it = q.upper_bound(layers[i]);
            if (it == q.end()) {
                lift[0][i] = -1;
            } else {
                lift[0][i] = it->second;
            }
        }

        for (int i = 1; i <= logn; i++) {
            for (int j = 0; j < 2 * n; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
    }

    bool query_lt(int u, int v) const {
        if (layers[u] >= layers[v]) {
            return false;
        }
        if (v < u) {
            v += n;
        }

        for (int i = logn; i >= 0; i--) {
            int nu = lift[i][u];
            if (nu != -1 && layers[nu] <= layers[v]) {
                u = nu;
            }
        }

        return v <= u;
    }
};

struct Solver {
    int n, m;
    vector<int> layers;
    QConn q_f, q_r;

    Solver() {}
    Solver(int m, const vector<int>& arr)
        : n(sz(arr)),
          m(m),
          layers(solve_layers(m, arr)),
          q_f(m, layers),
          q_r(m, reversed(layers)) {
        dbg(layers);
    }

    bool query_lt(int u, int v) const {
        return q_f.query_lt(u, v) || q_r.query_lt(n - u - 1, n - v - 1);
    }

    int query(int u, int v) const {
        if (query_lt(u, v)) {
            return -1;
        } else if (query_lt(v, u)) {
            return 1;
        }
        return 0;
    }
} solver;

void init(int m, vector<int> arr) {
    solver = Solver(m, arr);
}

int compare_plants(int u, int v) {
    return solver.query(u, v);
}