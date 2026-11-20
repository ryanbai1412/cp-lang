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

long sq(long x) {
    return x * x;
}

void solve() {
    int n, tc_type;
    cin >> n >> tc_type;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int m = accumulate(arr, arr + n, 0);
    long dp[n + 1][m + 1] {};
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 0; j <= m; j++) {
            dp[i][j] = 1e18;
            int csm = 0;
            for (int k = i; k < n; k++) {
                csm += arr[k];
                if (csm >= j) {
                    dp[i][j] = min(dp[i][j], dp[k + 1][csm] + sq(csm));
                }
            }
        }
    }
    cout << dp[0][0] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}