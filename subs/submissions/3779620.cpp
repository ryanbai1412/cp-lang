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
			psum[i + 1] = (psum[i] + (s[i] - 'a') * pow[i]) % mod;
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

void solve() {
	string s;
	cin >> s;
	int n = sz(s);
	assert(n <= 5000);
	SA sa(s);
	int m = 2 * n;
	int arr[m];
	for(int i = 0; i < m; i += 2) {
		arr[i] = n - sa.arr[i / 2];
		arr[i + 1] = sa.l[i / 2];
	}
	vector<int> change[n + 1];
	for(int i = 0; i < m; i++) {
		change[arr[i]].push_back(i);
	}
	bool alive[m];
	memset(alive, 1, sizeof(alive));
	auto get = [&](int i) -> bool {
		return 0 <= i && i < m && alive[i];
	};
	int ans = 1, csz = 1;
	for(int i = 0; i <= n; i++) {
		for(auto &a: change[i]) {
			if(get(a - 1) && get(a + 1)) {
				csz++;
			}else if(!get(a - 1) && !get(a + 1)) {
				csz--;
			}
			alive[a] = false;
		}
		ans += csz;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}