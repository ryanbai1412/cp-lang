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

constexpr long mod = 1e6;
constexpr int maxn = 35;

long memo[maxn][3][8];

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

long dp(int x, int y, int mask) {
    if (y == 3) {
        x--;
        y = 0;
    }
    if (x == -1) {
        return !mask;
    }
    long& ans = memo[x][y][mask];
    if (ans != -1) {
        return ans;
    }
    ans = 0;
    if (!on(mask, y)) {
        ans += dp(x, y + 1, mask | (1 << y));
        if (y + 1 < 3 && !on(mask, y + 1)) {
            ans += dp(x, y + 2, mask);
        }
    } else {
        ans += dp(x, y + 1, mask & (~(1 << y)));
    }
    return ans %= mod;
}

void solve() {
    int n;
    cin >> n;
    cout << dp(n - 1, 0, 0) << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    memset(memo, -1, sizeof(memo));
    for (int tc = 1; tc <= 5; tc++) {
        solve();
    }
}