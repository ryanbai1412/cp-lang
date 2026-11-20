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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int dp[n + 1][m + 1];
    pair<int, int> opt[n + 1][m + 1];
    memset(dp, 0x3f, sizeof(dp));
    dp[n][0] = 0;
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 1; j <= m; j++) {
            for (int a = i; a < n; a++) {
                int dist = 0;
                for (int b = i; b < n; b++) {
                    dist += abs(arr[a] - arr[b]);
                    if (b >= a) {
                        int x = dist + dp[b + 1][j - 1];
                        if (x < dp[i][j]) {
                            dp[i][j] = x;
                            opt[i][j] = {a, b};
                        }
                    }
                }
            }
        }
    }
    cout << dp[0][m] << endl;
    int i = 0, j = m;
    vector<int> ans;
    while (i < n) {
        auto [a, b] = opt[i][j];
        ans.push_back(a);
        i = b + 1;
        j--;
    }
    for (int i = 0; i < m; i++) {
        cout << arr[ans[i]] << " \n"[i == m - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}