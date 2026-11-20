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
	long arr[n + 1][2];
	for (int i = 1; i <= n; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	arr[0][0] = arr[0][1] = 0;
	long dp1[n + 1], dp2[n + 1];
	dp1[n] = dp2[n] = 0;
	for (int i = n - 1; i >= 0; i--) {
		dp1[i] = dp2[i] = 1e18;
		{
			long cadd = 0, pcx = i ? arr[i - 1][0] : 0;
			for (int j = i + 1; j <= n; j++) {
				dp1[i] = min(dp1[i], cadd + abs(pcx - arr[j][0]) + dp2[j]);
				cadd += abs(arr[j - 1][1] - arr[j][1]);
			}
			dp1[i] = min(dp1[i], cadd);
		}
		{
			long cadd = 0, pcx = i ? arr[i - 1][1] : 0;
			for (int j = i + 1; j <= n; j++) {
				dp2[i] = min(dp2[i], cadd + abs(pcx - arr[j][1]) + dp1[j]);
				cadd += abs(arr[j - 1][0] - arr[j][0]);
			}
			dp2[i] = min(dp2[i], cadd);
		}
		dbg(i, dp1[i], dp2[i]);
	}
	cout << min(dp1[0], dp2[0]) << endl;
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