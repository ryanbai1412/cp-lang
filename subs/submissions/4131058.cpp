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
}

void solve() {
	int n;
	cin >> n;
	long ans = 0;
	long cn = inv(n - 1), cpow = cn;
	for (int i = 2; i <= n; i++) {
		cpow = (cpow * cn) % mod;
		long cur = choose(n, i);
		cur = (cur * fact[i - 1]) % mod;
		cur = (cur * cpow) % mod;
		ans += cur;
	}
	cout << ans % mod << endl;
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