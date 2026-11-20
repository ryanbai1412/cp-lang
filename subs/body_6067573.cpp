
struct IO {
    char buf[20 << 20], *i = buf;
    IO() {
        std::ignore = fread_unlocked(buf, 1, sizeof(buf), stdin);
    }
    IO& operator>>(int& x) {
        bool neg = false;
        while (*i < '-') {
            i++;
        }
        if (*i == '-') {
            neg = true;
            i++;
        }
        x = 0;
        while (*i >= '0') {
            x = x * 10 + *(i++) - '0';
        }
        if (neg) {
            x = -x;
        }
        return *this;
    }
} io;
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
    io >> q >> m;
    vector<int> ans1, ans2;
    char* io_save = io.i;
    {
        map<int, pair<int, int>> colors;
        DS2 ds1(2 * m);
        for (int _qit = 0; _qit < q; _qit++) {
            int ty;
            io >> ty;
            if (ty == 1) {
                int t, c, l, r, d;
                io >> t >> c >> l >> r >> d;
                int len = r - l, l_pos = l;
                if (d == -1) {
                    l_pos = 2 * m - l;
                }
                l_pos = mod(l_pos - t, 2 * m);
                ds1.update(l_pos, l_pos + len, +1);
                dbg(l_pos, len);
                colors[c] = {l_pos, len};
            } else if (ty == 2) {
                int t, l, r;
                io >> t >> l >> r;
                int ans = 0;
                int base = mod(-t, 2 * m);
                dbg(base, base + r, base + l, base + l + 2 * m - 1);
                ans += ds1.query(base, base + r, base + l, 6 * m - 1);
                dbg(ans);
                dbg(base + max(r + 1, 2 * m - r), base + 2 * m - 1,
                    base + 2 * m - l + 1, base + 4 * m - l);
                dbg(max(r + 1, 2 * m - r), 2 * m - 1, 2 * m - l + 1, 4 * m - l);
                base += 2 * m;
                ans1.push_back(ans);
            } else {
                int _t, c;
                io >> _t >> c;
                auto& [l_pos, len] = colors[c];
                ds1.update(l_pos, l_pos + len, -1);
            }
        }
    }
    io.i = io_save;
    {
        map<int, pair<int, int>> colors;
        DS2 ds2(2 * m);
        for (int _qit = 0; _qit < q; _qit++) {
            int ty;
            io >> ty;
            if (ty == 1) {
                int t, c, l, r, d;
                io >> t >> c >> l >> r >> d;
                int len = r - l, l_pos = l;
                if (d == -1) {
                    l_pos = 2 * m - l;
                }
                l_pos = mod(l_pos - t, 2 * m);
                ds2.update(l_pos + 2 * m, l_pos - len + 2 * m, +1);
                dbg(l_pos, len);
                colors[c] = {l_pos, len};
            } else if (ty == 2) {
                int t, l, r;
                io >> t >> l >> r;
                int ans = 0;
                int base = mod(-t, 2 * m);
                dbg(base, base + r, base + l, base + l + 2 * m - 1);
                dbg(ans);
                dbg(base + max(r + 1, 2 * m - r), base + 2 * m - 1,
                    base + 2 * m - l + 1, base + 4 * m - l);
                dbg(max(r + 1, 2 * m - r), 2 * m - 1, 2 * m - l + 1, 4 * m - l);
                base += 2 * m;
                ans += ds2.query(base + max(r + 1, 2 * m - r), base + 2 * m - 1,
                                 0, base + 2 * m - l);
                ans2.push_back(ans);
            } else {
                int _t, c;
                io >> _t >> c;
                auto& [l_pos, len] = colors[c];
                ds2.update(l_pos + 2 * m, l_pos - len + 2 * m, -1);
            }
        }
    }
    for (int i = 0; i < sz(ans1); i++) {
        cout << ans1[i] + ans2[i] << endl;
    }
}
