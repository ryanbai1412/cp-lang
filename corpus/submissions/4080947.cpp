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

const int maxn = 1e5;

bool root[maxn];
int memo[maxn][2][2];
vector<int> graph[maxn];

namespace scycle {
bool vis[maxn];
vector<int> s, ans;

void dfs(int u, int p = -1) {
	if (sz(ans)) {
		return;
	} else if (vis[u]) {
		ans = vector<int>(find(begin(s), end(s), u), s.end());
		return;
	}
	vis[u] = true;
	s.push_back(u);
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	s.pop_back();
}

vector<int> compute() {
	dfs(0);
	return ans;
}
}  // namespace scycle

int dp(int u, bool par, bool cur, int p = -1) {
	int& ans = memo[u][par][cur];
	if (ans != -1) {
		return ans;
	}
	if (par) {
		ans = 0;
		for (auto& v : graph[u]) {
			if (!root[v] && v != p) {
				ans += dp(v, cur, false, u);
			}
		}
	} else {
		int tot = 0;
		ans = 1e9;
		for (auto& v : graph[u]) {
			if (!root[v] && v != p) {
				tot += dp(v, cur, false, u);
			}
		}
		for (auto& v : graph[u]) {
			if (!root[v] && v != p) {
				int cur = tot - dp(v, cur, false, u) + dp(v, cur, true, u);
				ans = min(ans, cur);
			}
		}
	}
	ans += cur;
	return ans;
}

int solve(const vector<array<int, 4>>& arr) {
	auto expects = [&](int x) -> bool { return x & 2; };
	auto on = [&](int x) -> bool { return x & 1; };
	auto valid = [&](int pa, int pb, int cur) -> bool {
		if (on(pa) + on(cur) != expects(pb)) {
			return false;
		}
		if (on(pb) > expects(cur)) {
			return false;
		}
		return true;
	};
	int n = sz(arr);
	long ans = 1e9;
	for (int a = 0; a < 4; a++) {
		for (int b = 0; b < 4; b++) {
			long dp[n + 1][4][4];
			for (int na = 0; na < 4; na++) {
				for (int nb = 0; nb < 4; nb++) {
					if (valid(na, nb, a) && valid(nb, a, b)) {
						dp[n][na][nb] = 0;
					} else {
						dp[n][na][nb] = 1e9;
					}
				}
			}
			for (int i = n - 1; i >= 2; i--) {
				for (int ca = 0; ca < 4; ca++) {
					for (int cb = 0; cb < 4; cb++) {
						dp[i][ca][cb] = 1e9;
						for (int nxt = 0; nxt < 4; nxt++) {
							if (valid(ca, cb, nxt)) {
								dp[i][ca][cb] =
									min(dp[i][ca][cb],
										arr[i][nxt] + dp[i + 1][cb][nxt]);
							}
						}
					}
				}
			}
			ans = min(ans, dp[2][a][b] + arr[0][a] + arr[1][b]);
		}
	}
	return ans == 1e9 ? -1 : ans;
}

void solve() {
	int n;
	cin >> n;
	for (int i = 0; i < n; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	vector<int> cycle = scycle::compute();
	for (auto& a : cycle) {
		dbg(a);
		root[a] = true;
	}
	memset(memo, -1, sizeof(memo));
	vector<array<int, 4>> cur;
	dbg(dp(3, true, true), dp(2, false, true));
	for (auto& a : cycle) {
		cur.push_back({dp(a, false, false), dp(a, false, true),
					   dp(a, true, false), dp(a, true, true)});
		{
			auto [b, c, d, e] = cur.back();
			dbg(a, b, c, d, e);
		}
	}
	cout << solve(cur) << endl;
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