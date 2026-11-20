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
    static constexpr int MOD = 998244353;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    int x;

    constexpr mint(long x = 0) : x(norm(x)) {}
    constexpr mint(int x, bool) : x(x) {}

    friend istream& operator>>(istream& in, mint& m) {
        int x;
        in >> x;
        m = x;
        return in;
    }

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    friend mint operator-(const mint& m) {
        if (!m.x) {
            return 0;
        } else {
            return mint(MOD - m.x, true);
        }
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

    friend mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

template <typename T>
struct A {
    int mn;
    vector<T> arr;

    A() {}
    A(int mn, int mx) : mn(mn), arr(mx - mn + 1) {}

    T& operator[](int i) {
        return arr[i - mn];
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    mint arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    mint sum[n];
    for (int i = 0; i < n; i++) {
        sum[i] = accumulate(arr[i], arr[i] + m, mint(0));
    }
    mint ans = 1;
    for (auto& a : sum) {
        ans *= (a + 1);
    }
    for (int col = 0; col < m; col++) {
        A<mint> dp[n + 1];
        for (auto& a : dp) {
            a = A<mint>(-n, n);
        }
        for (int i = 1; i <= n; i++) {
            dp[n][i] = 1;
        }
        for (int i = n - 1; i >= 0; i--) {
            for (int j = -n + 1; j < n; j++) {
                dp[i][j] += arr[i][col] * dp[i + 1][j + 1] +
                            (sum[i] - arr[i][col]) * dp[i + 1][j - 1] +
                            dp[i + 1][j];
            }
        }
        ans -= dp[0][0];
    }
    cout << ans - 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}