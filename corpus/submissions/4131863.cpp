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
	string s;
	cin >> s;
	long dp[n + 1] {};
	dp[0] = 1;
	for (int i = n - 2; i >= 0; i--) {
		dbg(dp[3]);
		int ci = n - i - 1;
		dbg(i, ci);
		long psum[n + 1] {};
		if (s[i] == '<') {
			psum[ci] = dp[ci];
			dbg(ci, dp[ci]);
			for (int j = ci - 1; j >= 0; j--) {
				psum[j] = psum[j + 1] + dp[j];
				dbg(j, psum[j], psum[j + 1], dp[j]);
			}
		} else {
			dbg("x", s[i]);
			partial_sum(dp, dp + ci, psum + 1);
		}
		memcpy(dp, psum, sizeof(dp));
		for (auto& a : dp) {
			a %= mod;
		}
		for (int j = 0; j < n; j++) {
			dbg(i, j, dp[j]);
		}
	}
	cout << accumulate(dp, dp + n + 1, long(0)) % mod << endl;
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