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
    using A = Mint<u32(2e9 + 11)>;
    // using A = u64;
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

void solve() {
    string s;
    int q;
    cin >> s >> q;

    vector<string> qs(q);
    vector<pair<int, int>> bounds(q);

    for (int qi = 0; qi < q; qi++) {
        auto& [l, r] = bounds[qi];
        cin >> qs[qi] >> l >> r;
        l--;
        r--;
        assert(l == 0 && r == sz(s) - 1);
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

    sort(begin(sa), end(sa), [&](const auto& a, const auto& b) -> bool {
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

    vector<int> inds[q];
    for (int i = 0; i < m; i++) {
        int id = sa_id(i);

        if (id != -1) {
            inds[id].push_back(i);
        }
    }

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

        int ans = 0;

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
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}