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

const int maxn = 2e5 + 5;

long fact[maxn + 1], ifact[maxn + 1];

void pcomp() {
	fact[0] = ifact[0] = 1;
	for (int i = 1; i <= maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
		ifact[i] = inv(fact[i]);
	}
}

long choose(int n, int k) {
	if (k < 0 || k > n) {
		return 0;
	}
	return ((fact[n] * ifact[n - k]) % mod * ifact[k]) % mod;
}

long ways(int x1, int y1, int x2, int y2) {
	return choose(x2 - x1 + y2 - y1, x2 - x1);
}

void solve() {
	int r, c, n;
	cin >> r >> c >> n;
	pair<int, int> arr[n + 1];
	for (int i = 0; i < n; i++) {
		auto &[x, y] = arr[i];
		cin >> x >> y;
		x--;
		y--;
	}
	arr[n] = {r - 1, c - 1};
	sort(arr, arr + n);
	long dp[n + 1];
	for (int i = 0; i <= n; i++) {
		auto [x, y] = arr[i];
		dp[i] = ways(0, 0, x, y);
		for (int j = 0; j < i; j++) {
			dp[i] = (dp[i] - ways(arr[j].first, arr[j].second, x, y) * dp[j]) % mod;
		}
		if (dp[i] < 0) {
			dp[i] += mod;
		}
		dbg(i, x, y, dp[i]);
	}
	cout << dp[n] << endl;
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