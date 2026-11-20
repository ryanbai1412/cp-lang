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
#define sz(x) int(size(x))

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "{" << p.first << ", " << p.second << "}";
}

struct Node {
    int len;
    pair<int, int> mn, pref, suff;

    static pair<int, int> mx(const pair<int, int>& a, const pair<int, int>& b) {
        if (a.first == b.first) {
            return max(a, b);
        }
        return min(a, b);
    }

    static pair<int, int> merge(const pair<int, int>& a,
                                const pair<int, int>& b) {
        if (a.first == b.first) {
            return {a.first, a.second + b.second};
        }
        return a;
    }

    Node operator+(const Node& n) const {
        return {len + n.len, mx(mx(mn, n.mn), merge(suff, n.pref)),
                pref.second == len ? merge(pref, n.pref) : pref,
                n.suff.second == n.len ? merge(n.suff, suff) : n.suff};
    }

    void apply_lazy(int x) {
        mn.first += x;
        suff.first += x;
        pref.first += x;
    }

    static Node zero() {
        return {1, {0, 1}, {0, 1}, {0, 1}};
    }

    friend ostream& operator<<(ostream& os, const Node& node) {
        os << "len: " << node.len << " mn: " << node.mn
           << " suff: " << node.suff << " pref: " << node.pref;
        return os;
    }
};

struct ST {
    int n;
    vector<Node> v;
    vector<int> lazy;

    ST(int n) : n(n), v(4 * n, Node::zero()), lazy(4 * n) {
        build(1, 0, n - 1);
    }
    
    void build(int o, int l, int r) {
        if (l == r) {
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            v[o].apply_lazy(x);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = v[lc] + v[rc];
        v[o].apply_lazy(lazy[o]);
    }

    void update(int l, int r, int x) {
        dbg(l, r, x);
        update(1, 0, n - 1, l, r, x);
    }

    int query() const {
        auto& [mn, mn_cnt] = v[1].mn;
        return !mn ? mn_cnt : 0;
    }
};

void solve() {
    int n, m, _b, k;
    cin >> n >> m >> _b >> k;
    int arr[k][5];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
            b--;
        }
    }
    vector<array<int, 3>> upds[n + 1];
    for (auto& [x1, y1, x2, y2, _] : arr) {
        upds[x1].push_back({y1, y2, +1});
        upds[x2].push_back({y1, y2, -1});
    }
    ST st(m);
    int ans = 0;
    for (int i = 0, j = 0; i < n; i++) {
        while (j < n) {
            for (auto& [l, r, x] : upds[j]) {
                if (x > 0) {
                    st.update(l, r, x);
                }
            }
            j++;
            if (st.query() < j - i) {
                break;
            }
            ans = max(ans, j - i);
        }
        dbg(i, j);
        if (j == n && st.query() >= j - i) {
            ans = max(ans, j - i);
        }
        for (auto& [l, r, x] : upds[i]) {
            if (x < 0) {
                st.update(l, r, x);
            }
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}