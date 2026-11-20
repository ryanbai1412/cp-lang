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

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n];
	memset(arr, 0x3f, sizeof(arr));
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		if (u > v) {
			swap(u, v);
		}
		arr[v] = min(arr[v], u);
	}
	int suff[n + 1];
	suff[n] = 1e9;
	for (int i = n - 1; i >= 0; i--) {
		suff[i] = min(suff[i + 1], arr[i]);
	}
	int dp[n + 1];
	dp[n] = 0;
	for (int i = n - 1; i >= 1; i--) {
		dp[i] = 1e9;
		int cur = 0;
		for (int j = i + 1; j <= n; j++) {
			cur += arr[j - 1] >= i;
			if (suff[j] >= i) {
				dp[i] = min(dp[i], cur + dp[j]);
			}
		}
	}
	cout << dp[1] << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}