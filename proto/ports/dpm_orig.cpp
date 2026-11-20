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
	int n, m;
	cin >> n >> m;
	long dp[m + 1] {};
	dp[0] = 1;
	for (int i = 0; i < n; i++) {
		long psum[m + 2];
		psum[0] = 0;
		for (int j = 0; j <= m; j++) {
			psum[j + 1] = psum[j] + dp[j];
			if (psum[j + 1] >= mod) {
				psum[j + 1] -= mod;
			}
		}
		auto sum = [&](int l, int r) -> long {
			l = max(0, l);
			r = min(m, r);
			long ans = psum[r + 1] - psum[l] + mod;
			if (ans >= mod) {
				ans -= mod;
			}
			return ans;
		};
		int x;
		cin >> x;
		for (int j = 0; j <= m; j++) {
			dp[j] = sum(j - x, j);
		}
	}
	cout << dp[m] << endl;
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