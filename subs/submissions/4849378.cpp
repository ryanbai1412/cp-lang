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
    int siz;
    long sm;

    Node operator+(const Node& n) const {
        return {siz + n.siz, sm + n.sm};
    }
};

struct ST {
    int n;
    vector<Node> v;

    ST(int n) : n(n), v(4 * n, {0, 0}) {}

    template <typename T>
    void update(int o, int l, int r, int ind, const T& t) {
        if (l == r) {
            t(v[o]);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, t);
        } else {
            update(rc, mid + 1, r, ind, t);
        }
        v[o] = v[lc] + v[rc];
    }

    template <typename T>
    void update(int ind, const T& t) {
        update(1, 0, n - 1, ind, t);
    }

    void set(int ind, const Node& x) {
        update(1, 0, n - 1, ind, [&](Node& y) -> void {
            y = x;
        });
    }

    void set(int ind, int x) {
        set(ind, {x, long(ind) * x});
    }

    void add(int ind, int cnt) {
        update(ind, [&](Node& y) -> void {
            y.siz += cnt;
            y.sm += long(ind) * cnt;
        });
    }

    Node query(int o, int l, int r, int ql, int qr) const {
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

void solve() {
    int n;
    long kv;
    cin >> n >> kv;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    ST st(n + 2);
    int base = n, mx = n;
    for (int i = 0; i < n; i++) {
        if (i == 3) {
            for (int j = base; j <= n + 1; j++) {
                dbg(j, st.query(j, j).siz);
            }
        }
        int l = base - 1, r = 2 * n;
        while (l < r) {
            int mid = (l + r) / 2;
            if (st.query(base - 1, mid).siz >= arr[i] - 1) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        auto cur = st.query(base - 1, l);
        int extra = cur.siz - (arr[i] - 1);
        long cost = cur.sm - long(l) * extra + long(arr[i] - 1) * (1 - base);
        if (cost <= kv) {
            st.set(n + 1, 0);
            for (int j = mx; j > l; j--) {
                st.set(j, 0);
            }
            st.add(l, -extra);
            mx = l;
            st.add(--base, i - (arr[i] - 1));
            long cans = kv - cost;
            if (cans + base <= mx) {
                st.add(int(cans) + base, 1);
            } else {
                st.set(n + 1, {1, kv - cost + base});
            }
            cout << cans << endl;
        } else {
            st.add(base - 1, 1);
            cout << -1 << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}