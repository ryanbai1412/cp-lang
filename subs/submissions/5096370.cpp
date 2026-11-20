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
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int psum[n + 1];
    psum[0] = 0;
    partial_sum(arr, arr + n, psum + 1);
    auto sum = [&](int l, int r) -> int {
        return psum[r + 1] - psum[l];
    };
    int avg = sum(0, n - 1) / n, dp[n][n];
    memset(dp, 0x3f, sizeof(dp));
    for (int l = n - 1; l >= 0; l--) {
        for (int r = l; r < n; r++) {
            if (sum(l, r) == avg * (r - l + 1)) {
                dp[l][r] = min(dp[l][r], r - l);
            }
            for (int i = l; i < r; i++) {
                dp[l][r] = min(dp[l][r], dp[l][i] + dp[i + 1][r]);
            }
        }
    }
    cout << dp[0][n - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}