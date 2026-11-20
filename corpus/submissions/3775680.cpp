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

constexpr int pows[] {1, 3, 9, 27, 81, 243, 729, 2187, 6564, 19692};

template<int n>
struct sieve {
	int prime[n + 1];

	constexpr sieve(): prime {} {
		for(int i = 0; i <= n; i++) {
			prime[i] = 1;
		}
		prime[0] = prime[1] = 0;
		prime[2] = 1;
		int pind = 0;
		for(int i = 2; i <= n; i++) {
			if(prime[i]) {
				prime[i] = ++pind;
			}
			for(int j = i * i; j <= n; j += i) {
				prime[j] = 0;
			}
		}
	}

	int operator[](int i) const {
		return prime[i];
	}
};

struct mask {
	int x;

	inline mask set(int ind, int v) {
		return mask {x + (v - (*this)[ind]) * pows[ind]};
	}

	inline int operator[](int ind) {
		return (x / pows[ind]) % 3;
	}
};

struct mask2 {
	int x;

	inline bool operator[](int ind) {
		return !!((x >> ind) & 1);
	}
};

void solve() {
	static constexpr sieve<500> prime {};
	int n;
	long mod;
	cin >> n >> mod;
	vector<int> primes;
	for(int i = 2; i <= 500; i++) {
		if(prime[i]) {
			primes.push_back(i);
		}
	}
	int sqn;
	for(sqn = 0; sqn < sz(primes) - 1 && primes[sqn] * primes[sqn + 1] <= n; sqn++) ;

	vector<int> fact[n + 1];
	for(int i = 2; i <= n; i++) {
		if(prime[i]) {
			for(int j = i; j <= n; j += i) {
				fact[j].push_back(i);
			}
		}
	}

	auto bpow = [&](long base, int exp) -> long {
		long ans = 1;
		while(exp) {
			if(exp & 1) {
				ans = (ans * base) % mod;
			}
			base = (base * base) % mod;
			exp >>= 1;
		}
		return ans;
	};

	long ans = 0;
	long ways[pows[sqn]];
	for(int ii = 0; ii < pows[sqn]; ii++) {
		mask i {ii};
		long &way = ways[ii];
		way = 1;
		bool vis[n + 1] {};
		for(int j = 2; j <= n; j++) {
			if(!vis[j]) {
				if(prime[j] && prime[j] > sqn) {
					int acnt = 0, bcnt = 0;
					for(int k = j; k <= n; k += j) {
						vis[k] = true;
						bool a = true, b = true;
						for(auto &c: fact[k]) {
							if(c == j) {
								continue;
							}
							assert(prime[c] <= sqn);
							int cur = i[prime[c] - 1];
							if(cur) {
								a &= cur == 1;
								b &= cur == 2;
							}else {
								a = b = false;
							}
						}
						acnt += a;
						bcnt += b;
					}
					// all a, all b, overcounted empty set
					long cur = (bpow(2, acnt) + bpow(2, bcnt) - 1 + mod) % mod;
					way = (way * cur) % mod;
				}else {
					for(auto &a: fact[j]) {
						assert(prime[a] && prime[a] <= sqn);
					}
					bool a = true, b = true;
					for(auto &c: fact[j]) {
						assert(prime[c] <= sqn);
						int cur = i[prime[c] - 1];
						if(cur) {
							a &= cur == 1;
							b &= cur == 2;
						}else {
							a = b = false;
						}
					}
					long cur = 1 + a + b;
					way = (way * cur) % mod;
				}
			}
		}
	}
	for(int ii = 0; ii < (1 << sqn); ii++) {
		mask2 i {ii};
		long cur = 0;
		for(int ij = 0; ij < pows[sqn]; ij++) {
			mask j {ij};
			for(int k = 0; k < sqn; k++) {
				if(i[k] && j[k]) {
					goto loop;
				}
			}
			cur += ways[j.x];
			loop:;
		}
		cur %= mod;
		cur += mod;
		cur %= mod;
		cur = (cur * bpow(2, __builtin_popcount(ii))) % mod;
		if(__builtin_popcount(ii) & 1) {
			ans -= cur;
		}else {
			ans += cur;
		}
	}
	ans %= mod;
	ans += mod;
	ans %= mod;
	assert(ans >= 0);
	cout << ans << endl;
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