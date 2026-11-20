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

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

long inv(long x) {
	return bpow(x, mod - 2);
}

const int maxf = 1e6;

long fact[maxf + 1], ifact[maxf + 1];

void pcomp() {
	fact[0] = ifact[0] = 1;
	for (int i = 1; i <= maxf; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
		ifact[i] = inv(fact[i]);
	}
}

long choose(int n, int k) {
	if (n >= 0 && k >= 0 && n - k >= 0) {
		return fact[n] * ifact[k] % mod * ifact[n - k] % mod;
	}
	return 0;
}

const int maxn = 55, maxl = 1e4 + 5;
long dp[2][maxn][maxl][3];

void solve() {
	int n, l;
	cin >> n >> l;
	if (n == 1) {
		cout << l << endl;
		return;
	}
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	sort(arr, arr + n, greater<>());
	int ci = 0, pi = 1;
	dp[ci][1][l - 1][0] = 1;
	for (int i = n - 1; i >= 0; i--) {
		swap(ci, pi);
		memset(dp[ci], 0, sizeof(dp[ci]));
		for (int j = 0; j <= n; j++) {
			for (int k = 0; k < l; k++) {
				for (int e = 0; e <= 2; e++) {
					auto gdp = [&](int nj, int nk, int ne) -> long {
						nj += j;
						nk += k;
						ne += e;
						if (0 <= nj && 0 <= nk && nk < l && 0 <= ne) {
							return dp[pi][nj][nk][ne];
						}
						return 0;
					};
					long& cdp = dp[ci][j][k][e];
					// create component
					cdp += (j - 1 + e) * gdp(+1, 2 * arr[i], 0);
					// append to component
					if (j) {
						cdp += (2 * j - 2 + e) * gdp(0, arr[i], 0);
					}
					// merge component
					if (j >= 2) {
						cdp += (j - 1) * gdp(-1, 0, 0);
					}
					// add left / right
					// use existing
					if (j) {
						cdp += e * gdp(0, 0, -1);
					}
					// create new
					cdp += e * gdp(+1, arr[i], -1);
					cdp %= mod;
				}
			}
		}
	}
	long ans = 0;
	for (int i = 0; i < l; i++) {
		dbg(i, dp[ci][0][i][2]);
		ans = (ans + dp[ci][0][i][2] * choose(i + n, n)) % mod;
	}
	ans %= mod;
	cout << ans << endl;
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