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
    int n, m;
    cin >> n >> m;
    m++;
    string s;
    cin >> s;
    long dp[n + 1][m + 1] {};
    dp[n][0] = 1;
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 1; j <= m; j++) {
            for (int k = i + 1; k <= n; k++) {
                dp[i][j] =
                    max(dp[i][j], stoi(s.substr(i, k - i)) * dp[k][j - 1]);
            }
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