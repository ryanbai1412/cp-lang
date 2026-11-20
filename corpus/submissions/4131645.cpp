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
	string as, bs;
	cin >> as >> bs;
	int n = sz(as), m = sz(bs);
	int dp[n + 1][m + 1] {};
	for (int i = n - 1; i >= 0; i--) {
		for (int j = m - 1; j >= 0; j--) {
			dp[i][j] = max(dp[i + 1][j], dp[i][j + 1]);
			if (as[i] == bs[j]) {
				dp[i][j] = max(dp[i][j], 1 + dp[i + 1][j + 1]);
			}
		}
	}
	int i = 0, j = 0;
	while (i < n && j < m) {
		int cur = dp[i][j];
		if (dp[i + 1][j] == cur) {
			i++;
		} else if (dp[i][j + 1] == cur) {
			j++;
		} else {
			cout << as[i];
			i++;
			j++;
		}
	}
	cout << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}