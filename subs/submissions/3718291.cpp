#pragma GCC optimize("Ofast,unroll-loops,no-stack-protector")
#pragma GCC target("sse,sse2,sse3,ssse3,sse4,sse4a,sse4.1,sse4.2,popcnt,abm,mmx,avx,avx2,fma")

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

const long mod = 1e9 + 7;
const int maxn = 150;

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
	for(int i = 1; i <= maxn; i++) {
		fact[i] = (fact[i - 1] * i) % mod;
		ifact[i] = (ifact[i - 1] * inv(i)) % mod;
	}
}

long choose(long n, long k) {
	if(k < 0 || k > n) {
		return 0;
	}
	return (fact[n] * ifact[k] % mod) * ifact[n - k] % mod;
}

using bmat = array<array<long, maxn>, maxn>;

static constexpr struct mat: bmat {
	constexpr mat(): bmat {} {}

	static constexpr mat identity() {
		mat ret;
		for(int i = 0; i < maxn; i++) {
			ret[i][i] = 1;
		}
		return ret;
	}
} imat(mat::identity());

mat operator*(const mat &a, const mat &b) {
	mat ret;
	for(int i = 0; i < maxn; i++) {
		for(int j = 0; j < maxn; j++) {
#pragma GCC ivdep
			for(int k = 0; k < maxn; k++) {
				ret[i][k] = (ret[i][k] + a[i][j] * b[j][k]) % mod;
			}
		}
	}
	return ret;
}

mat mpow(mat base, int exp) {
	int n = sz(base);
	mat ret(imat);
	for(int i = 0; i < n; i++) {
		ret[i][i] = 1;
	}
	while(exp) {
		if(exp & 1) {
			ret = ret * base;
		}
		base = base * base;
		exp >>= 1;
	}
	return ret;
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	int on = 0, arr[n];
	{
		string s, t;
		cin >> s >> t;
		for(int i = 0; i < n; i++) {
			on += arr[i] = s[i] ^ t[i];
		}
	}
	mat base;
	long tways = inv(choose(n, m));
	for(int i = 0; i <= n; i++) {
		for(int j = 0; j <= n; j++) {
			if(!((i ^ j ^ m) & 1)) {
				int change = (i - j + m) / 2;
				base[i][j] = choose(i, change) * choose(n - i, m - change) % mod;
				base[i][j] = base[i][j] * tways % mod;
			}
		}
	}
	long prob = mpow(base, k)[0][on];
	cout << prob * inv(choose(n, on)) % mod << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	pcomp();
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}