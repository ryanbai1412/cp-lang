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

template <int MOD>
struct Mint {
    static constexpr int norm(int x) {
        x %= MOD;
        if (x < 0) {
            return x + MOD;
        }
        return x;
    }

    static constexpr Mint pow(Mint base, long exp) {
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

    static constexpr Mint inv(Mint x) {
        return pow(x, MOD - 2);
    }

    int x;

    constexpr Mint() : x(0) {}
    constexpr Mint(int x) : x(norm(x)) {}
    constexpr Mint(int x, bool) : x(x) {}

    friend istream& operator>>(istream& in, Mint& m) {
        int x;
        in >> x;
        m = Mint(x);
        return in;
    }

    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    friend Mint operator-(const Mint& m) {
        if (!m.x) {
            return 0;
        } else {
            return Mint(MOD - m.x, true);
        }
    }

    Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }

    Mint& operator-=(const Mint& m) {
        x -= m.x;
        if (x < 0) {
            x += MOD;
        }
        return *this;
    }

    Mint& operator*=(const Mint& m) {
        x = int(long(x) * m.x % MOD);
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
};

using mint = Mint<int(1e9 + 7)>;

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int n, m;
    cin >> n >> m;
    if (m == 2 && n >= 10) {
        cout << 2 << endl;
        return;
    }
    mint f1[n + 1];
    f1[0] = f1[1] = 1;
    for (int i = 2; i <= n; i++) {
        f1[i] = f1[i - 2] + f1[i - 1];
    }
    mint g[n + 1];
    for (int i = 1; i <= n; i++) {
        g[i] = mint::pow(f1[i], m);
    }
    if (n <= 1e4) {
        mint dp[n + 1];
        dp[1] = g[1];
        for (int i = 2; i <= n; i++) {
            dp[i] = g[i];
            for (int j = 1; j < i; j++) {
                dp[i] -= dp[j] * g[i - j];
            }
            dbg(g[i] - dp[i]);
        }
        cout << dp[n] << endl;
        return;
    }
    mint choose[m + 1][m + 1];
    for (int i = 0; i <= m; i++) {
        choose[i][0] = choose[i][i] = 1;
        for (int j = 1; j < i; j++) {
            choose[i][j] = choose[i - 1][j - 1] + choose[i - 1][j];
        }
    }
    mint dp[n + 1][m + 1];
    dp[n][m] = 1;
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 0; j <= m; j++) {
            for (int k = 0; k <= j; k++) {
                dp[i][j] += choose[j][k] * dp[i + 1][m - j + k];
            }
        }
        if (i) {
            dp[i][m] = 0;
        }
    }
    cout << dp[0][m] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}