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

#define int long

struct ST {
    int n;
    vector<long> v, lazy;

    ST(int n) : n(n), v(4 * n, 0), lazy(4 * n, -1) {}

    void maintain(int o, int l, int r) {
        if (lazy[o] != -1) {
            v[o] = lazy[o] * (r - l + 1);
            return;
        }
        if (l == r) {
            v[o] = 0;
        } else {
            v[o] = v[o * 2] + v[o * 2 + 1];
        }
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] = x;
            v[o] = x * (r - l + 1);
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

    long query(int o, int l, int r, int ql, int qr) const {
        if (lazy[o] != -1) {
            return lazy[o] * (min(r, qr) - max(l, ql) + 1);
        } else if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        long ans = 0;
        if (ql <= mid) {
            ans += query(lc, l, mid, ql, qr);
        }
        if (mid < qr) {
            ans += query(rc, mid + 1, r, ql, qr);
        }
        return ans;
    }

    long query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
    }
};

struct DS1 {
    vector<pair<int, int>> sma, smb;
    ST sta, stb;
    int ind = 0;
    long ans = 0;

    DS1(int n) : sta(n), stb(n) {}

    void insert(vector<pair<int, int>>& sm, ST& stm, ST& sto, int x) {
        int si = ind;
        while (sz(sm) && sm.back().second <= x) {
            auto [pi, px] = sm.back();
            sm.pop_back();
            ans -= px * sto.query(pi, si - 1);
            si = pi;
        }
        ans += x * sto.query(si, ind);
        sm.emplace_back(si, x);
        stm.update(si, ind, x);
    }

    void insert(int ax, int bx) {
        insert(sma, sta, stb, ax);
        insert(smb, stb, sta, bx);
        ind++;
    }

    long query() const {
        return ans;
    }
};

void solve() {
    int _id, n;
    cin >> _id >> n;
    int arra[n], arrb[n];
    for (auto& a : arra) {
        cin >> a;
    }
    for (auto& a : arrb) {
        cin >> a;
    }
    int q;
    cin >> q;
    while (q--) {
        int l, r;
        cin >> l >> r;
        l--;
        r--;
        DS1 ds(n);
        long ans = 0;
        for (int i = l; i <= r; i++) {
            ds.insert(arra[i], arrb[i]);
            ans += ds.query();
        }
        cout << ans << endl;
    }
}

int32_t main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}