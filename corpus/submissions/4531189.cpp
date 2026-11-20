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
	long dp[n + 1];
	dp[0] = 0;
	for (int i = 1; i <= n; i++) {
		dp[i] = 1e18;
		for (int j = 0; j < i && j + 2 <= k; j++) {
			dp[i] = min(dp[i], dp[i - j - 1] + arr[j + 2] + j * arr[1]);
		}
	}
	cout << dp[n - 2] + 2 * arr[1] << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}