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

template <int MOD>
struct Mint {
    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
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
        return Mint::pow(x, MOD - 2);
    }

    int x;

    constexpr Mint() : x(0) {}
    constexpr Mint(long x) : x(norm(x)) {}

    static constexpr Mint from_unchecked(int x) {
        Mint m;
        m.x = x;
        return m;
    }

    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    constexpr Mint operator-() const {
        if (!x) {
            return {};
        }
        return Mint::from_unchecked(MOD - x);
    }

    constexpr Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr Mint& operator-=(const Mint& m) {
        return *this += -m;
    }

    constexpr Mint& operator*=(const Mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }
    constexpr Mint& operator/=(const Mint& m) {
        return *this *= Mint::inv(m);
    }

    constexpr friend Mint operator+(Mint a, const Mint& b) {
        return a += b;
    }
    constexpr friend Mint operator-(Mint a, const Mint& b) {
        return a -= b;
    }
    constexpr friend Mint operator*(Mint a, const Mint& b) {
        return a *= b;
    }
    constexpr friend Mint operator/(Mint a, const Mint& b) {
        return a /= b;
    }
};

constexpr int MOD = 998244353;
using mint = Mint<MOD>;

mint solve(int n, int kv, mint p) {
    mint dp[n + 1][kv + 1] {};
    fill(dp[0], dp[0] + kv + 1, 1);

    for (int len = 1; len <= n; len++) {
        for (int h = kv / len + 2; h >= 0; h--) {
            if (h * len >= kv) {
                continue;
            }
            mint cp = 1;

            for (int i = 0; i < len; i++) {
                dp[len][h] += cp * (1 - p) * dp[i][h + 1] * dp[len - i - 1][h];
                cp *= p;
            }

            dp[len][h] += cp * dp[len][h + 1];
        }
    }

    return dp[n][0];
}

void solve() {
    int n, kv, px, py;
    cin >> n >> kv >> px >> py;
    mint p = mint(px) / py;

    cout << solve(n, kv + 1, p) - solve(n, kv, p) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}