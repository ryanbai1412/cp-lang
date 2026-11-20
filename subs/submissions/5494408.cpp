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

constexpr int maxn = 25, maxm = 255;

int dp1[maxn][maxn][maxm], dp2[maxn][maxn][maxm];

void solve() {
    string s;
    int kv;
    cin >> s >> kv;
    int n = sz(s), arr[n][n];
    for (int i = n - 1; i >= 0; i--) {
        arr[i][i] = int(s[i] - '0');
        for (int j = i + 1; j < n; j++) {
            arr[i][j] = min(maxm, arr[i][j - 1] * 10 + int(s[j] - '0'));
        }
    }
    memset(dp1, 0x3f, sizeof(dp1));
    for (int l = n - 1; l >= 0; l--) {
        for (int r = l; r < n; r++) {
            auto upd = [&](int x, int v) -> void {
                if (x < maxm) {
                    dp1[l][r][x] = min(dp1[l][r][x], v);
                }
            };
            upd(arr[l][r], 0);
            for (int i = l; i < r; i++) {
                for (int j = 0; j < maxm; j++) {
                    upd(arr[l][i] * j, 1 + dp1[i + 1][r][j]);
                }
            }
        }
    }
    memset(dp2, 0x3f, sizeof(dp2));
    for (int l = n - 1; l >= 0; l--) {
        for (int r = l; r < n; r++) {
            auto upd = [&](int x, int v) -> void {
                if (x < maxm) {
                    dp2[l][r][x] = min(dp2[l][r][x], v);
                }
            };
            for (int i = 0; i < maxm; i++) {
                upd(i, dp1[l][r][i]);
            }
            for (int i = l; i < r; i++) {
                for (int va = 0; va < maxm; va++) {
                    for (int vb = 0; vb < maxm; vb++) {
                        upd(va + vb, dp2[l][i][va] + 1 + dp2[i + 1][r][vb]);
                        if (va == 1 && vb == 1) {
                            dbg(va, vb, dp2[l][i][va], dp2[i + 1][r][vb]);
                            dbg(dp2[0][1][2]);
                        }
                    }
                }
            }
        }
    }
    int ans = dp2[0][n - 1][kv];
    if (ans >= int(1e9)) {
        cout << -1 << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}