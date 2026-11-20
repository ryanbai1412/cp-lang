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
    if (!n && !m) {
        exit(0);
    }
    string iarr[n];
    for (auto& a : iarr) {
        cin >> a;
    }
    swap(n, m);
    char arr[n][m];
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            arr[i][j] = iarr[j][i];
        }
    }
    int dp[n + 1][m];
    for (auto& a : dp) {
        fill(a, a + m, -1e9);
    }
    dp[n][m - 1] = 0;
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 0; j < m; j++) {
            {
                int cvalue = 0;
                for (int k = j; k < m; k++) {
                    if (arr[i][k] == '*') {
                        break;
                    } else if (isdigit(arr[i][k])) {
                        cvalue += arr[i][k] - '0';
                    }
                    dp[i][j] = max(dp[i][j], cvalue + dp[i + 1][k]);
                }
            }
            {
                int cvalue = 0;
                for (int k = j; k >= 0; k--) {
                    if (arr[i][k] == '*') {
                        break;
                    } else if (isdigit(arr[i][k])) {
                        cvalue += arr[i][k] - '0';
                    }
                    dp[i][j] = max(dp[i][j], cvalue + dp[i + 1][k]);
                }
            }
        }
    }
    cout << dp[0][m - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    while (true) {
        solve();
    }
}