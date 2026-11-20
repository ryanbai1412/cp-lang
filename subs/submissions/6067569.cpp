#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

int mod(int a, int b) {
    a %= b;
    if (a < 0) {
        a += b;
    }
    return a;
}

struct BIT1 {
    int n;
    vector<int> v;

    BIT1(int n) : n(n), v(n + 1) {}

    void update(int ind, int x) {
        ind++;
        while (ind <= n) {
            v[ind] += x;
            ind += ind & -ind;
        }
    }

    int query(int ind) const {
        int ans = 0;
        while (ind > 0) {
            ans += v[ind];
            ind -= ind & -ind;
        }
        return ans;
    }

    int query(int l, int r) const {
        return query(r) - query(l);
    }
};

struct BIT2 {
    int n;
    vector<BIT1> v;

    BIT2(int n, int m) : n(n), v(n + 1, BIT1(m)) {}

    void update(int ind, int y, int val) {
        ind++;
        while (ind <= n) {
            v[ind].update(y, val);
            ind += ind & -ind;
        }
    }

    int query(int ind, int y) const {
        int ans = 0;
        while (ind > 0) {
            ans += v[ind].query(y);
            ind -= ind & -ind;
        }
        return ans;
    }

    int query(int xl, int xr, int yl, int yr) const {
        return query(xr, yr) - query(xl, yr) - query(xr, yl) + query(xl, yl);
    }

    int query_i(int xl, int xr, int yl, int yr) const {
        return query(xl, xr + 1, yl, yr + 1);
    }
};

struct DS {
    vector<vector<int>> arr;

    DS(int n, int m) : arr(n, vector<int>(m)) {}

    void update(int x, int y, int v) {
        arr[x][y] += v;
    }

    int query(int xl, int xr, int yl, int yr) const {
        int ans = 0;

        for (int i = xl; i <= xr; i++) {
            for (int j = yl; j <= yr; j++) {
                ans += arr[i][j];
            }
        }

        return ans;
    }
};

struct DS2 : BIT2 {
    int n;

    DS2(int n) : BIT2(2 * n, 2 * n), n(n) {}

    void update(int x, int y, int v) {
        dbg(n, x, y, v);
        BIT2::update(x, y, v);
    }

    int query(int xl, int xr, int yl, int yr) const {
        auto q2 = [&](int xl, int xr, int yl, int yr) -> int {
            if (xr < 0 || yr < 0 || xl >= 2 * n || yl >= 2 * n) {
                return 0;
            }

            xl = max(xl, 0);
            yl = max(yl, 0);

            xr = min(xr, 2 * n - 1);
            yr = min(yr, 2 * n - 1);

            return BIT2::query_i(xl, xr, yl, yr);
        };

        return q2(xl, xr, yl, yr) + q2(xl - n, xr - n, yl - n, yr - n);
    }
};

void solve() {
    int q, m;
    cin >> q >> m;

    map<int, pair<int, int>> colors;
    DS2 ds1(2 * m), ds2(2 * m);

    while (q--) {
        int ty;
        cin >> ty;

        if (ty == 1) {
            int t, c, l, r, d;
            cin >> t >> c >> l >> r >> d;

            int len = r - l, l_pos = l;
            if (d == -1) {
                l_pos = 2 * m - l;
            }
            l_pos = mod(l_pos - t, 2 * m);

            ds1.update(l_pos, l_pos + len, +1);
            ds2.update(l_pos + 2 * m, l_pos - len + 2 * m, +1);

            dbg(l_pos, len);
            colors[c] = {l_pos, len};
        } else if (ty == 2) {
            int t, l, r;
            cin >> t >> l >> r;

            int ans = 0;

            int base = mod(-t, 2 * m);
            dbg(base, base + r, base + l, base + l + 2 * m - 1);
            ans += ds1.query(base, base + r, base + l, 6 * m - 1);
            dbg(ans);
            dbg(base + max(r + 1, 2 * m - r), base + 2 * m - 1,
                base + 2 * m - l + 1, base + 4 * m - l);
            dbg(max(r + 1, 2 * m - r), 2 * m - 1, 2 * m - l + 1, 4 * m - l);
            base += 2 * m;
            ans += ds2.query(base + max(r + 1, 2 * m - r), base + 2 * m - 1, 0,
                             base + 2 * m - l);

            cout << ans << endl;
        } else {
            int _t, c;
            cin >> _t >> c;

            auto& [l_pos, len] = colors[c];

            ds1.update(l_pos, l_pos + len, -1);
            ds2.update(l_pos + 2 * m, l_pos - len + 2 * m, -1);
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}