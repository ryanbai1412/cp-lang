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

const int maxm = 50 + 5, maxn = 1e3 + 5;

void solve() {
	int n, m;
	cin >> n >> m;
	string arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int psum[maxm][maxn];
	for (int i = 0; i < m; i++) {
		psum[i][0] = 0;
		for (int j = 0; j < n; j++) {
			psum[i][j + 1] = psum[i][j] + (arr[j][i] == '1');
		}
	}
	auto gchange = [&](int i, int l, int r, int nxt) -> int {
		int csum = psum[i][r + 1] - psum[i][l];
		if (nxt) {
			return (r - l + 1) - csum;
		} else {
			return csum;
		}
	};
	int dp[m + 1][n][n] {};
	for (int i = m - 1; i >= 0; i--) {
		for (int l = 0; l < n; l++) {
			for (int r = l + 1; r < n; r++) {
				dp[i][l][r] = min(gchange(i, l, r, 0), gchange(i, l, r, 1)) + dp[i + 1][l][r];
				for (int k = l; k < r; k++) {
					dp[i][l][r] = min(dp[i][l][r], dp[i + 1][l][k] + dp[i + 1][k + 1][r] + gchange(i, l, k, 0) +
													   gchange(i, k + 1, r, 1));
				}
			}
		}
	}
	dbg(dp[1][0][0], dp[1][1][3]);
	cout << dp[0][0][n - 1] << endl;
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