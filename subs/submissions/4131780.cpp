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

const long mod = 1e9 + 7;

void solve() {
	int n;
	cin >> n;
	bool arr[n][n];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	int dp[1 << n] {};
	dp[(1 << n) - 1] = 1;
	for (int i = (1 << n) - 2; i >= 0; i--) {
		int cind = __builtin_popcount(i);
		for (int j = 0; j < n; j++) {
			if (arr[cind][j] && !((i >> j) & 1)) {
				dp[i] += dp[i | (1 << j)];
				if (dp[i] >= mod) {
					dp[i] -= mod;
				}
			}
		}
	}
	cout << dp[0] << endl;
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