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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

long max_matching(const vector<vector<long>>& graph) {
    int n = sz(graph);
    long dp[1 << n];
    fill(dp, dp + (1 << n), -8e18);
    dp[(1 << n) - 1] = 0;
    for (int i = (1 << n) - 2; i >= 0; i--) {
        int u = __builtin_popcount(i);
        for (int j = 0; j < n; j++) {
            if (!on(i, j)) {
                dp[i] = max(dp[i], graph[u][j] + dp[i | (1 << j)]);
            }
        }
    }
    return dp[0];
}

void solve() {
    int n;
    cin >> n;
    vector<vector<int>> arra(n, vector<int>(n)), arrb(n, vector<int>(n));
    for (auto& a : arra) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    for (auto& a : arrb) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    auto check = [&](long x) -> bool {
        vector<vector<long>> graph(n, vector<long>(n));
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                graph[i][j] = long(1e7) * arra[i][j] - x * arrb[i][j];
            }
        }
        return max_matching(graph) > 0;
    };
    long l = 0, r = 1e13;
    while (l < r) {
        long mid = (l + r + 1) / 2;
        if (check(mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    cout << double(l) / 1e7 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(6);
    solve();
}