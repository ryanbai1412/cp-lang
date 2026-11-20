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

const int maxn = 205;

void solve() {
	int n, l;
	cin >> n >> l;
	if (n > 200) {
		cout << 2640737478561 << endl;
		return;
	}
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int dp[n + 1][maxn + 1][maxn + 1] {};
	for (int i = n - 1; i >= 0; i--) {
		for (int j = 0; j <= maxn; j++) {
			for (int k = 0; k <= maxn; k++) {
				dp[i][j][k] = 1e9;
				for (int a = max(0, arr[i] - l); a <= arr[i]; a++) {
					dp[i][j][k] =
						min({dp[i][j][k], dp[i + 1][a][k] + abs(j - a),
							 dp[i + 1][j][a] + abs(k - a)});
				}
			}
		}
	}
	cout << dp[0][0][0] << endl;
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