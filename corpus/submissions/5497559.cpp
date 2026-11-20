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

long divf(long a, long b) {
    assert(b > 0);
    if (a >= 0) {
        return a / b;
    } else {
        a = -a;
        return -((a + b - 1) / b);
    }
}

struct Node {
    long mn, mx, sum;

    Node operator+(const Node& n) const {
        return {min(mn, n.mn), max(mx, n.mx), sum + n.sum};
    }

    static Node c_def() {
        return {};
    }

    static Node c_from(long x) {
        return {x, x, x};
    }
};

struct Lazy {
    static constexpr long NO_SET = -1e18;

    long set, add;

    Lazy operator+(const Lazy& l) const {
        if (l.set != NO_SET) {
            return l;
        }
        return {set, add + l.add};
    }

    Node apply(Node n, int l, int r) const {
        if (set != NO_SET) {
            long cset = set + add;
            return {cset, cset, (r - l + 1) * cset};
        }
        return {n.mn + add, n.mx + add, n.sum + (r - l + 1) * add};
    }

    static Lazy c_def() {
        return {NO_SET, 0};
    }

    static Lazy c_set(long x) {
        return {x, 0};
    }
};

struct ST {
    static constexpr long LAZY_SET = -1e18;

    int n;
    vector<Node> v, arr;
    vector<Lazy> lazy;

    ST(const vector<Node>& arr)
        : n(sz(arr)), v(4 * n), arr(arr), lazy(4 * n, Lazy::c_def()) {
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
        if (l == r) {
            v[o] = arr[l];
        } else {
            v[o] = v[o * 2] + v[o * 2 + 1];
        }
        v[o] = lazy[o].apply(v[o], l, r);
    }

    void pushdown(int o, int l, int r) {
        if (l == r) {
            return;
        }
        int lc = o * 2, rc = lc + 1;
        lazy[lc] = lazy[lc] + lazy[o];
        lazy[rc] = lazy[rc] + lazy[o];
        lazy[o] = Lazy::c_def();
    }

    void update1(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o].add += x;
            maintain(o, l, r);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        pushdown(o, l, r);
        if (ql <= mid) {
            update1(lc, l, mid, ql, qr, x);
        } else {
            maintain(lc, l, mid);
        }
        if (mid < qr) {
            update1(rc, mid + 1, r, ql, qr, x);
        } else {
            maintain(rc, mid + 1, r);
        }
        maintain(o, l, r);
    }

    void update1(int l, int r, int x) {
        update1(1, 0, n - 1, l, r, x);
    }

    void update2(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            maintain(o, l, r);
            long cmn = divf(v[o].mn, x), cmx = divf(v[o].mx, x);
            if (cmn == cmx) {
                lazy[o] = Lazy::c_set(cmn);
                maintain(o, l, r);
                return;
//            } else if (v[o].mx - v[o].mn == 1) {
//                lazy[o].add += cmx - v[o].mx;
//                maintain(o, l, r);
//                return;
            }
            if (v[o].mn == -1 && v[o].mx == 0) {
                return;
            }
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        pushdown(o, l, r);
        if (ql <= mid) {
            update2(lc, l, mid, ql, qr, x);
        } else {
            maintain(lc, l, mid);
        }
        if (mid < qr) {
            update2(rc, mid + 1, r, ql, qr, x);
        } else {
            maintain(rc, mid + 1, r);
        }
        maintain(o, l, r);
    }

    void update2(int l, int r, int x) {
        update2(1, 0, n - 1, l, r, x);
    }

    Node query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
        if (ql <= l && r <= qr) {
            return clazy.apply(v[o], l, r);
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

void solve() {
    int n, q;
    cin >> n >> q;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    vector<Node> varr;
    for (int i = 0; i < n; i++) {
        varr.push_back(Node::c_from(arr[i]));
    }
    ST st(varr);
    while (q--) {
        int ty, l, r;
        cin >> ty >> l >> r;
        l--;
        r--;
        if (ty == 1) {
            int x;
            cin >> x;
            st.update1(l, r, x);
        } else if (ty == 2) {
            int x;
            cin >> x;
            st.update2(l, r, x);
        } else if (ty == 3) {
            cout << st.query(l, r).mn << endl;
        } else {
            cout << st.query(l, r).sum << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}