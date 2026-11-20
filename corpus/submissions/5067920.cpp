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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int n;
    cin >> n;
    int arr[n][n];
    vector<int> weights;
    memset(arr, -1, sizeof(arr));
    while (true) {
        int x, y, z;
        cin >> x >> y >> z;
        if (!x && !y && !z) {
            break;
        }
        x--;
        y--;
        arr[x][y] = sz(weights);
        weights.push_back(z);
    }
    int m = sz(weights);
    vector<int> dp[n][n];
    dp[n - 1][n - 1] = {0};
    for (int i = n - 1; i >= 0; i--) {
        for (int j = n - 1; j >= 0; j--) {
            auto& cdp = dp[i][j];
            auto append = [&](const vector<int>& v) -> void {
                cdp.insert(cdp.end(), begin(v), end(v));
            };
            if (i + 1 < n) {
                append(dp[i + 1][j]);
            }
            if (j + 1 < n) {
                append(dp[i][j + 1]);
            }
            sort(begin(dp[i][j]), end(dp[i][j]));
            dp[i][j].erase(unique(begin(dp[i][j]), end(dp[i][j])),
                           dp[i][j].end());
            if (arr[i][j] != -1) {
                for (auto& a : dp[i][j]) {
                    a |= 1 << arr[i][j];
                }
            }
        }
    }
    bool vis[1 << m] {};
    for (auto& a : dp[0][0]) {
        vis[a] = true;
    }
    int ia[1 << m] {}, dpa[1 << m] {};
    for (int a = 0; a < (1 << m); a++) {
        for (int i = 0; i < m; i++) {
            if (on(a, i)) {
                ia[a] += weights[i];
            }
        }
        if (vis[a]) {
            dpa[a] = ia[a];
        }
    }
    for (int i = (1 << m) - 1; i >= 0; i--) {
        for (int j = 0; j < m; j++) {
            if (!on(i, j)) {
                vis[i] |= vis[i ^ (1 << j)];
            }
        }
    }
    for (int i = 0; i < (1 << m); i++) {
        for (int j = 0; j < m; j++) {
            if (on(i, j)) {
                dpa[i] = max(dpa[i], dpa[i ^ (1 << j)]);
            }
        }
    }
    int ans = 0;
    for (int i = 0; i < (1 << m); i++) {
        if (vis[i]) {
            ans = max(ans, ia[i] + dpa[((1 << m) - 1) ^ i]);
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}