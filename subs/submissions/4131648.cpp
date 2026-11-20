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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n;
	cin >> n;
	ld arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	ld dp[n + 1] {};
	dp[0] = 1;
	for (int i = 0; i < n; i++) {
		ld cp = arr[i];
		for (int j = i + 1; j >= 1; j--) {
			dp[j] = cp * dp[j - 1] + (1 - cp) * dp[j];
		}
		dp[0] *= (1 - cp);
	}
	ld ans = 0;
	for (int i = 0; i <= n; i++) {
		if (i > (n - i)) {
			ans += dp[i];
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	cout << fixed << setprecision(9);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}