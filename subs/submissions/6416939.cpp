#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

    long dp_mn[n + 1][m + 1], dp_mx[n + 1][m + 1];
    for (auto& a : dp_mn) {
        for (auto& b : a) {
            b = 1e18;
        }
    }
    for (auto& a : dp_mx) {
        for (auto& b : a) {
            b = -1e18;
        }
    }
    for (int i = 0; i <= n; i++) {
        dp_mn[i][0] = dp_mx[i][0] = 0;
    }
    for (int j = 0; j <= m; j++) {
        dp_mn[0][j] = dp_mx[0][j] = 0;
    }
    for (int i = 1; i <= n; i++) {
        for (int j = 1; j <= m; j++) {
            if (i % 2 == 0) {
                int nj = j - i / 2;
                if (nj >= 0) {
                    dp_mn[i][j] = min(dp_mn[i][j], dp_mn[i][nj] + 1);
                    dp_mx[i][j] = max(dp_mx[i][j], dp_mx[i][nj] + 1);
                }
            }
            if (j % 2 == 0) {
                int ni = i - j / 2;
                if (ni >= 0) {
                    dp_mn[i][j] = min(dp_mn[i][j], dp_mn[ni][j] + 1);
                    dp_mx[i][j] = max(dp_mx[i][j], dp_mx[ni][j] + 1);
                }
            }
        }
    }
    assert(0 <= dp_mx[n][m] && dp_mx[n][m] <= n * m);
    cout << dp_mn[n][m] << " " << dp_mx[n][m] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}