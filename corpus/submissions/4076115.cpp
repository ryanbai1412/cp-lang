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
	int n;
	cin >> n;
	int in[n];
	for (auto& a : in) {
		cin >> a;
	}
	int arr[n - 1];
	for (int i = 0; i < n - 1; i++) {
		arr[i] = in[i + 1] - in[i];
	}
	sort(arr, arr + n - 1, greater<>());
	int psum[n - 1];
	partial_sum(arr, arr + n - 1, psum);
	int m = accumulate(psum, psum + n - 1, 0);
	reverse(arr, arr + n - 1);
	partial_sum(arr, arr + n - 1, psum);
	dbg(m);
	long dp[n][m + 1];
	memset(dp, 0x3f, sizeof(dp));
	for (int sum = 0; sum <= m; sum++) {
		dp[n - 1][sum] = long(-sum) * sum;
	}
	auto sq = [&](long x) -> long { return x * x; };
	for (int i = n - 2; i >= 0; i--) {
		for (int sum = 0; sum <= m; sum++) {
			{
				int nind = sum + psum[i];
				if (nind <= m) {
					dp[i][sum] = dp[i + 1][nind] + n * sq(psum[i]);
				}
			}
			{
				int nind = sum + (i + 1) * arr[i];
				if (nind <= m) {
					dp[i][sum] =
						min(dp[i][sum], dp[i + 1][nind] + 2 * n * sum * arr[i] +
											n * (i + 1) * sq(arr[i]));
					dbg(i, sum, nind,
						dp[i + 1][nind] + 2 * n * sum * arr[i] +
							n * (i + 1) * sq(arr[i]));
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