#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
    long kv;
    cin >> n >> m >> kv;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    int mxs = n * n / 2 + 10;
    long dp[m + 1][n + 1][mxs] {};
    for (int i = m - 1; i >= 0; i--) {
        for (int j = n - 1; j >= i; j--) {
            for (int k = j - i; k < mxs; k++) {
                dp[i][j][k] = max(dp[i][j + 1][k],
                                  arr[j] + dp[i + 1][j + 1][k - (j - i)]);
            }
        }
    }

    for (int i = 0; i < mxs; i++) {
        if (dp[0][0][i] >= kv) {
            cout << i << endl;
            return;
        }
    }
    cout << "NO" << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}