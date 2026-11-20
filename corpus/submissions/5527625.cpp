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

struct Lazy {
    static constexpr int NO_SET = -1;

    int set;
    long mul, m, b;

    friend ostream& operator<<(ostream& os, const Lazy& lazy) {
        os << "set: " << lazy.set << " mul: " << lazy.mul << " m: " << lazy.m
           << " b: " << lazy.b;
        return os;
    }

    Lazy operator+(const Lazy& l) const {
        if (l.set != NO_SET) {
            return l;
        }
        return {set, mul * l.mul, m * l.mul + l.m, b * l.mul + l.b};
    }

    long operator()(int x) const {
        long cx = x;
        if (set != NO_SET) {
            cx = set;
        }
        cx *= mul;
        cx += m * x + b;
        return cx;
    }

    static Lazy c_def() {
        return {NO_SET, 1, 0, 0};
    }

    static Lazy c_add(int x) {
        return {NO_SET, 1, 0, x};
    }

    static Lazy c_set(int x) {
        return {x, 1, 0, 0};
    }

    static Lazy c_mul(int x) {
        return {NO_SET, x, 0, 0};
    }

    static Lazy c_arith(int x) {
        return {NO_SET, 1, x, 0};
    }
};

struct ST {
    int n;
    vector<Lazy> lazy;

    ST(int n) : n(n), lazy(4 * n, Lazy::c_def()) {}

    void update(int o, int l, int r, int ql, int qr, const Lazy& x) {
        if (ql <= l && r <= qr) {
            lazy[o] = lazy[o] + x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        lazy[lc] = lazy[lc] + lazy[o];
        lazy[rc] = lazy[rc] + lazy[o];
        lazy[o] = Lazy::c_def();
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
    }

    void update(int l, int r, const Lazy& x) {
        update(1, 0, n - 1, l, r, x);
    }

    void update_all(const Lazy& x) {
        update(0, n - 1, x);
    }

    Lazy query(int o, int l, int r, int ind, Lazy clazy) {
        clazy = lazy[o] + clazy;
        if (l == r) {
            return clazy;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            return query(lc, l, mid, ind, clazy);
        } else {
            return query(rc, mid + 1, r, ind, clazy);
        }
    }

    Lazy query(int ind) {
        return query(1, 0, n - 1, ind, Lazy::c_def());
    }
};

void solve() {
    {
        dbg(Lazy::c_mul(2) + Lazy::c_arith(1));
        ST st(1);
        st.update_all(Lazy::c_mul(2));
        auto cx = st.query(0);
        dbg(cx.set, cx.mul, cx.m, cx.b);
        dbg(cx(7));
//        return;
    }
    int n, ql, qr;
    cin >> n >> ql >> qr;
    int ops[n][2];
    for (auto& [t, x] : ops) {
        cin >> t >> x;
    }
    int q;
    cin >> q;
    vector<int> arr(q);
    for (auto& a : arr) {
        cin >> a;
    }
    ST st(q);
    auto sarr = arr;
    sort(begin(sarr), end(sarr));
    auto st_get = [&](int ind) -> long {
        return st.query(ind)(sarr[ind]);
    };
    for (auto& [t, x] : ops) {
        if (t == 1) {
            st.update_all(Lazy::c_add(x));
        } else if (t == 2) {
            st.update_all(Lazy::c_add(-x));
        } else if (t == 3) {
            st.update_all(Lazy::c_mul(x));
        } else {
            st.update_all(Lazy::c_arith(x));
        }
        dbg(st_get(0));
        {
            int l = -1, r = q - 1;
            while (l < r) {
                int mid = (l + r + 1) / 2;
                if (st_get(mid) < ql) {
                    l = mid;
                } else {
                    r = mid - 1;
                }
            }
            if (l >= 0) {
                st.update(0, l, Lazy::c_set(ql));
            }
        }
        {
            int l = 0, r = q;
            while (l < r) {
                int mid = (l + r) / 2;
                if (st_get(mid) > qr) {
                    r = mid;
                } else {
                    l = mid + 1;
                }
            }
            if (l < q) {
                st.update(l, q - 1, Lazy::c_set(qr));
            }
        }
    }
    for (auto& a : arr) {
        int ind = int(lower_bound(begin(sarr), end(sarr), a) - sarr.begin());
        cout << st_get(ind) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}