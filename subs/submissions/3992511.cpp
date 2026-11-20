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

const int maxm = 105;

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	long ans = 0;
	long dp[3][maxm] {};
	for (int i = 0; i < n; i++) {
		ans += dp[2][arr[i]];
		for (int j: {1, 0}) {
			for (int k = 0; k < maxm; k++) {
				if (k + arr[i] < maxm) {
					dp[j + 1][k + arr[i]] += dp[j][k];
				}
			}
		}
		dp[0][arr[i]]++;
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