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

const long mod = 65521, infmod = long(4e18) / mod * mod;

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

struct mask {
	int x;

	bool operator[](int i) {
		return !!((x >> i) & 1);
	}
};

struct BM {
	int l, m, b, n;
	vector<long> s, c, bv;

	BM(): l(0), m(1), b(1), n(0), s {}, c {1}, bv {1} {}

	void update(long x) {
		s.push_back(x);

		for(int i = 1; i <= l; i++) {
			x = (x + c[i] * s[n - i]) % mod;
		}

		if(x == 0) {
			++m;
		}else if(2 * l <= n) {
			vector<long> tmp = c;

			c.resize(n + 1 - l + 1);
			for(int i = 0; i < sz(bv); i++) {
				c[i + m] = (c[i + m] + infmod - x * inv(b) * bv[i]) % mod;
			}

			l = n + 1 - l;
			m = 1;
			b = x;
			bv = tmp;
		}else {
			for(int i = 0; i < bv.size(); i++) {
				c[i + m] = (c[i + m] + infmod - x * inv(b) * bv[i]) % mod;
			}

			m++;
		}

		n++;
	}
};

long solve(int k, int n) {
	n--;
	auto get = [&](int u, int v) -> int {
		if(!(0 <= u && u < n && 0 <= v && v < n)) {
			return 0;
		}
		if(u == v) {
			return min(k, u) + min(k, n - u);
		}else if(abs(u - v) <= k) {
			return -1;
		}
		return 0;
	};
	auto next = [&](int j, int ind) -> int {
		return (j | (1 << ind)) >> 1;
	};
	auto conv = [&](int ind, int rel) -> int {
		return ind + rel - k;
	};
	int m = (1 << (2 * k));
	long dp[n + 1][m];
	memset(dp[n], 0, sizeof(dp[n]));
	dp[n][(1 << k) - 1] = 1;
	for(int i = n - 1; i >= 0; i--) {
		for(int ji = 0; ji < m; ji++) {
			if(__builtin_popcount(ji) != k) {
				continue;
			}
			mask j {ji};
			if(!j[0]) {
				int psum = 0;
				for(int a = 2 * k; a >= 1; a--) {
					psum += j[a];
				}
				int mul = 1;
				if(psum & 1) {
					mul = -1;
				}
				dp[i][ji] = mul * get(i, conv(i, 0)) * dp[i + 1][next(ji, 0)];
			}else {
				dp[i][ji] = 0;
				int psum = 0;
				for(int a = 2 * k; a >= 0; a--) {
					if(!j[a]) {
						int mul = 1;
						if(psum & 1) {
							mul = -1;
						}
						dp[i][ji] += mul * get(i, conv(i, a)) * dp[i + 1][next(ji, a)];
					}else {
						psum++;
					}
				}
				dp[i][ji] += infmod;
				dp[i][ji] %= mod;
			}
		}
	}
	return dp[0][(1 << k) - 1];
}

using mat = vector<vector<long>>;

mat mul(const mat &a, const mat &b) {
	int x = sz(a), y = sz(b), z = sz(b[0]);
	mat c(x, vector<long>(z));

	for(int i = 0; i < x; i++) {
		for(int j = 0; j < y; j++) {
			for(int k = 0; k < z; k++) {
				c[i][k] = (c[i][k] + a[i][j] * b[j][k]) % mod;
			}
		}
	}

	return c;
}

mat bpow(mat base, long exp) {
	int n = sz(base);
	mat ans(n, vector<long>(n));
	for(int i = 0; i < n; i++) {
		ans[i][i] = 1;
	}

	while(exp) {
		if(exp & 1) {
			ans = mul(ans, base);
		}
		base = mul(base, base);
		exp >>= 1;
	}
	return ans;
}

void solve() {
	BM bm {};
	long k, n;
	cin >> k >> n;
	if(n <= 100) {
		cout << solve(k, n) << endl;
		return;
	}
	long dp[100 + 1] {};
	for(int i = k; i <= 100; i++) {
		bm.update(dp[i] = solve(k, i));
	}
	vector<long> poly(bm.c.begin() + 1, bm.c.end());
	{
		int m = sz(poly);
		mat matrix(m, vector<long>(m));

		for(int i = 0; i < m - 1; i++) {
			matrix[i][i + 1] = 1;
		}
		for(int i = 0; i < m; i++) {
			matrix[i][0] = mod - poly[i];
		}

		vector<long> start(m);
		for(int i = 0; i < m; i++) {
			start[m - i - 1] = dp[i + k];
		}

		mat ans = mul(mat(1, start), bpow(matrix, n - m - 2));
		dbg(solve(k, n));
		cout << ans[0][0] << endl;
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