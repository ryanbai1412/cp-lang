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

const int maxn = 1e5;

long fact[maxn + 1], ifact[maxn + 1];

void pcomp() {
	fact[0] = ifact[0] = 1;
	for (int i = 1; i <= maxn; i++) {
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

void solve() {
	int n, l;
	cin >> n >> l;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	sort(arr, arr + n);
	long ans = 0;
	do {
		int cur = l - 1;
		for (int i = 0; i < n - 1; i++) {
			cur -= max(arr[i], arr[i + 1]);
		}
		dbg(cur, cur + n, n);
		ans += choose(cur + n, n);
	} while (next_permutation(arr, arr + n));
	map<int, int> cnt;
	for (auto& a : arr) {
		cnt[a]++;
	}
	ans %= mod;
	for (auto& [_, v] : cnt) {
		ans = (ans * fact[v]) % mod;
	}
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