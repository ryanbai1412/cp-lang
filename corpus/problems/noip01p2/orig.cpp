// corpus/submissions/5086680.cpp with each zero-initialised VLA cleared by memset: clang rejects `T a[n] {}`.
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
    long dp[m + 1][n + 1][n + 1];
    memset(dp, 0, sizeof dp);
    fill(dp[0][0], dp[0][0] + n + 1, 1);
    for (int i = 1; i <= m; i++) {
        for (int j = 0; j <= n; j++) {
            for (int k = 0; k <= n; k++) {
                for (int a = 0; j - a >= k; a++) {
                    dp[i][j][k] += dp[i - 1][a][j - a];
                }
            }
        }
    }
    cout << dp[m][n][1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}