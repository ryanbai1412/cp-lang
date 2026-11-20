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
    int n, m;
    cin >> n >> m;
    int arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    vector<vector<int>> psum(n + 1, vector<int>(m + 1));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            psum[i + 1][j + 1] =
                psum[i + 1][j] + psum[i][j + 1] - psum[i][j] + arr[i][j];
        }
    }
    auto query = [&](int x1, int y1, int x2, int y2) -> int {
        x2++;
        y2++;
        return psum[x2][y2] - psum[x1][y2] - psum[x2][y1] + psum[x1][y1];
    };
    int dp[n][n][m][m];
    for (int x1 = n - 1; x1 >= 0; x1--) {
        for (int x2 = x1; x2 < n; x2++) {
            for (int y1 = m - 1; y1 >= 0; y1--) {
                for (int y2 = y1; y2 < m; y2++) {
                    auto& mdp = dp[x1][x2][y1][y2];
                    if (tie(x1, y1) == tie(x2, y2)) {
                        mdp = 0;
                        continue;
                    }
                    mdp = 1e9;
                    for (int x = x1; x < x2; x++) {
                        mdp =
                            min(mdp, dp[x1][x][y1][y2] + dp[x + 1][x2][y1][y2]);
                    }
                    for (int y = y1; y < y2; y++) {
                        mdp =
                            min(mdp, dp[x1][x2][y1][y] + dp[x1][x2][y + 1][y2]);
                    }
                    mdp += query(x1, y1, x2, y2);
                }
            }
        }
    }
    cout << dp[0][n - 1][0][m - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}