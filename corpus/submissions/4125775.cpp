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

const int maxn = 3e3 + 5;

long fact[maxn + 1], ifact[maxn + 1], dp1[maxn + 1];

long choose(int n, int k) {
	return (fact[n] * ifact[k] % mod * ifact[n - k]) % mod;
}

void pcomp() {
	fact[0] = ifact[0] = 1;
	for (int i = 1; i <= maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
		ifact[i] = inv(fact[i]);
	}
	for (int n = 1; n <= maxn; n++) {
		long cpow = 1;
		for (int k = 0; k <= n - 2; k++) {
			dp1[n] = (dp1[n] + cpow * ifact[k]) % mod;
			cpow = (cpow * n) % mod;
		}
		dp1[n] = (dp1[n] * fact[n - 1]) % mod;
	}
}

void solve() {
	int n;
	cin >> n;
	long dp[n + 1] {}, ways[n + 1] {};
	dp[0] = 1;
	dp[1] = 0;
	for (int i = 2; i <= n; i++) {
		for (int j = 1; j <= i; j++) {
			dp[i] = (dp[i] + dp1[j] * dp[i - j] % mod * choose(i - 1, j - 1)) % mod;
			dp[i] = (dp[i] + dp1[j] * ways[i - j] % mod * choose(i - 1, j - 1)) % mod;
		}
		ways[i] = bpow(i - 1, i);
//		dp[i] = (dp[i] * inv(bpow(i - 1, i))) % mod;
		dbg(i, dp[i], dp1[i], ways[i]);
	}
	dbg(dp[n]);
	cout << (dp[n] * inv(ways[n])) % mod << endl;
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