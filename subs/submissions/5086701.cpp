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
    int vn, kv;
    cin >> vn >> kv;
    string s;
    for (int i = 0; i < vn; i++) {
        string tmp;
        while (!sz(tmp)) {
            getline(cin, tmp);
        }
        s += tmp;
    }
    int n = sz(s), m;
    cin >> m;
    string arr[m];
    for (auto& a : arr) {
        cin >> a;
    }
    int can[n];
    memset(can, 0x3f, sizeof(can));
    for (auto& a : arr) {
        for (int i = 0; i < n; i++) {
            if (s.substr(i, sz(a)) == a) {
                can[i] = min(can[i], i + sz(a) - 1);
            }
        }
    }
    int dp1[n + 1][n] {};
    for (int r = 0; r < n; r++) {
        for (int l = r - 1; l >= 0; l--) {
            dp1[l][r] = dp1[l + 1][r];
            if (can[l] <= r) {
                dp1[l][r] = max(dp1[l][r], dp1[l + 1][r] + 1);
            }
        }
    }
    int dp[n + 1][kv + 1] {};
    for (auto& a : dp) {
        fill(a, a + kv + 1, -1e9);
    }
    dp[n][0] = 0;
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 1; j <= kv; j++) {
            for (int k = i; k < n; k++) {
                dp[i][j] = max(dp[i][j], dp1[i][k] + dp[k + 1][j - 1]);
            }
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