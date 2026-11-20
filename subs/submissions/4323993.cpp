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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int m = *max_element(arr, arr + n) + 1;
	int last[m];
	memset(last, -1, sizeof(last));
	for (int i = 0; i < n; i++) {
		last[arr[i]] = i;
	}
	ld dp[n] {};
	int cnt[m] {};
	for (int i = n - 2; i >= 0; i--) {
		cnt[arr[i + 1]]++;
		if (i && i != last[arr[i]]) {
			continue;
		}
		dp[i] = (1 + ld(cnt[arr[0]])) / (n - i);
		for (int j = 0; j < m; j++) {
			if (last[j] > i && j != arr[0]) {
				dp[i] += ld(cnt[j]) / (n - i) * dp[last[j]];
			}
		}
		dbg(i, dp[i]);
	}
	cout << dp[0] << endl;
}

int main() {
	cout << fixed << setprecision(6);
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