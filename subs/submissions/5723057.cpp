#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

struct mint {
    static constexpr int MOD = 1e9 + 7;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    static constexpr mint pow(mint base, long exp) {
        mint ans = 1;

        while (exp) {
            if (exp & 1) {
                ans *= base;
            }
            base *= base;
            exp >>= 1;
        }

        return ans;
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    static constexpr mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    constexpr mint operator-() const {
        if (!x) {
            return {};
        }
        return mint::from_unchecked(MOD - x);
    }

    constexpr mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr mint& operator-=(const mint& m) {
        return *this += -m;
    }

    constexpr mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    friend constexpr mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend constexpr mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend constexpr mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

template <typename Cb>
void submasks(int mask, const Cb& cb) {
    for (int i = mask;; i = (i - 1) & mask) {
        cb(i);
        if (!i) {
            break;
        }
    }
}

constexpr long MOD = 1e9 + 7;

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    for (int i = 1; i < n; i++) {
        int p;
        cin >> p;
    }

    if (!kv) {
        long ans = 1;
        for (int i = 0; i < m; i++) {
            ans = (ans * (2 * n - 1)) % MOD;
            n++;
        }
        cout << ans << endl;
        return;
    }

    long dp[2][1 << kv] {};
    int cdp = 0, pdp = 1;

    dp[cdp][0] = 1;

    int pcnt[1 << kv];
    for (int mask = 0; mask < (1 << kv); mask++) {
        pcnt[mask] = __builtin_popcount(mask);
    }

    for (int i = m - 1; i >= 0; i--) {
        swap(cdp, pdp);

        for (int mask = (1 << (kv - 1)); mask < (1 << kv); mask++) {
            dp[cdp][mask] = dp[pdp][(mask << 1) ^ (1 << kv)];
        }

        for (int mask = 0; mask < (1 << (kv - 1)); mask++) {
            int cn = n + i + pcnt[mask];

            dp[cdp][mask] = (2 * cn - 1) * dp[pdp][mask << 1];

            dp[cdp][mask] += (cn - 1) * dp[pdp][(mask << 1) | 1];

            for (int x = mask; x;) {
                int j = x & -x;

                dp[cdp][mask] += dp[pdp][(mask & (~j)) << 1];

                x ^= j;
            }

            dp[cdp][mask] %= MOD;
        }
    }

    cout << dp[cdp][0] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
#ifndef LOCAL
    freopen("tree.in", "r", stdin);
    freopen("tree.out", "w", stdout);
#endif
    int tc_id, tcs;
    cin >> tc_id >> tcs;
    while (tcs--) {
        solve();
    }
}