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
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	long dp[n][n];
	for (int i = n - 1; i >= 0; i--) {
		long csum = arr[i];
		dp[i][i] = 0;
		for (int j = i + 1; j < n; j++) {
			csum += arr[j];
			dp[i][j] = 1e18;
			for (int k = i; k < j; k++) {
				dp[i][j] = min(dp[i][j], dp[i][k] + dp[k + 1][j]);
			}
			dp[i][j] += csum;
		}
	}
	cout << dp[0][n - 1] << endl;
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