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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 3e3 + 5, maxk = 3e3 + 5;

void solve() {
	int n, k;
	cin >> n >> k;
	long arr[k + 1];
	for (int i = 1; i <= k; i++) {
		cin >> arr[i];
	}
	long dp[n][k + 1];
	memset(dp, 0x3f, sizeof(dp));
	dp[0][0] = 0;
	for (int i = 1; i < n; i++) {
		for (int j = 0; j < k; j++) {
			dp[i][1] = min(dp[i][1], dp[i - 1][j] + arr[j + 1]);
		}
		for (int j = 0; j < i; j++) {
			for (int x = 1; x <= k; x++) {
				dp[i][x] = min(dp[i][x], dp[j][x - 1] + dp[i - j][1]);
			}
		}
	}
	long cans = 1e18;
	for (int x = 0; x <= k; x++) {
		cans = min(cans, dp[n - 1][x] + arr[x]);
	}
	cout << cans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}