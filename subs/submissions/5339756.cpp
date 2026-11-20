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

void solve() {
    string s;
    int kv;
    cin >> s >> kv;
    int n = sz(s), psum[n + 1], dp[n][n] {};
    psum[0] = 0;
    for (int i = 0; i < n; i++) {
        psum[i + 1] = psum[i] + (s[i] == 'H');
    }
    auto sum_h = [&](int l, int r) -> int {
        return psum[r + 1] - psum[l];
    };
    for (int l = n - 1; l >= 0; l--) {
        for (int r = l; r < n; r++) {
            for (int i = l; i <= r; i++) {
                if (s[i] != 'H') {
                    continue;
                }
                int cans = r - l + 1;
                if (i > l) {
                    cans += dp[l][i - 1];
                }
                if (i < r) {
                    cans += dp[i + 1][r];
                }
                dp[l][r] = max(dp[l][r], cans);
            }
        }
    }
    int dp1[n + 1][kv + 1] {};
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 0; j <= kv; j++) {
            dp1[i][j] = dp1[i + 1][j];
            for (int k = i; k < n; k++) {
                int nj = j + sum_h(i, k);
                if (nj <= kv) {
                    dp1[i][j] = max(dp1[i][j], dp[i][k] + dp1[k + 1][nj]);
                }
            }
        }
    }
    cout << dp1[0][0] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}