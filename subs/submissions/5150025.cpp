#pragma GCC optimize("Ofast")
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

constexpr int maxn = 200, maxk = 5;

int arr[maxn][maxn];
long dp[maxn][maxn][maxk + 1], dp2[maxn][maxn][maxk + 1];

namespace s1 {

long solve(const vector<vector<int>>& arr, int kv) {
    int n = sz(arr), m = sz(arr[0]);
    long dp[n][m][kv + 1];
    for (int i = n - 1; i >= 0; i--) {
        for (int j = m - 1; j >= 0; j--) {
            if (i == n - 1 && j == m - 1) {
                fill(dp[i][j], dp[i][j] + kv, 0);
                dp[i][j][kv] = arr[i][j];
                continue;
            }
            for (int k = 0; k <= kv; k++) {
                dp[i][j][k] = 1e18;
                if (i + 1 < n) {
                    dp[i][j][k] = min(dp[i][j][k], arr[i][j] + dp[i + 1][j][k]);
                    if (k + 1 <= kv) {
                        dp[i][j][k] = min(dp[i][j][k], dp[i + 1][j][k + 1]);
                    }
                }
                if (j + 1 < m) {
                    dp[i][j][k] = min(dp[i][j][k], arr[i][j] + dp[i][j + 1][k]);
                    if (k + 1 <= kv) {
                        dp[i][j][k] = min(dp[i][j][k], dp[i][j + 1][k + 1]);
                    }
                }
            }
        }
    }
    return dp[0][0][0];
}

long solve(int x1, int y1, int x2, int y2, int kv) {
    vector<vector<int>> carr;
    for (int i = x1; i <= x2; i++) {
        carr.emplace_back(arr[i] + y1, arr[i] + y2 + 1);
    }
    return solve(carr, kv);
}

}

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
    memset(qans, 0x3f, sizeof(qans));
    vector<array<int, 6>> queries[n][m];
    for (int i = 0; i < q; i++) {
        int x1, y1, x2, y2, kv;
        cin >> x1 >> y1 >> x2 >> y2 >> kv;
        x1--;
        y1--;
        x2--;
        y2--;
        if (x2 - x1 <= 3 && y2 - y1 <= 3) {
            qans[i] = s1::solve(x1, y1, x2, y2, kv);
            continue;
        }
        for (int dx = 0; dx <= (x2 + y2) % 3; dx++) {
            for (int dy = 0; dy < 3; dy++) {
                int cx = x2 - dx, cy = y2 - dy;
                if (x1 <= cx && y1 <= cy && (cx + cy) % 3 == 0) {
                    queries[cx][cy].push_back({x1, y1, x2, y2, kv, i});
                }
            }
        }
    }
    for (int x2 = 0; x2 < n; x2++) {
        for (int y2 = 0; y2 < m; y2++) {
            if (!sz(queries[x2][y2])) {
                continue;
            }
            for (int i = x2; i < x2 + 3 && i < n; i++) {
                for (int j = y2; j < y2 + 3 && j < m; j++) {
                    if (i == x2 && j == y2) {
                        memset(dp2[i][j], 0, sizeof(dp2[i][j]));
                        continue;
                    }
                    memset(dp2[i][j], 0x3f, sizeof(dp2[i][j]));
                    for (int k = 0; k <= maxk; k++) {
                        if (i > x2) {
                            dp2[i][j][k] = min(dp2[i][j][k], arr[i][j] + dp2[i - 1][j][k]);
                            if (k) {
                                dp2[i][j][k] = min(dp2[i][j][k], dp2[i - 1][j][k - 1]);
                            }
                        }
                        if (j > y2) {
                            dp2[i][j][k] = min(dp2[i][j][k], arr[i][j] + dp2[i][j - 1][k]);
                            if (k) {
                                dp2[i][j][k] = min(dp2[i][j][k], dp2[i][j - 1][k - 1]);
                            }
                        }
                    }
                }
            }
            solve(x2 + 1, y2 + 1);
            for (auto& [x1, y1, cx2, cy2, kv, qi] : queries[x2][y2]) {
                for (int k = 0; k <= kv; k++) {
                    qans[qi] = min(qans[qi], dp[x1][y1][k] + dp2[cx2][cy2][kv - k]);
                }
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