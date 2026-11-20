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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T, size_t N, size_t M>
using Mat = array<array<T, M>, N>;

template <typename T, size_t A, size_t B, size_t C>
Mat<T, A, C> operator*(const Mat<T, A, B>& a, const Mat<T, B, C>& b) {
    Mat<T, A, C> ans {};
    for (size_t i = 0; i < A; i++) {
        for (size_t j = 0; j < B; j++) {
            for (size_t k = 0; k < C; k++) {
                ans[i][k] += a[i][j] * b[j][k];
            }
        }
    }
    return ans;
}

struct Node {
    Mat<u64, 1, 5> v;

    Node& operator+=(const Node& n) {
        for (int i = 0; i < 5; i++) {
            v[0][i] += n.v[0][i];
        }
        return *this;
    }

    friend Node operator+(Node a, const Node& b) {
        return a += b;
    }

    static Node c_def() {
        return {{1, 0, 0, 0, 0}};
    }
};

struct Lazy {
    Mat<u64, 5, 5> v;

    Lazy& operator+=(const Lazy& l) {
        v = v * l.v;
        return *this;
    }

    friend Lazy operator+(Lazy a, const Lazy& b) {
        return a += b;
    }

    void apply_inplace(Node& n) {
        n.v = n.v * v;
    }

    Node apply(Node n) const {
        return {n.v * v};
    }

    static Lazy c_def() {
        return {{{{1, 0, 0, 0, 0},
                  {0, 1, 0, 0, 0},
                  {0, 0, 1, 0, 0},
                  {0, 0, 0, 1, 0},
                  {0, 0, 0, 0, 1}}}};
    }

    static Lazy c_adda(u64 x) {
        return {{{{1, x, 0, 0, 0},
                  {0, 1, 0, 0, 0},
                  {0, 0, 1, x, 0},
                  {0, 0, 0, 1, 0},
                  {0, 0, 0, 0, 1}}}};
    }

    static Lazy c_addb(u64 x) {
        return {{{{1, 0, x, 0, 0},
                  {0, 1, 0, x, 0},
                  {0, 0, 1, 0, 0},
                  {0, 0, 0, 1, 0},
                  {0, 0, 0, 0, 1}}}};
    }

    static Lazy c_add(bool b, u64 x) {
        return b ? c_addb(x) : c_adda(x);
    }

    static Lazy c_h_add() {
        return {{{{1, 0, 0, 0, 0},
                  {0, 1, 0, 0, 0},
                  {0, 0, 1, 0, 0},
                  {0, 0, 0, 1, 1},
                  {0, 0, 0, 0, 1}}}};
    }
};

struct ST {
    int n;
    vector<Node> v;
    vector<Lazy> lazy;

    ST(int n) : n(n), v(4 * n), lazy(4 * n, Lazy::c_def()) {}

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = Node::c_def();
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void maintain(int o, int l, int r) {
        if (l == r) {
            v[o] = Node::c_def();
        } else {
            v[o] = v[o * 2] + v[o * 2 + 1];
        }
        lazy[o].apply_inplace(v[o]);
    }

    void update(int o, int l, int r, int ql, int qr, const Lazy& x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            maintain(o, l, r);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        lazy[lc] += lazy[o];
        lazy[rc] += lazy[o];
        lazy[o] = Lazy::c_def();
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

    void update(int l, int r, const Lazy& x) {
        update(1, 0, n - 1, l, r, x);
    }

    Node query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
        if (ql <= l && r <= qr) {
            return clazy.apply(v[o]);
        }
        clazy = lazy[o] + clazy;
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr, clazy) +
                       query(rc, mid + 1, r, ql, qr, clazy);
            }
            return query(lc, l, mid, ql, qr, clazy);
        }
        return query(rc, mid + 1, r, ql, qr, clazy);
    }

    Node query(int l, int r) const {
        return query(1, 0, n - 1, l, r, Lazy::c_def());
    }
};

struct DS1 {
    vector<pair<int, int>> st;

    DS1(int n) : st {{n, 1e9}} {}

    template <typename Cb>
    void update(int ind, int x, const Cb& cb) {
        while (st.back().second < x) {
            cb(st.back().first, st[sz(st) - 2].first - 1, st.back().second);
            st.pop_back();
        }
        cb(ind, ind, 0);
        st.emplace_back(ind, x);
    }
};

void solve() {
    int _id, n;
    cin >> _id >> n;
    vector<int> arr[2];
    for (auto& a : arr) {
        a.resize(n);
        for (auto& b : a) {
            cin >> b;
        }
    }
    int q;
    cin >> q;
    u64 qans[q];
    vector<pair<int, int>> queries[n];
    for (int i = 0; i < q; i++) {
        int l, r;
        cin >> l >> r;
        l--;
        r--;
        queries[l].emplace_back(r, i);
    }
    ST st(n);
    DS1 ds[2] {DS1(n), DS1(n)};
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 0; j < 2; j++) {
            ds[j].update(i, arr[j][i], [&](int l, int r, int x) -> void {
                dbg(j, l, r, arr[j][i] - x);
                st.update(l, r, Lazy::c_add(j, arr[j][i] - x));
            });
        }
        st.update(0, n - 1, Lazy::c_h_add());
        for (auto& [r, qi] : queries[i]) {
            dbg(st.query(i, r).v[0][3]);
            qans[qi] = st.query(i, r).v[0][4];
        }
    }
    for (auto& a : qans) {
        cout << a << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}