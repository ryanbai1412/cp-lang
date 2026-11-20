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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int m;
    string s;
    cin >> m >> s;
    int n = sz(s), arr[n];
    for (int i = 0; i < n; i++) {
        arr[i] = s[i] - 'a';
    }
    if (m >= 22) {
        cout << "NO" << endl;
        return;
    }
    int prv[n + 1][m];
    memset(prv, -1, sizeof(prv));
    for (int i = 0; i < n; i++) {
        memcpy(prv[i + 1], prv[i], sizeof(prv[i]));
        prv[i + 1][arr[i]] = i;
    }
    int dp[1 << m];
    memset(dp, -1, sizeof(dp));
    dp[0] = n;
    for (int i = 0; i < (1 << m); i++) {
        dp[i] = n;
        for (int j = 0; j < m; j++) {
            if (!on(i, j)) {
                continue;
            }
            int cdp = dp[i ^ (1 << j)];
            if (cdp == -1) {
                dp[i] = -1;
            } else {
                dp[i] = min(dp[i], prv[cdp][j]);
            }
        }
    }
    if (dp[(1 << m) - 1] != -1) {
        cout << "YES" << endl;
    } else {
        cout << "NO" << endl;
    }
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