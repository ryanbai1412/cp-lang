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
    int n, kv;
    cin >> n >> kv;
    pair<int, int> arr[n];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    sort(arr, arr + n);
    int xs[n];
    vector<int> ys {0};
    for (int i = 0; i < n; i++) {
        xs[i] = arr[i].first;
        ys.push_back(arr[i].second);
    }
    sort(begin(ys), end(ys));
    ys.erase(unique(begin(ys), end(ys)), ys.end());
    int m = sz(ys);
    int dp[n + 1][n][m];
    memset(dp, 0x3f, sizeof(dp));
    for (int i = 0; i < n; i++) {
        fill(dp[i + 1][i], dp[i + 1][i] + m, 0);
    }
    for (int l = n - 1; l >= 0; l--) {
        for (int r = l; r < n; r++) {
            for (int k = m - 1; k >= 0; k--) {
                auto& mdp = dp[l][r][k];
                if (arr[l].second <= ys[k]) {
                    mdp = dp[l + 1][r][k];
                    continue;
                } else if (arr[r].second <= ys[k]) {
                    mdp = dp[l][r - 1][k];
                    continue;
                }
                for (int i = l; i <= r; i++) {
                    int cy = xs[i] == xs[l] ? m - 1 : kv / (xs[i] - xs[l]);
                    int yind =
                        int(upper_bound(begin(ys), end(ys), cy) - ys.begin()) -
                        1;
                    assert(ys[yind] * (xs[i] - xs[l]) <= kv && yind + 1 == m ||
                           ys[yind] * (xs[i] - xs[l]) > kv);
                    mdp = min(mdp, dp[l][i][yind] + dp[i + 1][r][k] + 1);
                }
            }
        }
    }
    dbg(dp[0][0][0]);
    cout << dp[0][n - 1][0] << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}