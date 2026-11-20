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

long psum1(long n) {
    return n * (n + 1) / 2;
}

long psum2(long n) {
    return n * (n + 1) * (2 * n + 1) / 6;
}

template <typename Cb>
long query_psum(const Cb& cb, long l, long r) {
    return cb(r) - cb(l - 1);
}

struct Node {
    long xy, x, y, x2;

    Node operator+(const Node& n) const {
        return {xy + n.xy, x + n.x, y + n.y, x2 + n.x2};
    }

    static Node c_from(long x, long y) {
        return {x * y, x, y, x * x};
    }

    static Node c_range(long x, long y, int l, int r) {
        long xsum = query_psum(psum1, x + l, x + r);
        return {query_psum(psum2, x + l, x + r) + xsum * (y - x), xsum,
                query_psum(psum1, y + l, y + r),
                query_psum(psum2, x + l, x + r)};
    }
};

struct Lazy {
    static constexpr long NO_SET = -1e18;

    long xset, yset, xadd, yadd;

    Lazy operator+(const Lazy& l) const {
        if (l.xset != NO_SET) {
            return l;
        }
        return {xset, yset, xadd + l.xadd, yadd + l.yadd};
    }

    Lazy& operator+=(const Lazy& l) {
        return *this = *this + l;
    }

    Node apply(const Node& n, int l, int r) const {
        if (xset != NO_SET) {
            return Node::c_range(xset + xadd, yset + yadd, l, r);
        }
        long m = r - l + 1;
        return {n.xy + xadd * n.y + yadd * n.x + m * xadd * yadd,
                n.x + m * xadd, n.y + m * yadd,
                n.x2 + 2 * xadd * n.x + m * xadd * xadd};
    }

    static Lazy c_def() {
        return c_add(0, 0);
    }

    static Lazy c_add(long addx, long addy) {
        return {NO_SET, NO_SET, addx, addy};
    }

    static Lazy c_set(long setx, long sety) {
        assert(setx != NO_SET && sety != NO_SET);
        return {setx, sety, 0, 0};
    }
};

struct ST {
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

    void update(int o, int l, int r, int ql, int qr, const Lazy& x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            dbg("A", o, l, r);
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
    int arr[n][2];
    for (int i = 0; i < 2; i++) {
        for (auto& a : arr) {
            cin >> a[i];
        }
    }
    vector<Node> stv;
    for (auto& [x, y] : arr) {
        stv.push_back(Node::c_from(x, y));
    }
    ST st(stv);
    while (q--) {
        int t, l, r;
        cin >> t >> l >> r;
        l--;
        r--;
        if (t == 1) {
            auto [sxy, sx, sy, sx2] = st.query(l, r);
            long m = r - l + 1;
            long num = sxy * m - sx * sy, den = m * sx2 - sx * sx;
            if (den == 0) {
                continue;
            }
            cout << double(num) / double(den) << endl;
        } else if (t == 2) {
            int addx, addy;
            cin >> addx >> addy;
            st.update(l, r, Lazy::c_add(addx, addy));
        } else {
            int setx, sety;
            cin >> setx >> sety;
            setx++;
            sety++;
            st.update(l, r, Lazy::c_set(setx, sety));
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(5);
    solve();
}