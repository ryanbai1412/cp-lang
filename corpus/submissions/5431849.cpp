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

    static constexpr mint inv(mint base) {
        return pow(base, MOD - 2);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}
    constexpr mint(long x, bool) : x(x) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    mint operator-() const {
        if (!x) {
            return 0;
        }
        return mint(MOD - x, true);
    }

    mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    mint& operator-=(const mint& m) {
        return *this += -m;
    }

    mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }
    mint& operator/=(const mint& m) {
        return *this *= inv(m);
    }

    friend mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend mint operator*(mint a, const mint& b) {
        return a *= b;
    }
    friend mint operator/(mint a, const mint& b) {
        return a /= b;
    }
};

struct M {
    static constexpr int maxn = 1e4 + 5;

    mint fact[maxn], ifact[maxn];

    M() {
        fact[0] = ifact[0] = 1;
        for (int i = 1; i < maxn; i++) {
            fact[i] = i * fact[i - 1];
            ifact[i] = mint::inv(fact[i]);
        }
    }

    mint choose(int n, int k) const {
        return fact[n] * ifact[k] * ifact[n - k];
    }

    mint choose1(int n, int k) const {
        if (k < 0 || k > n) {
            return 0;
        }
        return choose(n, k);
    }
} M;

// # of ways to place the given rank when scores are 1..=u
mint solve1(int m, int u, int rank) {
    int above = rank - 1, below = m - rank;
    mint ans = 0;
    for (int i = 1; i <= u; i++) {
        ans += mint::pow(i, below) * mint::pow(u - i, above);
    }
    return ans;
}

void solve() {
    int n, m, kv;
    cin >> m >> n >> kv;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int ranks[n];
    for (auto& a : ranks) {
        cin >> a;
    }
    mint dp[n + 1][m] {};
    dp[n][kv] = 1;
    for (int i = n - 1; i >= 0; i--) {
        int rank = ranks[i], u = arr[i], below = m - rank;
        mint ways = solve1(m, u, rank);
        dbg(rank, below, ways);
        for (int j = 0; j < m; j++) {
            for (int k = 0; k <= below; k++) {
                dp[i][j] += dp[i + 1][k] * ways * M.choose(j, k) *
                            M.choose1(m - j - 1, below - k);
            }
            dbg(i, j, dp[i][j]);
        }
    }
    cout << dp[0][m - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}