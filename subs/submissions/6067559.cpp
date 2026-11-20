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

struct DS2 : DS {
    int n;

    DS2(int n) : DS(3 * n, 3 * n), n(n) {}

    void update(int x, int y, int v) {
        dbg(n, x, y, v);
        DS::update(x, y, v);
        DS::update(x + n, y + n, v);
    }

    // int query(int xl, int xr, int yl, int yr) const {
    //     static constexpr int iinf = 1e7;
    //
    //     auto norm = [&](int& x) -> void {
    //         if (x < -iinf) {
    //             x = 0;
    //         } else if (x > iinf) {
    //             x = 2 * n - 1;
    //         }
    //     };
    //
    //     norm(xl);
    //     norm(xr);
    //     norm(yl);
    //     norm(yr);
    //
    //     return DS::query(xl, xr, yl, yr);
    // }
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
            ans += ds2.query(base + max(r + 1, 2 * m - r), base + 2 * m - 1,
                             base - l + 1, base + 2 * m - l);

            cout << ans << endl;
        } else {
            int c;
            cin >> c;

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