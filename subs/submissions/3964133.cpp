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

const int maxn = 3e3 + 5;
const long mod = 998244352;

long fact[maxn];

void pcomp() {
	fact[0] = 1;
	for (int i = 1; i < maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
	}
}

void solve() {
	int k, n;
	cin >> n >> k;
	int edges[n - 1][2];
	for (auto& [u, v] : edges) {
		cin >> u >> v;
		u--;
		v--;
	}
	long ans = 0;
	for (int i = 0; i < (1 << (n - 1)); i++) {
		if (__builtin_popcount(i) != k) {
			continue;
		}

		int deg[n] {};
		for (int j = 0; j < n - 1; j++) {
			if ((i >> j) & 1) {
				auto &[u, v] = edges[j];
				deg[u]++;
				deg[v]++;
			}
		}
		long cur = 1;
		for (int j = 0; j < n; j++) {
			cur = (cur * fact[deg[j]]) % mod;
		}
		ans = (ans + cur) % mod;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}