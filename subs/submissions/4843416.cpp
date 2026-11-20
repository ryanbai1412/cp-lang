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

int mod;

struct ST {
    int n;
    vector<int> v;

    ST(int n) : n(n), v(1 << (__lg(n - 1) + 2), 1) {}

    void update(int o, int l, int r, int ind, int x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = v[lc] * v[rc] % mod;
    }

    void update(int ind, int x) {
        update(1, 0, n - 1, ind, (x + 1) % mod);
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr) *
                       query(rc, mid + 1, r, ql, qr) % mod;
            }
            return query(lc, l, mid, ql, qr);
        }
        return query(rc, mid + 1, r, ql, qr);
    }

    int query(int l, int r) const {
        if (l > r) {
            return 1;
        }
        return query(1, 0, n - 1, l, r);
    }
};

void solve() {
    int n, m;
    cin >> n >> m >> mod;
    pair<int, int> arr[n];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
        y--;
    }
    sort(arr, arr + n);
    int pos[m];
    vector<int> queries[n];
    {
        vector<int> inds[m];
        for (int i = 0; i < n; i++) {
            inds[arr[i].second].push_back(i);
        }
        bool vis[n] {};
        for (int i = n - 1; i >= 0; i--) {
            auto& [len, t] = arr[i];
            if (!vis[t]) {
                vis[t] = true;
                int qind = int(lower_bound(arr, arr + n,
                                           pair<int, int> {len / 2 + 1, -1}) -
                               arr);
                queries[qind].push_back(t);
                pos[t] = *lower_bound(begin(inds[t]), end(inds[t]), qind);
            }
        }
    }
    int qpsum[n];
    for (int i = 0; i < n; i++) {
        qpsum[i] = (i ? qpsum[i - 1] : 0) + sz(queries[i]);
    }
    int rqpos[m];
    for (int i = 0, j = 0; i < n; i++) {
        for (auto& a : queries[i]) {
            rqpos[a] = j++;
        }
    }
    bool ok[m] {};
    int cnt[m] {};
    ST st(m), qords(n);
    int ans = 0;
    for (int i = 0; i < n; i++) {
        for (auto& a : queries[i]) {
            dbg(pos[a], qpsum[pos[a]]);
            ok[a] = true;
            ans = (ans + st.query(0, n - 1) *
                             qords.query(rqpos[a] + 1, qpsum[pos[a]] - 1)) %
                  mod;
            dbg(a, ans);
            st.update(a, cnt[a] - 1);
            ans = (ans + st.query(0, n - 1)) % mod;
            st.update(a, cnt[a]);
            dbg(a, ans);
        }
        int t = arr[i].second;
        cnt[t]++;
        if (ok[t]) {
            st.update(t, cnt[t]);
        }
        qords.update(rqpos[t], cnt[t]);
    }
    dbg(ans);
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}