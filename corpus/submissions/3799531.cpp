#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 100;
const long mod = 998244353;

long fact[maxn + 1], ifact[maxn + 1];

long bpow(long base, long exp) {
	long ans = 1;
	while(exp) {
		if(exp & 1) {
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

void pcomp() {
	fact[0] = ifact[0] = 1;
	for(int i = 0; i < maxn; i++) {
		fact[i + 1] = (fact[i] * (i + 1)) % mod;
		ifact[i + 1] = (ifact[i] * inv(i + 1)) % mod;
	}
}

long choose(int n, int k) {
	if(k < 0 || n - k < 0) {
		return 0;
	}
	long x = (fact[n] * ifact[n - k]) % mod;
	return (x * ifact[k]) % mod;
}

void solve() {
	pcomp();
	int n, m, t;
	cin >> n >> m >> t;
	long dp[n];
	for(int i = 1; i <= n; i++) {
		if(t == 1) {
			dp[i - 1] = i;
		}else {
			dp[i - 1] = i * i;
		}
	}
	for(int i = 0; i < m; i++) {
		int x;
		cin >> x;
		int a = x, b = n - x;
		long ndp[n] {};
		for(int s = 0; s < a; s++) {
			for(int e = 0; e < n; e++) {
				long ways = (choose(e, s) * choose(n - e - 1, a - s - 1)) % mod;
				dbg(s, e, ways);
				ndp[e] = (ndp[e] + ways * dp[s]) % mod;
			}
		}
		for(int s = 0; s < b; s++) {
			for(int e = 0; e < n; e++) {
				long ways = (choose(e, s) * choose(n - e - 1, b - s - 1)) % mod;
				dbg(a + s, e, ways);
				ndp[e] = (ndp[e] + ways * dp[a + s]) % mod;
			}
		}
		long tot = inv(choose(a + b, a));
		for(int j = 0; j < n; j++) {
			ndp[j] = (ndp[j] * tot) % mod;
		}
		memcpy(dp, ndp, sizeof(dp));
	}
	int q;
	cin >> q;
	while(q--) {
		int x;
		cin >> x;
		x--;
		cout << dp[x] << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}