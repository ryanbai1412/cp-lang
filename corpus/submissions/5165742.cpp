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

constexpr int maxn = 3e4 + 5, maxm = 10;

int n, m;
long memo[maxn][1 << maxm];

long dp(int ind, int prv) {
    if (ind * m >= n) {
        return (ind - 1) * m + __lg(prv) < n;
    }
    long& ans = memo[ind][prv];
    if (ans != -1) {
        return ans;
    }
    ans = ind >= 2;
    for (int i = 1; i < prv; i++) {
        ans += dp(ind + 1, i);
    }
    return ans;
}

void solve() {
    cin >> m >> n;
    memset(memo, -1, sizeof(memo));
    cout << dp(0, 1 << m) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}