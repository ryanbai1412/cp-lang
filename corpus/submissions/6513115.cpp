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

using u32 = uint32_t;
using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

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

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <u32 MOD>
struct Mint {
    static constexpr u32 norm(long x) {
        if (0 <= x && x < MOD) {
            return u32(x);
        } else if (-x <= MOD && x < 0) {
            return u32(x + MOD);
        }
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return u32(x);
    }

    static Mint pow(Mint base, long exp) {
        Mint ans = 1;

        while (exp) {
            if (exp & 1) {
                ans *= base;
            }
            base *= base;
            exp >>= 1;
        }

        return ans;
    }

    static Mint inv(Mint x) {
        return pow(x, MOD - 2);
    }

    u32 x;

    constexpr Mint() : x(0) {}
    constexpr Mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    static Mint from_unchecked(u32 x) {
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
        x = u32(u64(x) * m.x % MOD);
        return *this;
    }
    Mint& operator/=(const Mint& m) {
        return *this *= Mint::inv(m);
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
    friend Mint operator/(Mint a, const Mint& b) {
        return a /= b;
    }

    bool operator==(const Mint& m) const {
        return x == m.x;
    }
    bool operator!=(const Mint& m) const {
        return x != m.x;
    }
};

template <typename A, typename B>
struct I {
    A a;
    B b;

    constexpr I() : a {}, b {} {}
    constexpr I(long x) : a(x), b(x) {}

    I& operator+=(const I& x) {
        a += x.a;
        b += x.b;
        return *this;
    }
    I& operator-=(const I& x) {
        a -= x.a;
        b -= x.b;
        return *this;
    }
    I& operator*=(const I& x) {
        a *= x.a;
        b *= x.b;
        return *this;
    }
    I& operator/=(const I& x) {
        a /= x.a;
        b /= x.b;
        return *this;
    }

    friend I operator+(I a, const I& b) {
        return a += b;
    }
    friend I operator-(I a, const I& b) {
        return a -= b;
    }
    friend I operator*(I a, const I& b) {
        return a *= b;
    }
    friend I operator/(I a, const I& b) {
        return a /= b;
    }

    bool operator==(const I& x) const {
        return a == x.a && b == x.b;
    }
    bool operator!=(const I& x) const {
        return a != x.a || b != x.b;
    }
};

struct HC {
    // using A = I<Mint<int(1e9 + 7)>, Mint<int(1e9 + 9)>>;
    // using A = Mint<u32(2e9 + 11)>;
    using A = u64;
    static constexpr A base = 1467;
    static constexpr int MAXN = 5e6 + 5;

    A pow[MAXN];

    HC() {
        pow[0] = 1;
        for (int i = 1; i < MAXN; i++) {
            pow[i] = pow[i - 1] * base;
        }
    }
} hc;

using A = HC::A;

struct Hasher {
    int n;
    string s;
    vector<A> hsh;

    Hasher(const string& s) : n(sz(s)), s(s) {
        hsh.push_back(0);
        for (auto& a : s) {
            hsh.push_back(hsh.back() * HC::base + a);
        }
    }

    int size() const {
        return n;
    }

    A query(int l, int r) const {
        return hsh[r] - hsh[l] * hc.pow[r - l];
    }

    char operator[](int ind) const {
        return s[ind];
    }
};

struct SV {
    const Hasher* h;
    int off;

    SV(const Hasher& h, int off) : h(&h), off(off) {}

    int size() const {
        return h->size() - off;
    }

    A query(int l, int r) const {
        return h->query(off + l, off + r);
    }

    char operator[](int ind) const {
        return h->s[off + ind];
    }
};

template <typename A, typename B>
int lcp(const A& sa, const B& sb) {
    int l = 0, r = min(sz(sa), sz(sb));

    while (l < r) {
        int mid = (l + r + 1) / 2;
        if (sa.query(0, mid) == sb.query(0, mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }

    return l;
}

template <typename A, typename B>
int mscmp(const A& sa, const B& sb) {
    int l = lcp(sa, sb), na = sz(sa), nb = sz(sb);

    if (l == na && l == nb) {
        return 0;
    } else if (l == na) {
        return -1;
    } else if (l == nb) {
        return +1;
    }

    if (sa[l] < sb[l]) {
        return -1;
    } else if (sa[l] > sb[l]) {
        return +1;
    }
    assert(false);
}

template <typename T>
struct RMQ {
    int n, logn;
    vector<vector<T>> v;

    RMQ(const vector<T>& arr)
        : n(sz(arr)), logn(__lg(n + 5)), v(logn + 1, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i <= logn; i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = min(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
            }
        }
    }

    T query(int l, int r) const {
        int k = __lg(r - l);
        return min(v[k][l], v[k][r - (1 << k)]);
    }
};

struct ST {
    int n;
    vector<int> v;

    ST(int n) : n(n), v(4 * n, 1e9) {}

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

        v[o] = min(v[lc], v[rc]);
    }

    void update(int ind, int x) {
        update(1, 0, n - 1, ind, x);
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }

        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 1e9;
        if (ql <= mid) {
            ans = min(ans, query(lc, l, mid, ql, qr));
        }
        if (mid < qr) {
            ans = min(ans, query(rc, mid + 1, r, ql, qr));
        }

        return ans;
    }

    int query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
    }

    int queryl_le(int o, int l, int r, int ql, int x) const {
        if (l == r) {
            if (v[o] <= x) {
                return l;
            } else {
                return -1;
            }
        }

        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            int cans = queryl_le(lc, l, mid, ql, x);
            if (cans != -1) {
                return cans;
            } else if (v[rc] <= x) {
                return queryl_le(rc, mid + 1, r, ql, x);
            }
            return -1;
        } else {
            return queryl_le(rc, mid + 1, r, ql, x);
        }
    }

    int queryl_le(int cl, int x) const {
        return queryl_le(1, 0, n - 1, cl, x);
        int l = cl, r = n;

        while (l < r) {
            int mid = (l + r) / 2;
            if (query(cl, mid) <= x) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }

        return l;
    }
};

struct DS {
    int n;
    ST st1, st2;

    DS(int n) : n(n), st1(n), st2(n) {}

    void update(int ind, int x) {
        st1.update(ind, x);
        st2.update(n - ind - 1, x);
    }

    int queryl_le(int cl, int x) const {
        return st1.queryl_le(cl, x);
    }

    int queryr_re(int cr, int x) const {
        return n - st2.queryl_le(n - cr - 1, x) - 1;
    }
};

vector<int> inv_perm(const vector<int>& arr) {
    int n = sz(arr);

    vector<int> ans(n);
    for (int i = 0; i < n; i++) {
        ans[arr[i]] = i;
    }

    return ans;
}

void solve() {
    string s;
    int q;
    cin >> s >> q;

    vector<string> qs(q);
    vector<pair<int, int>> bounds(q);

    bool is_all = true;
    for (int qi = 0; qi < q; qi++) {
        auto& [l, r] = bounds[qi];
        cin >> qs[qi] >> l >> r;
        l--;
        r--;
        is_all &= l == 0 && r == sz(s) - 1;
        // assert(l == 0 && r == sz(s) - 1);
    }

    Hasher hsh_s(s);
    vector<Hasher> hsh_qs;
    for (auto& a : qs) {
        hsh_qs.emplace_back(a);
    }

    vector<SV> sa;
    auto push_hsh = [&](const Hasher& hsh) -> void {
        for (int i = 0; i < sz(hsh); i++) {
            sa.emplace_back(hsh, i);
        }
    };

    push_hsh(hsh_s);
    for (auto& a : hsh_qs) {
        push_hsh(a);
    }

    stable_sort(begin(sa), end(sa), [&](const auto& a, const auto& b) -> bool {
        return mscmp(a, b) < 0;
    });

    int m = sz(sa);
    vector<int> sa_lcp;
    for (int i = 0; i < m - 1; i++) {
        sa_lcp.push_back(lcp(sa[i], sa[i + 1]));
    }

    RMQ lcp_rmq(sa_lcp);
    auto query_lcp = [&](int u, int v) -> int {
        if (u == v) {
            return sz(sa[u]);
        }
        if (u > v) {
            swap(u, v);
        }
        return lcp_rmq.query(u, v);
    };
    auto sa_id = [&](int ind) -> int {
        auto& ch = sa[ind].h;
        if (ch == &hsh_s) {
            return -1;
        }

        return ch - hsh_qs.data();
    };

    vector<int> s_inds, s_suffs, inds[q];
    for (int i = 0; i < m; i++) {
        int id = sa_id(i);

        if (id != -1) {
            inds[id].push_back(i);
        } else {
            s_inds.push_back(i);
            s_suffs.push_back(sa[i].off);
        }
    }
    auto s_rsuff = inv_perm(s_suffs);

    if (is_all) {
        int prev_s[m], next_s[m];
        prev_s[0] = -1;
        for (int i = 0; i < m; i++) {
            if (sa_id(i) == -1) {
                prev_s[i] = i;
            }
            if (i + 1 < m) {
                prev_s[i + 1] = prev_s[i];
            }
        }

        next_s[m - 1] = -1;
        for (int i = m - 1; i >= 0; i--) {
            if (sa_id(i) == -1) {
                next_s[i] = i;
            }
            if (i) {
                next_s[i - 1] = next_s[i];
            }
        }

        for (int qi = 0; qi < q; qi++) {
            auto& cind = inds[qi];

            long ans = 0;

            for (int i = 0; i < sz(cind); i++) {
                int clcp = 0;
                if (i) {
                    clcp = query_lcp(cind[i - 1], cind[i]);
                }

                auto upd = [&](int ind) -> void {
                    if (ind == -1) {
                        return;
                    }
                    clcp = max(clcp, query_lcp(ind, cind[i]));
                };

                upd(prev_s[cind[i]]);
                upd(next_s[cind[i]]);

                ans += sz(sa[cind[i]]) - clcp;
            }

            cout << ans << endl;
        }
        return;
    }

    long qans[q];
    int qord[q];
    iota(qord, qord + q, 0);
    sort(qord, qord + q, CmpByKey([&](int ind) -> int {
        return -bounds[ind].first;
    }));

    DS ds(sz(s));
    int pbound = sz(s) - 1;

    for (auto& qi : qord) {
        for (; pbound >= bounds[qi].first; pbound--) {
            ds.update(s_rsuff[pbound], pbound);
            dbg(pbound);
        }

        auto& cind = inds[qi];

        long ans = 0;

        for (int i = 0; i < sz(cind); i++) {
            int clcp = 0;
            if (i) {
                clcp = query_lcp(cind[i - 1], cind[i]);
            }

            int pos = int(lower_bound(begin(s_inds), end(s_inds), cind[i]) -
                          begin(s_inds));
            dbg(pos, s_inds);

            auto check = [&](int x) -> bool {
                auto [cl, cr] = bounds[qi];
                cr -= x - 1;

                bool ok = false;

                auto go = [&](int ind) -> void {
                    if (ind == -1 || ind == sz(s)) {
                        return;
                    }
                    dbg(ind);
                    ok |= query_lcp(s_inds[ind], cind[i]) >= x;
                };
                if (pos < sz(s_inds)) {
                    go(ds.queryl_le(pos, cr));
                }
                if (pos) {
                    go(ds.queryr_re(pos - 1, cr));
                }

                return ok;
            };

            // dbg(check(0));
            // return;
            int l = 0, r = sz(sa[cind[i]]);
            while (l < r) {
                int mid = (l + r + 1) / 2;
                if (check(mid)) {
                    l = mid;
                } else {
                    r = mid - 1;
                }
            }
            clcp = max(clcp, l);

            ans += sz(sa[cind[i]]) - clcp;
        }

        qans[qi] = ans;
    }

    for (auto& a : qans) {
        cout << a << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}