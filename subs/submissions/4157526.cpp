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
	int m, n;
	cin >> m >> n;
	string names[n];
	int arr[n];
	for (int i = 0; i < n; i++) {
		cin >> names[i] >> arr[i];
	}
	pair<int, int> dp[n + 1];
	for (int i = n - 1; i >= 0; i--) {
		dp[i] = {1e9, 1e9};
		int cmx = 0;
		for (int j = i + 1; j <= min(n, i + m); j++) {
			cmx = max(cmx, arr[j - 1]);
			dp[i] = min(dp[i], {cmx + dp[j].first, j});
		}
	}
	cout << "Total Time: " << dp[0].first << endl;
	int i = 0;
	while (i < n) {
		int e = dp[i].second;
		for (; i < e; i++) {
			cout << names[i] << " \n"[i == e - 1];
		}
	}
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