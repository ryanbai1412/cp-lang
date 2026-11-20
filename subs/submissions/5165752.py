"""
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
"""

m, n = map(int, input().split())

memo = [[-1] * (1 << 10) for _ in range(int(3e4 + 5))]


def dp(ind, prv):
    if ind * m >= n:
        return (ind - 1) * m + prv.bit_length() <= n
    if memo[ind][prv] != -1:
        return memo[ind][prv]

    ans = ind >= 2
    for i in range(1, prv):
        ans += dp(ind + 1, i)

    memo[ind][prv] = ans
    return ans


print(dp(0, 1 << m))