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

constexpr double eps = 1e-9;

void solve() {
    int m, n, q = 10;
    cin >> m >> n;
    double dp[(m - 1) * n + 5 * m] {};
    dp[0] = 1;
    int cl = 0, cr = 0;
    bool div = n >= 1e5;
    for (int iter = 0, up = div ? n / 2 : n; iter < up; iter++) {
        double csum = 0;
        for (int i = cr + m - 1, lind = i; i >= cl; i--) {
            for (; lind > i - m && lind >= 0; lind--) {
                csum += dp[lind];
            }
            csum -= exchange(dp[i], csum / m);
        }
        for (; dp[cr] > eps; cr++)
            ;
        for (; dp[cl] < eps; cl++) {
            dp[cl] = 0;
        }
        dbg(cl, cr);
    }
    double dpsum[(m - 1) * n + 1];
    dpsum[0] = 0;
    partial_sum(dp, dp + (m - 1) * n, dpsum + 1);
    while (q--) {
        int ql, qr;
        cin >> ql >> qr;
        if (div) {
            ql /= 2;
            qr /= 2;
        }
        double ans = dpsum[qr + 1] - dpsum[ql];
        if (div) {
            ans *= sqrt(2);
        }
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(6);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}