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
    int cnt0, cnt1, v, pref, suff;

    Node operator+(const Node& n) const {
        return {cnt0 + n.cnt0, cnt1 + n.cnt1, max({v, n.v, suff + n.pref}),
                !cnt1 ? cnt0 + n.pref : pref, !n.cnt1 ? n.cnt0 + suff : n.suff};
    }

    static Node from_lazy(int x, int l, int r) {
        int siz = r - l + 1;
        if (x == 0) {
            return {siz, 0, siz, siz, siz};
        } else {
            assert(x == 1);
            return {0, siz, 0, 0, 0};
        }
    }
};

struct ST {
    static constexpr Node DEF {0, 1, 0, 0, 0};

    int n;
    vector<Node> v;
    vector<int> lazy;

    ST(int n) : n(n), v(4 * n), lazy(4 * n, -1) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = DEF;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void maintain(int o, int l, int r) {
        if (lazy[o] != -1) {
            v[o] = Node::from_lazy(lazy[o], l, r);
            return;
        }
        if (l == r) {
            v[o] = DEF;
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
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (lazy[o] != -1) {
            return Node::from_lazy(lazy[o], max(l, ql), min(r, qr));
        }
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr) + query(rc, mid + 1, r, ql, qr);
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
    int n, q;
    cin >> n >> q;
    ST st(n);
    while (q--) {
        int t;
        cin >> t;
        if (t == 0) {
            int l, r;
            cin >> l >> r;
            l--;
            r--;
            st.update(l, r, 0);
        } else if (t == 2) {
            int l, r;
            cin >> l >> r;
            l--;
            r--;
            cout << st.query(l, r).v << endl;
        } else {
            int l0, r0, l1, r1;
            cin >> l0 >> r0 >> l1 >> r1;
            l0--;
            r0--;
            l1--;
            r1--;
            int kv = st.query(l0, r0).cnt1;
            if (!kv) {
                continue;
            }
            st.update(l0, r0, 0);
            int l = l1, r = r1;
            while (l < r) {
                int mid = (l + r) / 2;
                if (st.query(l1, mid).cnt0 >= kv) {
                    r = mid;
                } else {
                    l = mid + 1;
                }
            }
            st.update(l1, l, 1);
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}