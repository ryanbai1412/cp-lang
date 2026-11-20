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
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	long psum[n + 1];
	psum[0] = 0;
	for (int i = 0; i < n; i++) {
		psum[i + 1] = psum[i] + arr[i];
	}
	auto sum = [&](int l, int r) -> long {
		return psum[r + 1] - psum[l];
	};
	long dp[n + 1];
	dp[n] = 0;
	for (int i = n - 1; i >= 0; i--) {
		dp[i] = dp[i + 1];
		if (i + k <= n) {
			dp[i] = max(dp[i], sum(i, i + k - 1) + dp[i + k]);
		}
	}
	cout << dp[0] << endl;
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