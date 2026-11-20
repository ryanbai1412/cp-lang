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

constexpr int maxn = 1 << 19;

struct ST {
    long v[maxn << 1];

    ST() {
        fill(begin(v), end(v), -1e18);
    }

    void update(int o, int l, int r, int ind, long x) {
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
        v[o] = max(v[lc], v[rc]);
    }

    void update(int ind, long x) {
        update(1, 0, maxn - 1, ind, x);
    }

    long query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        long ans = -1e18;
        if (ql <= mid) {
            ans = max(ans, query(lc, l, mid, ql, qr));
        }
        if (mid < qr) {
            ans = max(ans, query(rc, mid + 1, r, ql, qr));
        }
        return ans;
    }

    long query(int l, int r) const {
        return query(1, 0, maxn - 1, l, r);
    }
} lst, rst;

void solve() {
    int n, u, d, s;
    cin >> n >> u >> d >> s;
    vector<pair<int, int>> arr[maxn];
    for (int i = 0; i < n; i++) {
        int t, l, c;
        cin >> t >> l >> c;
        arr[t].emplace_back(l, c);
    }
    lst.update(s, s * d);
    rst.update(s, -s * u);
    arr[maxn - 1].emplace_back(s, 0);
    for (auto& a : arr) {
        if (!sz(a)) {
            continue;
        }
        sort(begin(a), end(a));
        int m = sz(a);
        long opt[m], mval[m];
        fill(opt, opt + m, -1e18);
        fill(mval, mval + m, -1e18);
        for (int i = 0; i < m; i++) {
            mval[i] =
                max(lst.query(0, a[i].first) - a[i].first * d,
                    rst.query(a[i].first, maxn - 1) + a[i].first * u);
        }
        long psum[m + 1];
        psum[0] = 0;
        for (int i = 0; i < m; i++) {
            psum[i + 1] = psum[i] + a[i].second;
        }
        {
            long val[m], copt = -1e18;
            for (int i = m - 1; i >= 0; i--) {
                copt = max(copt, -(u + d) * a[i].first + psum[i + 1]);
                val[i] = copt + (u + d) * a[i].first - psum[i] + mval[i];
            }
            copt = -1e18;
            for (int i = m - 1; i >= 0; i--) {
                copt = max(copt, -u * a[i].first + psum[i] + val[i]);
                opt[i] = max(opt[i], copt + u * a[i].first - psum[i]);
            }
        }
        {
            long val[m], copt = -1e18;
            for (int i = 0; i < m; i++) {
                copt = max(copt, (u + d) * a[i].first - psum[i]);
                val[i] = copt - (u + d) * a[i].first + psum[i + 1] + mval[i];
            }
            copt = -1e18;
            for (int i = 0; i < m; i++) {
                copt = max(copt, d * a[i].first - psum[i + 1] + val[i]);
                opt[i] = max(opt[i], copt - d * a[i].first + psum[i + 1]);
            }
        }
        for (int i = 0; i < m; i++) {
            lst.update(a[i].first, opt[i] + a[i].first * d);
            rst.update(a[i].first, opt[i] - a[i].first * u);
        }
    }
    cout << lst.query(s, s) - s * d << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}