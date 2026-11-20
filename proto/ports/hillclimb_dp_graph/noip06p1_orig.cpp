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
    long arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    auto solve = [&]() -> long {
        long carr[n][2];
        for (int i = 0; i < n; i++) {
            carr[i][0] = arr[i];
            carr[i][1] = arr[(i + 1) % n];
        }
        long dp[n][n] {};
        for (int l = n - 1; l >= 0; l--) {
            for (int r = l + 1; r < n; r++) {
                for (int i = l; i < r; i++) {
                    dp[l][r] = max(dp[l][r], dp[l][i] + dp[i + 1][r] +
                                                 carr[l][0] * carr[i][1] * carr[r][1]);
                }
            }
        }
        return dp[0][n - 1];
    };
    long ans = 0;
    for (int i = 0; i < n; i++) {
        rotate(arr, arr + 1, arr + n);
        ans = max(ans, solve());
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}