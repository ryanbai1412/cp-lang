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

struct mint {
    static constexpr int MOD = 998244353;

    static constexpr int norm(long x) {
        if (0 <= x && x < MOD) {
            return int(x);
        } else if (-x <= MOD && x < 0) {
            return int(x + MOD);
        }
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    static mint pow(mint base, long exp) {
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

    static mint inv(mint x) {
        return pow(x, MOD - 2);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    static mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    mint operator-() const {
        if (!x) {
            return {};
        }
        return mint::from_unchecked(MOD - x);
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
        return *this *= mint::inv(m);
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
    static constexpr int MAXN = 2e6 + 5;

    mint fact[MAXN], ifact[MAXN];

    M() {
        fact[0] = ifact[0] = 1;
        for (int i = 1; i < MAXN; i++) {
            fact[i] = i * fact[i - 1];
        }
        ifact[MAXN - 1] = mint::inv(fact[MAXN - 1]);
        for (int i = MAXN - 1; i >= 1; i--) {
            ifact[i - 1] = ifact[i] * i;
        }
    }

    mint choose(int n, int k) const {
        return fact[n] * ifact[k] * ifact[n - k];
    }

    mint choose2(int n, int k) const {
        if (n < 0 || k < 0 || k > n) {
            return 0;
        }
        return choose(n, k);
    }
} M;

mint ways1(int x, int y) {
    return M.choose2(x + y, x);
}

mint ways2(int x, int y) {
    if (x > y) {
        swap(x, y);
    }
    return ways1(x, y) - ways1(y + 1, x - 1);
}

mint cdp(int n, int x, int y) {
    x = n - x;
    y = n - y - 1;
    return ways2(x, y);
}

void solve() {
    int n;
    cin >> n;

    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    mint dp[n + 1][n + 1] {};
    dp[n][n - 1] = 1;

    for (int i = n - 1; i >= 1; i--) {
        for (int x = n - 1; x >= i - 1; x--) {
            dp[i][x] = dp[i + 1][x] + dp[i][x + 1];
            // dp[i][x] = accumulate(dp[i + 1] + x, dp[i + 1] + n, 0);
            dbg(i, x, dp[i][x], cdp(n, i, x));
        }
    }

    set<int> small;
    for (int i = 0; i < n; i++) {
        small.insert(i);
    }

    mint ans = 0;
    int cmx = -1;
    for (int i = 0; i < n; i++) {
        dbg(i, cmx, ans);
        if (i) {
            ans += cdp(n, i, max(cmx + 1, arr[i] + 1));
            // ans += dp[i][max(cmx + 1, arr[i] + 1)];
        } else {
            for (int j = max(cmx + 1, arr[i] + 1); j < n ; j++) {
                ans += cdp(n, i + 1, j);
            }
            // ans += accumulate(dp[i + 1] + max(cmx + 1, arr[i] + 1), dp[i + 1] + n,
            //                   mint(0));
        }
        // ans += accumulate(dp[i + 1] + max(cmx + 1, arr[i] + 1), dp[i + 1] + n,
        //                   mint(0));
        if (*begin(small) > arr[i]) {
            ans += dp[i + 1][cmx];
        }

        if (arr[i] > cmx) {
            cmx = arr[i];
        } else if (*begin(small) != arr[i]) {
            break;
        }
        small.erase(arr[i]);
    }

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}