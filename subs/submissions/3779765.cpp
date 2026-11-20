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

template<int mod>
constexpr long bpow(long base, long exp) {
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

template<int mod>
constexpr long inv(long x) {
	return bpow<mod>(x, mod - 2);
}

template<int mod>
struct Hash {
	static constexpr int base = 67, ibase = inv<mod>(base);
	vector<long> pow, ipow, psum;

	Hash(const string &s) {
		int n = sz(s);
		pow.resize(n + 1);
		ipow.resize(n + 1);
		psum.resize(n + 1);
		pow[0] = ipow[0] = 1;
		for(int i = 0; i < n; i++) {
			pow[i + 1] = (pow[i] * base) % mod;
			ipow[i + 1] = (ipow[i] * ibase) % mod;
			psum[i + 1] = (psum[i] + conv(s[i]) * pow[i]) % mod;
		}
	}

	int conv(char c) {
		if(isupper(c)) {
			return c - 'A';
		}else if(islower(c)) {
			return 26 + c - 'a';
		}
		return 52 + c - '0';
	}

	long hash(int l, int r) {
		return ((psum[r] - psum[l] + mod) * ipow[l]) % mod;
	}
};

struct SA {
	static constexpr int mod1 = 1e9 + 7, mod2 = 1e9 + 9;
	int n;
	string s;
	Hash<mod1> h1;
	Hash<mod2> h2;
	vector<int> arr, l;

	SA(const string &_s): n(sz(_s)), s(_s), h1(s), h2(s), arr(n), l(n) {
		iota(begin(arr), end(arr), 0);
		sort(begin(arr), end(arr), [&](int a, int b) -> bool {
			return cmp(a, b);
		});
		for(int i = 0; i < n - 1; i++) {
			l[i] = lcp(arr[i], arr[i + 1]);
		}
	}

	bool eq(int a, int b, int len) {
		return a + len <= n && b + len <= n
			   && h1.hash(a, a + len) == h1.hash(b, b + len)
			   && h2.hash(a, a + len) == h2.hash(b, b + len);
	}

	int lcp(int a, int b) {
		int l = 0, r = n - 1;
		while(l < r) {
			int mid = (l + r + 1) / 2;
			if(eq(a, b, mid)) {
				l = mid;
			}else {
				r = mid - 1;
			}
		}
		return l;
	}

	bool cmp(int a, int b) {
		if(a == b) {
			return false;
		}
		int l = lcp(a, b);
		if(max(a, b) + l == n) {
			return a + l == n;
		}
		return s[a + l] < s[b + l];
	}
};

struct DS {
	int sz = 0;
	long cnt = 0, best = LONG_MIN;

	void maintain() {
		cnt = sz * (sz - 1) / 2;
	}

	void add(int x) {
		sz++;
		maintain();
	}

	void merge(const DS &ds) {
		sz += ds.sz;
		maintain();
	}

	long gbest() {
		return cnt ? best : 0;
	}
};

struct DSU {
	static constexpr DS null {};

	vector<int> p;
	vector<DS> rep;
	long cnt, best;

	DSU(int n): p(n, -1), rep(n, null), cnt(0), best(LONG_MIN) {}

	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}

	void merge(int u, int v) {
		u = find(u);
		v = find(v);
		if(u == v) {
			return;
		}
		if(p[u] < p[v]) {
			swap(u, v);
		}
		cnt -= rep[u].cnt;
		cnt -= rep[v].cnt;
		p[v] += p[u];
		p[u] = v;
		rep[v].merge(rep[u]);
		rep[u] = null;
		cnt += rep[v].cnt;
		best = max(best, rep[v].best);
	}
};

void solve() {
	int n;
	string s;
	cin >> n >> s;
	long arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	SA sa(s);
	vector<int> change[n];
	for(int i = 0; i < n - 1; i++) {
		change[sa.l[i]].push_back(i);
	}
	DSU dsu(n);
	for(int i = 0; i < n; i++) {
		dsu.rep[i].add(arr[i]);
	}
	int ans[n][2];
	bool active[n] {};
	for(int i = n - 1; i >= 0; i--) {
		for(auto &a: change[i]) {
			dsu.merge(a, a + 1);
		}
		ans[i][0] = dsu.cnt;
		ans[i][1] = dsu.best;
	}
	for(int i = 0; i < n; i++) {
		cout << ans[i][0] << " " << !!ans[i][0] * arr[0] * arr[0] << endl;
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