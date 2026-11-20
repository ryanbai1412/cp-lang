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

constexpr int maxn = 355, maxm = 45;

int arr[maxn], memo[maxm][maxm][maxm][maxm];

int dp(int ind, int c1, int c2, int c3, int c4) {
    if (c1 < 0 || c2 < 0 || c3 < 0 || c4 < 0) {
        return -1e9;
    }
    if (!c1 && !c2 && !c3 && !c4) {
        return arr[ind];
    }
    int& ans = memo[c1][c2][c3][c4];
    if (ans != -1) {
        return ans;
    }
    ans = max(ans, dp(ind + 1, c1 - 1, c2, c3, c4));
    ans = max(ans, dp(ind + 2, c1, c2 - 1, c3, c4));
    ans = max(ans, dp(ind + 3, c1, c2, c3 - 1, c4));
    ans = max(ans, dp(ind + 4, c1, c2, c3, c4 - 1));
    return ans += arr[ind];
}

void solve() {
    int n, m;
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    int arrb[m];
    for (auto& a : arrb) {
        cin >> a;
    }
    int cnt[4] {};
    for (auto& a : arrb) {
        cnt[a - 1]++;
    }
    memset(memo, -1, sizeof(memo));
    cout << dp(0, cnt[0], cnt[1], cnt[2], cnt[3]) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}