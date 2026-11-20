#pragma GCC optimize("Ofast","unroll-loops")
#pragma GCC target("avx2")

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

constexpr int maxn = 205, maxk = 5;

int arr[maxn][maxn];
long dp[maxn][maxn][maxk + 1];

void solve(int n, int m) {
    memset(dp[n - 1], 0x3f, sizeof(dp[n - 1]));
    memset(dp[n - 1][m - 1], 0, sizeof(dp[n - 1][m - 1]));
    dp[n - 1][m - 1][0] = arr[n - 1][m - 1];
    for (int i = n - 1; i >= 0; i--) {
        if (i + 1 < n) {
            for (int j = 0; j < m; j++) {
                for (int k = 0; k <= maxk; k++) {
                    dp[i][j][k] = arr[i][j] + dp[i + 1][j][k];
                }
                for (int k = 1; k <= maxk; k++) {
                    dp[i][j][k] = min(dp[i][j][k], dp[i + 1][j][k - 1]);
                }
            }
        }
        for (int j = m - 2; j >= 0; j--) {
            for (int k = 0; k <= maxk; k++) {
                dp[i][j][k] = min(dp[i][j][k], arr[i][j] + dp[i][j + 1][k]);
            }
            for (int k = 1; k <= maxk; k++) {
                dp[i][j][k] = min(dp[i][j][k], dp[i][j + 1][k - 1]);
            }
        }
    }
}

void solve() {
    int n, m, q;
    cin >> n >> m >> q;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            cin >> arr[i][j];
        }
    }
    long qans[q];
    vector<array<int, 4>> queries[n][m];
    for (int i = 0; i < q; i++) {
        int x1, y1, x2, y2, kv;
        cin >> x1 >> y1 >> x2 >> y2 >> kv;
        x1--;
        y1--;
        x2--;
        y2--;
        queries[x2][y2].push_back({x1, y1, kv, i});
    }
    for (int x2 = 0; x2 < n; x2++) {
        for (int y2 = 0; y2 < m; y2++) {
            solve(x2 + 1, y2 + 1);
            for (auto& [x1, y1, kv, qi] : queries[x2][y2]) {
                qans[qi] = dp[x1][y1][kv];
            }
        }
    }
    for (auto& a : qans) {
        cout << a << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}