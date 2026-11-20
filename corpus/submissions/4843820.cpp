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

constexpr int nlazy = INT_MIN;

struct Node {
    long pref, sm;

    Node operator+(const Node& n) const {
        return {max(pref, sm + n.pref), sm + n.sm};
    }

    Node apply(int lazy, int l, int r) const {
        if (lazy == nlazy) {
            return *this;
        } else {
            long x = long(lazy) * (r - l + 1);
            return {max(long(0), x), x};
        }
    }

    static Node from(int x) {
        return {max(0, x), x};
    }
};

struct ST {
    int n;
    vector<Node> v;
    vector<int> lazy;

    ST(int n) : n(n), v(4 * n, Node::from(0)), lazy(4 * n, nlazy) {}

    void maintain(int o, int l, int r) {
        if (l == r) {
            v[o] = Node::from(0);
        } else {
            v[o] = v[o * 2] + v[o * 2 + 1];
        }
        v[o] = v[o].apply(lazy[o], l, r);
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] = x;
        } else {
            int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
            if (lazy[o] != nlazy) {
                lazy[lc] = lazy[rc] = lazy[o];
                lazy[o] = nlazy;
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
        }
        maintain(o, l, r);
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    Node query(int o, int l, int r, int ql, int qr, int clazy) {
        if (ql <= l && r <= qr) {
            return v[o].apply(clazy, l, r);
        }
        if (clazy == nlazy) {
            clazy = lazy[o];
        }
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

    Node query(int l, int r) {
        return query(1, 0, n - 1, l, r, nlazy);
    }
};

void solve() {
    int n;
    cin >> n;
    ST st(n);
    while (true) {
        char t;
        cin >> t;
        if (t == 'I') {
            int l, r, x;
            cin >> l >> r >> x;
            l--;
            r--;
            st.update(l, r, x);
        } else if (t == 'Q') {
            dbg(st.query(0, 3).pref);
            int x;
            cin >> x;
            int l = -1, r = n - 1;
            while (l < r) {
                int mid = (l + r + 1) / 2;
                if (st.query(0, mid).pref <= x) {
                    l = mid;
                } else {
                    r = mid - 1;
                }
            }
            cout << l + 1 << endl;
        } else {
            break;
        }
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}