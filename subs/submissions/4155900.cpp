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
	pair<int, int> dp[2][n][n] {};
	int ci = 0, pi = 1;
	for (int i = m - 1; i >= 0; i--) {
		swap(ci, pi);
		for (int cx = n; cx >= 1; cx--) {
			for (int l = 0; l + cx < n; l++) {
				int r = l + cx;
				int optl = l - 1;
				if (l) {
					optl = max(optl, dp[ci][l - 1][r].second);
				}
				int optr = r;
				if (r + 1 < n) {
					optr = min(optr, dp[ci][l][r + 1].second);
				}
				dp[ci][l][r] = {1e9, 1e9};
				for (int k = optl; k <= optr; k++) {
					if (k == l - 1) {
						dp[ci][l][r] =
							min(dp[ci][l][r],
								pair<int, int> {gchange(i, l, r, 1) + dp[pi][l][r].first, k});
					} else if (k == r) {
						dp[ci][l][r] =
							min(dp[ci][l][r],
								pair<int, int> {gchange(i, l, r, 0) + dp[pi][l][r].first, k});
					} else {
						dbg(l, k, r);
						dp[ci][l][r] =
							min(dp[ci][l][r],
								pair<int, int> {dp[pi][l][k].first + dp[pi][k + 1][r].first +
													gchange(i, l, k, 0) + gchange(i, k + 1, r, 1),
												k});
					}
				}
			}
		}
	}
	cout << dp[ci][0][n - 1].first << endl;
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