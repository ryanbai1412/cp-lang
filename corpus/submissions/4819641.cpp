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
    int n;
    cin >> n;
    string s;
    cin >> s;
    int dp[n][n];
    memset(dp, 0x3f, sizeof(dp));
    for (int i = 0; i < n - 1; i++) {
        dp[i + 1][i] = 0;
    }
    for (int i = n - 1; i >= 0; i--) {
        dp[i][i] = 0;
        for (int j = i + 1; j < n; j++) {
            dp[i][j] = min(dp[i + 1][j], dp[i][j - 1]) + 1;
            if (s[i] == s[j]) {
                dp[i][j] = min(dp[i][j], dp[i + 1][j - 1]);
            }
        }
    }
    cout << dp[0][n - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}