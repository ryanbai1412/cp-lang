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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 114, maxk = 20;
const long mod = 998244353;

int n, k;
long memo[maxn][1 << maxk];

long dfs(int u, int cur) {
	if (__builtin_popcount(cur) == k / 2) {
		return 0;
	}
	if (u == n) {
		return 1;
	}
	if (memo[u][cur] != -1) {
		return memo[u][cur];
	}
	long& ans = memo[u][cur];
	ans = 0;
	ans += dfs(u + 1, (cur >> 1) | (1 << (k - 1)));
	ans += dfs(u + 1, cur >> 1);
	if (ans >= mod) {
		ans -= mod;
	}
	return ans;
}

void solve() {
	int m;
	cin >> n >> k >> m;
	int cur = 0;
	if (m) {
		string s;
		cin >> s;
		for (int i = 0; i < k; i++) {
			if (i < m && s[i] == '1') {
				cur |= 1 << i;
			}
		}
	}
	bool val[1 << k];
	int dp[1 << k];
	for (int i = 0; i < (1 << k); i++) {
		val[i] = __builtin_popcount(i) != k / 2;
		dp[i] = val[i];
	}
	int ckx = (1 << k - 1);
	for (int i = n - 1; i >= k; i--) {
		int ndp[1 << k] {};
		for (int i = 0; i < (1 << k); i++) {
			if (val[i]) {
				ndp[i] = dp[i >> 1] + dp[(i >> 1) | ckx];
				if (ndp[i] >= mod) {
					ndp[i] -= mod;
				}
			}
		}
		memcpy(dp, ndp, sizeof(dp));
	}
	long ans = 0;
	for (int i = 0; i < (1 << (k - m)); i++) {
		ans += dp[cur | (i << m)];
	}
	cout << ans % mod << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}