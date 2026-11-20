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
	int n, k;
	cin >> n >> k;
	string s;
	cin >> s;
	bool arr[n];
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] == 'C';
	}
	int psum[n + 1], ssum[n + 1];
	psum[0] = ssum[n] = 0;
	for (int i = 0; i < n; i++) {
		psum[i + 1] = psum[i] + arr[i];
	}
	for (int i = n - 1; i >= 0; i--) {
		ssum[i] = ssum[i + 1] + arr[i];
	}
	bool dp[n][n][k + 1] {};
	for (int l = n - 1; l >= 0; l--) {
		for (int r = 0; r < n; r++) {
			dp[l][r][k] = false;
			if (l > r) {
				continue;
			}
			int tk = psum[l] + ssum[r] - arr[r];
			for (int ck = 0; ck <= min(k - 1, tk); ck++) {
				int ok = tk - ck;
				if (ok >= k) {
					dp[l][r][ck] = true;
				} else {
					if (l == r) {
						dp[l][r][ck] = false;
					} else {
						dp[l][r][ck] = !dp[l + 1][r][ok] || !dp[l][r - 1][ok];
					}
				}
			}
		}
	}
	if (dp[0][n - 1][0]) {
		cout << "DA" << endl;
	} else {
		cout << "NE" << endl;
	}
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