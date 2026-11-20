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
#define sz(x) int((x).size())

const int maxn = 205, maxm = 505;
const long mod = 1e9 + 7;

long ans1, ans2;
int k, garr[maxn][2];
vector<int> st, graph[maxn];

long cnt(long l, long r) {
	if (l > r) {
		return 0;
	}
	long ans = r * (r + 1) / 2 - l * (l - 1) / 2;
	return ans % mod;
}

void csolve() {
	int n = sz(st);
	int arr[n][2];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < 2; j++) {
			arr[i][j] = garr[st[i]][j];
		}
	}
	long dp[2][maxm] {}, dp2[2][maxm] {};
	fill(begin(dp[0]), end(dp[0]), 1);
	for (auto& [l, r] : arr) {
		for (int i = 0; i < maxm; i++) {
			{
				long a = dp2[0][i] * max(0, min(r, i + k) - max(l, i + 1) + 1) +
						 dp[0][i] * cnt(max(l, i + 1), min(r, i + k));
				long b = dp2[1][i] * max(0, min(r, i + k) - max(l, i) + 1) +
						 dp[1][i] * cnt(max(l, i), min(r, i + k));
				if (l <= i && i <= r) {
					b += dp2[0][i] + dp[0][i] * i;
				}
				dp2[0][i] = a % mod;
				dp2[1][i] = b % mod;
			}
			{
				long a = dp[0][i] * max(0, min(r, i + k) - max(l, i + 1) + 1);
				long b = dp[1][i] * max(0, min(r, i + k) - max(l, i) + 1);
				if (l <= i && i <= r) {
					b += dp[0][i];
				}
				dp[0][i] = a % mod;
				dp[1][i] = b % mod;
			}
		}
	}
	ans1 = (ans1 + accumulate(begin(dp[1]), end(dp[1]), long(0))) % mod;
	ans2 = (ans2 + accumulate(begin(dp2[1]), end(dp2[1]), long(0))) % mod;
}

void dfs(int u, int p = -1) {
	st.push_back(u);
	if (st[0] <= u) {
		csolve();
	}
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	st.pop_back();
}

void solve() {
	int n;
	cin >> n >> k;
	for (int i = 0; i < n; i++) {
		cin >> garr[i][0] >> garr[i][1];
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		dfs(i);
	}
	cout << ans1 << endl;
	cout << ans2 << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}