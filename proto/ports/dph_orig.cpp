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
	char arr[n][m + 1];
	for (auto& a : arr) {
		cin >> a;
	}
	int dp[n + 1][m + 1] {};
	for (int i = n - 1; i >= 0; i--) {
		for (int j = m - 1; j >= 0; j--) {
			if (arr[i][j] == '.') {
				dp[i][j] = dp[i + 1][j] + dp[i][j + 1];
				if (dp[i][j] >= mod) {
					dp[i][j] -= mod;
				}
				if (i == n - 1 && j == m - 1) {
					dp[i][j] = 1;
				}
			}
		}
	}
	cout << dp[0][0] << endl;
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