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
    int dp[n][m][m][m] {};
    for (int i = n - 1; i >= 0; i--) {
        for (int a = m - 1; a >= 0; a--) {
            for (int bl = m - 1; bl > a; bl--) {
                for (int br = m - 1; br >= bl; br--) {
                    auto& mdp = dp[i][a][bl][br];
                    if (br + 1 < m) {
                        mdp = max(mdp, arr[i][br + 1] + dp[i][a][bl][br + 1]);
                    }
                    if (a + 1 < bl) {
                        mdp = max(mdp, arr[i][a + 1] + dp[i][a + 1][bl][br]);
                    }
                    if (i == n - 1 && bl != m - 1) {
                        mdp = -1e9;
                    }
                    if (i + 1 < n) {
                        mdp = max(mdp, arr[i + 1][a] + arr[i + 1][br] +
                                           dp[i + 1][a][br][br]);
                    }
                }
            }
        }
    }
    dbg(dp[1][0][2][2]);
    cout << arr[0][1] + dp[0][0][1][1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}