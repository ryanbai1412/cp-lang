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

constexpr int maxm = 2000 * 50 + 5;

void solve() {
    int n;
    cin >> n;
    int arri[n][3];
    for (auto& [k, x, p] : arri) {
        cin >> k >> x >> p;
    }
    int m;
    cin >> m;
    int arrq[m][3];
    for (auto& [k, x, p] : arrq) {
        cin >> k >> x >> p;
    }
    vector<array<int, 3>> arr;
    for (auto& [k, x, p] : arri) {
        arr.push_back({x, k, -p});
    }
    for (auto& [k, x, p] : arrq) {
        arr.push_back({x, -k, p});
    }
    sort(begin(arr), end(arr), greater<>());
    long dp[maxm];
    fill(begin(dp), end(dp), -1e18);
    dp[0] = 0;
    for (auto& [_x, k, p] : arr) {
        long pdp[maxm];
        memcpy(pdp, dp, sizeof(dp));
        for (int i = 0; i < maxm; i++) {
            int ni = i + k;
            if (0 <= ni && ni < maxm) {
                dp[ni] = max(dp[ni], pdp[i] + p);
            }
        }
    }
    cout << *max_element(begin(dp), end(dp)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}