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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

constexpr long bpow(long base, long exp, long mod) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		exp >>= 1;
		base = (base * base) % mod;
	}
	return ans;
}
 
constexpr long inv(long x, long mod) {
	return bpow(x, mod - 2, mod);
}
 
template <long base, long mod>
struct Pow {
	static constexpr int maxn = 3e3 + 5;
	long arr[maxn];
 
	constexpr Pow() : arr() {
		arr[0] = 1;
		for (int i = 1; i < maxn; i++) {
			arr[i] = (arr[i - 1] * base) % mod;
		}
	}
 
	long operator[](int i) const {
		return arr[i];
	}
};
 
template <long base, long mod>
struct Hasher {
	static constexpr long ibase = inv(base, mod);
	static constexpr Pow<base, mod> pow {};
	static constexpr Pow<ibase, mod> ipow {};
	vector<long> psum;
 
	explicit Hasher(const string& s) : psum(sz(s) + 1) {
		for (int i = 0; i < sz(s); i++) {
			psum[i + 1] = (psum[i] + (s[i] - 'a' + 1) * pow[i]) % mod;
		}
	}
 
	long hash(int l, int r) const {
		return (psum[r] - psum[l] + mod) * ipow[l] % mod;
	}
 
	long merge(int len, long a, long b) const {
		return (a + b * pow[len]) % mod;
	}
 
	long rot(int len, long h) const {
		long x = h % base;
		return ((h - x + mod) * ibase + x * pow[len - 1]) % mod;
	}
};

const long base = 31, base2 = 29, mod = 1e9 + 7;

template <bool inc>
struct Solver {
	int n, m, i;
	string s, t;
	vector<int> g, ans;
	Hasher<base, mod> hs, ht;

	Solver(const string& s, const string& t)
		: n(sz(s)), m(sz(t)), i(inc ? 0 : n - 1), s(s), t(t), g(m + 1), ans(m + 1), hs(s), ht(t) {}

	void adv() {
		if (i < 0 || i >= n) {
			return;
		}
		if (inc) {
			for (int j = 0; j < m; j++) {
				if (s[i] != t[j]) {
					g[j] = 0;
					continue;
				}
				int l = 0, r = min(n - i, m - j);
				while (l < r) {
					int mid = (l + r + 1) / 2;
					if (hs.hash(i, i + mid) == ht.hash(j, j + mid)) {
						l = mid;
					} else {
						r = mid - 1;
					}
				}
				g[j] = l;
			}
		} else {
			for (int j = 0; j < m; j++) {
				if (s[i] == t[j]) {
					g[j] = g[j + 1] + 1;
				} else {
					g[j] = 0;
				}
			}
		}
		int opt[m + 1];
		memset(opt, 0x3f, sizeof(opt));
		for (int j = 0; j < m; j++) {
			int& o = opt[g[j] + j];
			o = min(o, j);
		}
		int o = 1e9;
		for (int j = m - 1; j >= 0; j--) {
			o = min(o, opt[j + 1]);
			ans[j] = max(0, j - o + 1);
		}
		if (inc) {
			i++;
		} else {
			i--;
		}
	}
};

string reversed(string s) {
	reverse(begin(s), end(s));
	return s;
}

vector<vector<int>> comp(string s, string t) {
	int n = sz(s), m = sz(t);
	int g[n + 1][m + 1] {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (s[i] == t[j]) {
				if (i && j) {
					g[i][j] = g[i - 1][j - 1];
				}
				g[i][j]++;
			}
		}
	}
	vector<vector<int>> ans(n, vector<int>(m));
	for (int i = 0; i < n; i++) {
		int opt[m + 1] {};
		for (int j = 0; j < m; j++) {
			int& o = opt[j - g[i][j] + 1];
			o = max(o, j);
		}
		int o = 0;
		for (int j = 0; j < m; j++) {
			o = max(o, opt[j]);
			ans[i][j] = max(0, o - j + 1);
		}
	}
	return ans;
}

tuple<int, int, int> solve(string s, string t) {
	int n = sz(s), m = sz(t);
	Solver<false> s1(s, t);
	Solver<true> s2(reversed(s), reversed(t));
	tuple<int, int, int> ans {};
	s2.adv();
	for (int i = n - 1; i >= 0; i--) {
		s1.adv();
		s2.adv();
		for (int j = 0; j < m; j++) {
			int p1 = s1.ans[j], p2 = (i && j + 1 < m) ? s2.ans[m - j - 2] : 0;
			if (p1 + p2 == 3) {
				dbg(i, j, p1, p2, s2.g[0], s2.g[1], s2.g[2]);
			}
			ans = max(ans, tuple<int, int, int> {p1 + p2, i - p2, j - p1 + 1});
		}
	}
	return ans;
}

void solve() {
	string s, t;
	cin >> s >> t;
	auto [x, a, b] = solve(s, t);
	reverse(begin(s), end(s));
	auto [x1, a1, b1] = solve(s, t);
	dbg(x, a, b, x1, a1, b1);
	if (x1 > x) {
		x = x1;
		a = sz(s) - a1 - x;
		b = b1;
	}
	cout << x << endl;
	cout << a << " " << b << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}