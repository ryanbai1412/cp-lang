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

long solve(const vector<int>& arr) {
    int n = sz(arr);
    long dp[n][n] {};
    for (int l = n - 1; l >= 0; l--) {
        dp[l][l] = arr[l];
        for (int r = l + 1; r < n; r++) {
            dp[l][r] =
                max(arr[l] + 2 * dp[l + 1][r], arr[r] + 2 * dp[l][r - 1]);
        }
    }
    return 2 * dp[0][n - 1];
}

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    long ans = 0;
    for (auto& a : arr) {
        ans += solve(vector<int>(a, a + m));
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}