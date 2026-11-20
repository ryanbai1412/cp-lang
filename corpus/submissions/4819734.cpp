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

constexpr int mod = 1e9 + 7;

struct mint {
    static int norm(long x) {
        if (x >= mod) {
            x -= mod;
        }
        if (x < 0) {
            x += mod;
        }
        if (0 <= x && x < mod) {
            return int(x);
        }
        return int(x % mod);
    }

    int m;

    mint() : m(0) {}
    mint(int x) : m(norm(x)) {}
    mint(long x) : m(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& x) {
        return out << x.m;
    }

    mint& operator+=(const mint& x) {
        m += x.m;
        if (m >= mod) {
            m -= mod;
        }
        return *this;
    }

    mint& operator-=(const mint& x) {
        m -= x.m;
        if (m < 0) {
            m += mod;
        }
        return *this;
    }

    mint& operator*=(const mint& x) {
        m = int((long(m) * x.m) % mod);
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

void solve() {
    int n, k;
    cin >> n >> k;
    string s;
    cin >> s;
    mint dp1[n][n], dp2[n][n], dp3[n][n];
    auto eq = [&](char c1, char c2) -> bool {
        return c1 == '?' || c2 == '?' || c1 == c2;
    };
    for (int l = n - 1; l >= 0; l--) {
        for (int r = l + 1; r < n; r++) {
            if (eq(s[l], '(') && eq(s[r], ')')) {
                if (r - l - 1 <= k) {
                    bool b = true;
                    for (int i = l + 1; i < r; i++) {
                        b &= eq(s[i], '*');
                    }
                    dp1[l][r] += b;
                }
                for (int i = l + 1; i < r && i - l - 1 <= k; i++) {
                    dp1[l][r] += dp2[i][r - 1];
                    if (!eq(s[i], '*')) {
                        break;
                    }
                }
                if (l == 0 && r == n - 1) {
                    dbg(dp1[l][r]);
                }
                for (int i = r - 1; i > l && r - i - 1 <= k; i--) {
                    if (i < r - 1) {
                        dp1[l][r] += dp2[l + 1][i];
                    }
                    if (!eq(s[i], '*')) {
                        break;
                    }
                }
            }
            for (int j = l; j < r && j - l <= k; j++) {
                if (l < j && !eq(s[j], '*')) {
                    break;
                }
                dp3[l][r] += dp2[j + 1][r];
            }
            dp2[l][r] = dp1[l][r];
            for (int i = l; i < r; i++) {
                dp2[l][r] += dp1[l][i] * dp3[i][r];
//                for (int j = i; j < r && j - i <= k; j++) {
//                    if (i < j && !eq(s[j], '*')) {
//                        break;
//                    }
//                    dp2[l][r] += dp1[l][i] * dp2[j + 1][r];
//                }
            }
        }
    }
    dbg(dp1[0][n - 1]);
    cout << dp2[0][n - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}