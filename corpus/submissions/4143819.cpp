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

const long mod = 1e9 + 7;
const int maxn = 500;

long choose[maxn + 1][maxn + 1];

void pcomp() {
	for (int i = 0; i <= maxn; i++) {
		choose[i][0] = choose[i][i] = 1;
		for (int j = 1; j < i; j++) {
			choose[i][j] = choose[i - 1][j - 1] + choose[i - 1][j];
			if (choose[i][j] >= mod) {
				choose[i][j] -= mod;
			}
		}
	}
}

void solve() {
	int n, l;
	cin >> n >> l;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	sort(arr, arr + n);
	if (arr[0] == arr[n - 1]) {
		long dp[l + 1][n + 1] {};
		dp[0][0] = 1;
		for (int ilen = 1; ilen <= l; ilen++) {
			for (int mcnt = 1; mcnt <= n; mcnt++) {
				for (int pos = 0; pos < ilen; pos++) {
					int llen = max(0, pos - arr[0] + 1);
					int rlen = max(0, ilen - pos - arr[0]);
					for (int lx = 0; lx < mcnt; lx++) {
						dp[ilen][mcnt] =
							(dp[ilen][mcnt] +
							 choose[mcnt - 1][lx] * dp[llen][lx] % mod * dp[rlen][mcnt - lx - 1]) %
							mod;
					}
				}
				dbg(ilen, mcnt, dp[ilen][mcnt]);
			}
			dp[ilen][0] = 1;
		}
		cout << dp[l][n] << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}