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
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    sort(arr, arr + n);
    int dp[n + 1][m + 1], nxt[n + 1][m + 1];
    memset(dp, 0x3f, sizeof(dp));
    fill(dp[n], dp[n] + m + 1, 0);
    for (int i = n - 1; i >= 0; i--) {
        for (int j = i; j < n; j++) {
            int x = arr[(i + j) / 2], ans = 0;
            for (int k = i; k <= j; k++) {
                ans += abs(arr[k] - x);
            }
            for (int k = 0; k < m; k++) {
                int cv = dp[j + 1][k + 1] + ans;
                if (cv < dp[i][k]) {
                    dp[i][k] = cv;
                    nxt[i][k] = j;
                }
            }
        }
    }
    cout << dp[0][0] << endl;
    vector<int> ans;
    int i = 0, k = 0;
    while (i < n) {
        int j = nxt[i][k];
        ans.push_back(arr[(i + j) / 2]);
        i = j + 1;
        k++;
    }
    for (int j = 0; j < sz(ans); j++) {
        cout << ans[j] << " \n"[j == sz(ans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}