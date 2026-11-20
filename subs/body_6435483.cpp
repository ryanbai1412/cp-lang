
void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}
mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());
long rint(long l, long r) {
    return cowng() % (r - l + 1) + l;
}
template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}
template <int MOD>
struct Mint {
    static int norm(long x) {
        if (0 <= x && x < MOD) {
            return x;
        } else if (-MOD <= x && x < 0) {
            return x + MOD;
        }
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }
    int x;
    Mint() : x(0) {}
    Mint(long x) : x(norm(x)) {}
    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }
    static Mint from_unchecked(int x) {
        Mint m;
        m.x = x;
        return m;
    }
    Mint operator-() const {
        if (!x) {
            return {};
        }
        return Mint::from_unchecked(MOD - x);
    }
    Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    Mint& operator-=(const Mint& m) {
        return *this += -m;
    }
    Mint& operator*=(const Mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }
    friend Mint operator+(Mint a, const Mint& b) {
        return a += b;
    }
    friend Mint operator-(Mint a, const Mint& b) {
        return a -= b;
    }
    friend Mint operator*(Mint a, const Mint& b) {
        return a *= b;
    }
};
using mint = Mint<1'000'003>;
struct BDS {
    int n;
    vector<int> va;
    vector<mint> vb;
    BDS(int n) : n(n), va(n), vb(n) {}
    void update_b(int ind, mint x) {
        vb[ind] = x;
    }
    void update_a(int l, int r, int x) {
        assert(l <= r);
        for (int i = l; i <= r; i++) {
            va[i] += x;
        }
    }
    void update_a2(int l, int r, int x) {
        if (l > r) {
            return;
        }
        update_a(l, r, x);
    }
    mint query_all() const {
        mint ans = 0;
        for (int i = 0; i < n; i++) {
            if (!va[i]) {
                ans += vb[i];
            }
        }
        return ans;
    }
};
struct NV {
    int mn;
    mint x;
    NV operator+(const NV& o) const {
        if (mn == o.mn) {
            return {mn, x + o.x};
        } else if (mn < o.mn) {
            return *this;
        } else {
            return o;
        }
    }
    NV update_lazy(int lazy) const {
        return {mn + lazy, x};
    }
};
struct FDS {
    int n;
    vector<int> lazy;
    vector<NV> v;
    FDS(int n) : n(n), lazy(4 * n), v(4 * n) {}
    void update_b(int o, int l, int r, int ind, mint x) {
        if (l == r) {
            v[o].x = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update_b(lc, l, mid, ind, x);
        } else {
            update_b(rc, mid + 1, r, ind, x);
        }
        v[o] = (v[lc] + v[rc]).update_lazy(lazy[o]);
    }
    void update_b(int ind, mint x) {
        update_b(1, 0, n - 1, ind, x);
    }
    void update_a(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            v[o].mn += x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update_a(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update_a(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = (v[lc] + v[rc]).update_lazy(lazy[o]);
    }
    void update_a(int l, int r, int x) {
        update_a(1, 0, n - 1, l, r, x);
    }
    void update_a2(int l, int r, int x) {
        if (l > r) {
            return;
        }
        update_a(l, r, x);
    }
    mint query_all() const {
        if (v[1].mn > 0) {
            return {};
        }
        return v[1].x;
    }
};
using DS = FDS;
void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    vector<mint> dp(n + 1);
    dp[n] = 1;
    vector<int> nxt[2], g_nxt(n, -1), dup_inter[2];
    for (auto& a : nxt) {
        a.resize(n, -1);
    }
    for (auto& a : dup_inter) {
        a.resize(n, -1);
    }
    DS dss[2] {DS(n + 1), DS(n + 1)};
    for (int i = n - 1; i >= 0; i--) {
        int x = arr[i];
        for (int cb : {0, 1}) {
            auto& unq_nxt = nxt[cb ^ 1];
            auto& dup_nxt = nxt[cb];
            auto& cds = dss[cb];
            auto& cdup_inter = dup_inter[cb];
            cds.update_b(i + 1, dp[i + 1]);
            bool i_dup = i % 2 == cb;
            if (unq_nxt[x] != -1) {
                cds.update_a2(unq_nxt[x] + 1, n, +1);
            }
            if (!i_dup) {  // is unique
                if (g_nxt[x] != -1) {
                    cds.update_a2(g_nxt[x] + 1, n, +1);
                }
            } else {
                int d_nxt = dup_nxt[x], n_set;
                if (d_nxt != -1) {
                    cds.update_a2(d_nxt + 1, cdup_inter[d_nxt], -1);
                    n_set = d_nxt;
                } else {
                    n_set = n;
                }
                cdup_inter[i] = n_set;
                cds.update_a2(i + 1, cdup_inter[i], +1);
            }
            if (!i) {
                dbg(cb, cds.query_all());
            }
            dp[i] += cds.query_all();
        }
        nxt[i & 1][x] = i;
        g_nxt[x] = i;
    }
    dbg(dp);
    cout << dp[0] << endl;
}
