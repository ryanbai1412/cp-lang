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

const int maxn = 105, maxl = 1005;
const long mod = 1e9 + 7;

long dp[maxn][maxn][maxl][3];

void solve() {
	int n, l;
	cin >> n >> l;
	if (!n) {
		cout << 1 << endl;
		return;
	}
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	sort(arr, arr + n);
	for (auto& a : dp[n][1]) {
		a[0] = 1;
	}
	for (int i = n - 1; i >= 0; i--) {
		int nxt = (i + 1 == n) ? arr[i] : arr[i + 1];
		int cdiff = nxt - arr[i];
		for (int ncomps = 0; ncomps <= n; ncomps++) {
			for (int csum = 0; csum <= l; csum++) {
				for (int nends = 0; nends <= 2; nends++) {
					int adj_empty = max(0, 2 * ncomps - 2 + nends);
					long& cans = dp[i][ncomps][csum][nends];
					auto gdp = [&](int compd, int sumd, int endsd) -> long {
						compd += ncomps;
						sumd += csum;
						endsd += nends;
						if (0 <= compd && sumd <= l && 0 <= endsd) {
							return dp[i + 1][compd][sumd][endsd];
						}
						return 0;
					};
					// start new component
					cans += (ncomps - 1 + nends) * gdp(+1, (adj_empty + 2) * cdiff, 0);
					// append to a component
					cans += adj_empty * gdp(0, adj_empty * cdiff, 0);
					// merge two components
					if (ncomps >= 2) {
						cans += (ncomps - 1) * gdp(-1, (adj_empty - 2) * cdiff, 0);
					}
					// add to left and right
					// create a component
					cans += nends * gdp(1, (adj_empty + 1) * cdiff, -1);
					// use existing component
					if (ncomps) {
						cans += nends * gdp(0, (adj_empty - 1) * cdiff, -1);
					}
					cans %= mod;
				}
			}
		}
	}
	dbg(dp[1][1][0][1], dp[2][1][0][0]);
	cout << dp[0][0][0][2] << endl;
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