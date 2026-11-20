"""

long solve(const vector<int>& arr) {
    int n = sz(arr);
    long dp[n][n] {};
    for (int l = n - 1; l >= 0; l--) {
        dp[l][l] = arr[l];
        for (int r = l + 1; r < n; r++) {
            dp[l][r] =
                max(arr[l] + 2 * dp[l + 1][r], arr[r] + 2 * dp[l][r - 1]);
        }
    }
    return 2 * dp[0][n - 1];
}

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    long ans = 0;
    for (auto& a : arr) {
        ans += solve(vector<int>(a, a + m));
    }
    cout << ans << endl;
}

"""


def solve(arr):
    n = len(arr)
    dp = [[0] * n for _ in range(n)]
    for l in reversed(range(0, n)):
        dp[l][l] = arr[l]
        for r in range(l + 1, n):
            dp[l][r] = max(arr[l] + 2 * dp[l + 1][r], arr[r] + 2 * dp[l][r - 1])
    return 2 * dp[0][n - 1]


n, m = map(int, input().split())
ans = 0
for i in range(n):
    ans += solve(list(map(int, input().split())))
print(ans)