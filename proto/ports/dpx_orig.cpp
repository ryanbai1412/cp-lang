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

const int maxm = 1e4 + 5;

void solve() {
	int n;
	cin >> n;
	tuple<int, int, int> arr[n];
	for (auto& [w, c, v] : arr) {
		cin >> w >> c >> v;
	}
	sort(arr, arr + n, [&](const auto &a, const auto &b) -> bool {
		return get<0>(a) + get<1>(a) < get<0>(b) + get<1>(b);
	});
	long ans = 0;
	long dp[maxm] {};
	for (auto& [w, c, v] : arr) {
		for (int i = c; i >= 0; i--) {
			long cur = dp[i] + v;
			ans = max(ans, cur);
			if (i + w < maxm) {
				dp[i + w] = max(dp[i + w], cur);
			}
		}
	}
	cout << ans << endl;
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