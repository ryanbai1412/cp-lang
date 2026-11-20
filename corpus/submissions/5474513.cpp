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

constexpr double eps = 1e-6;

void solve() {
    int m, n, q = 10;
    cin >> m >> n;
    double dp[(m - 1) * n + 5 * m] {};
    dp[0] = 1;
    int cl = 0, cr = 0;
    for (int iter = 0; iter < n; iter++) {
        for (int i = cr + m - 1; i >= cl; i--) {
            dp[i] =
                accumulate(dp + max(0, i - m + 1), dp + i + 1, double(0)) / m;
        }
        for (; dp[cr] > eps; cr++)
            ;
        for (; dp[cl] < eps; cl++)
            ;
    }
    double dpsum[(m - 1) * n + 1];
    dpsum[0] = 0;
    partial_sum(dp, dp + (m - 1) * n, dpsum + 1);
    while (q--) {
        int ql, qr;
        cin >> ql >> qr;
        cout << dpsum[qr + 1] - dpsum[ql] << endl;
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