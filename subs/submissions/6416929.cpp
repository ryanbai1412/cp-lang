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

struct A {
    long mn, mx;

    A& operator+=(const A& o) {
        mn = min(mn, o.mn);
        mx = max(mx, o.mx);
        return *this;
    }

    A& operator+=(long x) {
        mn += x;
        mx += x;
        return *this;
    }

    friend A operator+(A a, const A& b) {
        return a += b;
    }
    friend A operator+(A a, long b) {
        return a += b;
    }
    friend A operator+(long b, A a) {
        return a + b;
    }

    static A c_def() {
        return {long(1e18), long(-1e18)};
    }

    static A c_zero() {
        return {};
    }
};

void solve() {
    int n, m;
    cin >> n >> m;

    A dp[n + 1][m + 1];
    for (auto& a : dp) {
        for (auto& b : a) {
            b = A::c_def();
        }
    }
    for (int i = 0; i <= n; i++) {
        dp[i][0] = A::c_zero();
    }
    for (int j = 0; j <= m; j++) {
        dp[0][j] = A::c_zero();
    }
    for (int i = 1; i <= n; i++) {
        for (int j = 1; j <= m; j++) {
            if (i % 2 == 0) {
                int nj = j - i / 2;
                if (nj >= 0) {
                    dp[i][j] += dp[i][nj] + 1;
                }
            }
            if (j % 2 == 0) {
                int ni = i - j / 2;
                if (ni >= 0) {
                    dp[i][j] += dp[ni][j] + 1;
                }
            }
        }
    }
    cout << dp[n][m].mn << " " << dp[n][m].mx << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}