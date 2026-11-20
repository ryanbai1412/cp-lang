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
	long arr[n][n];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	long pcomp[1 << n] {};
	for (int i = 0; i < (1 << n); i++) {
		for (int j = 0; j < n; j++) {
			if ((i >> j) & 1) {
				for (int k = j + 1; k < n; k++) {
					if ((i >> k) & 1) {
						pcomp[i] += arr[j][k];
					}
				}
			}
		}
	}
	long dp[1 << n];
	memset(dp, 0x4f, sizeof(dp));
	dp[0] = 0;
	for (int i = 1; i < (1 << n); i++) {
		dp[i] = pcomp[i];
		int j = i;
		while (j) {
			dp[i] = max(dp[i], pcomp[j] + dp[i ^ j]);
			j = (j - 1) & i;
		}
	}
	cout << dp[(1 << n) - 1] << endl;
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