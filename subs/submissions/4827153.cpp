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
    int n, m, k;
    cin >> n >> m >> k;
    int arr[n][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    int lim[n][2];
    memset(lim, -1, sizeof(lim));
    for (int i = 0; i < k; i++) {
        int x, l, r;
        cin >> x >> l >> r;
        lim[x][0] = l;
        lim[x][1] = r;
    }
    int dp[n + 1][m + 1], passed = 0;
    memset(dp, 0x3f, sizeof(dp));
    fill(dp[0] + 1, dp[0] + m + 1, 0);
    for (int i = 0; i < n; i++) {
        dp[i][0] = 1e9;
        if (lim[i][0] != -1) {
            for (int j = 0; j <= m; j++) {
                if (j <= lim[i][0] || lim[i][1] <= j) {
                    dp[i][j] = 1e9;
                }
            }
        }
        if (*min_element(dp[i], dp[i] + m + 1) >= 1e9) {
            cout << 0 << endl;
            cout << passed << endl;
            return;
        }
        passed += lim[i][0] != -1;
        auto& [x, y] = arr[i];
        for (int j = 1; j <= m; j++) {
            dp[i + 1][min(m, j + x)] = min(dp[i + 1][min(m, j + x)], dp[i][j] + 1);
        }
        for (int j = x + 1; j <= m; j++) {
            dp[i + 1][j] = min(dp[i + 1][j], dp[i + 1][j - x] + 1);
        }
        for (int j = y + 1; j <= m; j++) {
            dp[i + 1][j - y] = min(dp[i + 1][j - y], dp[i][j]);
        }
    }
    if (*min_element(dp[n], dp[n] + m + 1) >= 1e9) {
        cout << 0 << endl;
        cout << passed << endl;
        return;
    }
    cout << 1 << endl;
    cout << *min_element(dp[n], dp[n] + m + 1) << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}