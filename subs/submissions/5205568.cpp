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
    int n, m, kv;
    cin >> n >> m >> kv;
    int arr[n][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    int pipes[n][2];
    memset(pipes, -1, sizeof(pipes));
    for (int i = 0; i < kv; i++) {
        int ind, l, r;
        cin >> ind >> l >> r;
        pipes[ind][0] = l;
        pipes[ind][1] = r;
    }
    int dp[n + 1][m + 1];
    memset(dp, 0x3f, sizeof(dp));
    memset(dp[0], 0, sizeof(dp[0]));
    for (int i = 0; i < n; i++) {
        dp[i][0] = 1e9;
        if (pipes[i][0] != -1) {
            auto& [l, r] = pipes[i];
            for (int j = 0; j <= l; j++) {
                dp[i][j] = 1e9;
            }
            for (int j = r; j <= m; j++) {
                dp[i][j] = 1e9;
            }
        }
        auto& [x, y] = arr[i];
        for (int j = 0; j <= m; j++) {
            if (j - y >= 0) {
                dp[i + 1][j - y] = min(dp[i + 1][j - y], dp[i][j]);
            }
            for (int k = j + x, cv = 1; ; k += x, cv++) {
                k = min(k, m);
                dp[i + 1][k] = min(dp[i + 1][k], dp[i][j] + cv);
                if (k == m) {
                    break;
                }
            }
        }
    }
    dp[n][0] = 1e9;
    dbg(dp[8][8]);
    int ans = *min_element(dp[n], dp[n] + m + 1);
    if (ans < 1e9) {
        cout << 1 << endl;
        cout << ans << endl;
    } else {
        cout << 0 << endl;
        int vals[n];
        for (int i = 0; i < n; i++) {
            vals[i] = i ? vals[i - 1] : 0;
            if (pipes[i][0] != -1) {
                vals[i]++;
            }
        }
        for (int i = n - 1; i >= 0; i--) {
            if (*min_element(dp[i], dp[i] + m + 1) < 1e9) {
                cout << vals[i] << endl;
                return;
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}