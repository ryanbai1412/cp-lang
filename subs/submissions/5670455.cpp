#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

long eval(int n, int m, int g) {
    long ans = 0;

    vector<int> arra, arrb;

    for (int i = g; i <= n; i += g) {
        arra.push_back(n / i);
    }
    for (int j = g; j <= m; j += g) {
        arrb.push_back(m / j);
    }

    map<int, pair<int, int>> mp;
    for (auto& a : arra) {
        mp[-a].first++;
    }
    for (auto& b : arrb) {
        mp[-b].second++;
    }

    long pa = 0, pb = 0;
    for (auto& [nx, p] : mp) {
        auto [cnta, cntb] = p;
        long x = -nx;

        ans += cntb * pa * x * x;
        pb += cntb;
        ans += cnta * pb * x * x;
        pa += cnta;
    }

    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;

    long dp[n + 1] {};

    for (int i = n; i >= 1; i--) {
        dp[i] = eval(n, m, i);
        for (int j = 2 * i; j <= n; j += i) {
            dp[i] -= dp[j];
        }
    }

    cout << dp[1] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}