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
    int n;
    string s;
    cin >> n >> s;
    int dp[2][n];
    int cind = 0, pind = 1;
    for (int i = n - 1; i >= 0; i--) {
        swap(cind, pind);
        dp[cind][i] = 0;
        if (i + 1 < n) {
            dp[pind][i] = 0;
        }
        for (int j = i + 1; j < n; j++) {
            dp[cind][j] = 1 + min(dp[pind][j], dp[cind][j - 1]);
            if (s[i] == s[j]) {
                dp[cind][j] = min(dp[cind][j], dp[pind][j - 1]);
            }
        }
    }
    cout << dp[cind][n - 1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}