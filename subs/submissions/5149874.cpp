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

vector<vector<vector<long>>> solve(const vector<vector<int>>& arr, int kv) {
    int n = sz(arr), m = sz(arr[0]);
    vector<vector<vector<long>>> dp(n, vector<vector<long>>(m, vector<long>(kv + 1)));
    for (int i = n - 1; i >= 0; i--) {
        for (int j = m - 1; j >= 0; j--) {
            if (i == n - 1 && j == m - 1) {
                dp[i][j][0] = arr[i][j];
                continue;
            }
            for (int k = 0; k <= kv; k++) {
                dp[i][j][k] = 1e18;
                if (i + 1 < n) {
                    dp[i][j][k] = min(dp[i][j][k], arr[i][j] + dp[i + 1][j][k]);
                    if (k) {
                        dp[i][j][k] = min(dp[i][j][k], dp[i + 1][j][k - 1]);
                    }
                }
                if (j + 1 < m) {
                    dp[i][j][k] = min(dp[i][j][k], arr[i][j] + dp[i][j + 1][k]);
                    if (k) {
                        dp[i][j][k] = min(dp[i][j][k], dp[i][j + 1][k - 1]);
                    }
                }
            }
        }
    }
    return dp;
}

void solve() {
    int n, m, q;
    cin >> n >> m >> q;
    int arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
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
            vector<vector<int>> carr;
            for (int i = 0; i <= x2; i++) {
                carr.emplace_back(arr[i] + 0, arr[i] + y2 + 1);
            }
            auto cans = solve(carr, 5);
            for (auto& [x1, y1, kv, qi] : queries[x2][y2]) {
                qans[qi] = cans[x1][y1][kv];
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