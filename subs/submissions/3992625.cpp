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

void solve() {
	int n, k;
	cin >> n >> k;
	long cost[n], arr[n];
	for (auto& a : cost) {
		cin >> a;
	}
	for (auto& a : arr) {
		cin >> a;
	}
	long opt[n][n];
	for (int i = 0; i < n; i++) {
		long tot = 0;
		vector<pair<long, long>> cur;
		for (int j = i; j < n; j++) {
			cur.emplace_back(arr[j], cost[j]);
			sort(begin(cur), end(cur));
			tot += cost[j];
			long targ = tot / 2;
			long csum = 0;
			for (auto& [x, cnt] : cur) {
				csum += cnt;
				if (csum > targ) {
					long cans = 0;
					for (auto& [h, c] : cur) {
						cans += c * abs(x - h);
					}
					opt[i][j] = cans;
					dbg(i, j, cans);
					break;
				}
			}
		}
	}
	long dp[n + 1][k + 1] {};
	for (int i = n - 1; i >= 0; i--) {
		dp[i][0] = 4e18;
		for (int j = 1; j <= k; j++) {
			dp[i][j] = 4e18;
			for (int nxt = i + 1; nxt <= n; nxt++) {
				dp[i][j] = min(dp[i][j], dp[nxt][j - 1] + opt[i][nxt - 1]);
			}
		}
	}
	long ans = 4e18, ccost = 0;
	for (int i = 0; i < n; i++) {
		ans = min(ans, ccost + dp[i][k]);
		ccost += arr[i] * cost[i];
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
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