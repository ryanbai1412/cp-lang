#include "bits/extc++.h"

using namespace std;

template<typename T>
void dbgh(const T &t) {
    cerr << t << endl;
}

template<typename T, typename... U>
void dbgh(const T &t, const U &...u) {
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
    int n, kv;
    cin >> n >> kv;
    int arr[n];
    for (auto &a: arr) {
        cin >> a;
    }
    int dp[n + 1][kv + 1];
    memset(dp, 0x3f, sizeof(dp));
    dp[n][0] = 0;
    for (int i = n - 1; i >= 0; i--) {
        memcpy(dp[i], dp[i + 1], sizeof(dp[i]));
        if (i == n - 1) {
            continue;
        }
        for (int j = 1; j <= kv; j++) {
            dp[i][j] = min(dp[i][j], dp[i + 2][j - 1] + arr[i + 1] - arr[i]);
        }
    }
    cout << dp[0][kv] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}