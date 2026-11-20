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

template <long mod>
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

template <long mod>
long inv(long x) {
	return bpow<mod>(x, mod - 2);
}

const int maxn = 2e5;

template <long mod>
struct Hasher {
	constexpr static long base = 31, ibase = inv<mod>(base);
	long ipow[maxn + 1] {}, psum[maxn + 1] {};

	explicit Hasher(const string &s) {
		long cpow = 1;
		ipow[0] = 1;
		for (int i = 0; i < sz(s); i++) {
			psum[i + 1] = (psum[i] + (s[i] - 'a' + 1) * cpow) % mod;
			cpow = (cpow * base) % mod;
			ipow[i + 1] = inv<mod>(cpow);
		}
	}

	int query(int l, int r) const {
		return int((psum[r] - psum[l] + mod) * ipow[l] % mod);
	}
};

struct MHasher {
	using Node = array<int, 3>;

	static constexpr long mod1 = 1e9 + 7, mod2 = 1e9 + 9, mod3 = 998244353;
	Hasher<mod1> h1;
	Hasher<mod2> h2;
	Hasher<mod3> h3;

	explicit MHasher(const string &s): h1(s), h2(s), h3(s) {}
	
	Node query(int l, int r) const {
		return {h1.query(l, r), h2.query(l, r), h3.query(l, r)};
	}
};

void solve() {
	string s, t;
	cin >> s >> t;
	int n = sz(s), m = sz(t);
	if (n > m) {
		return;
	}
	int tcnt[26] {};
	for (auto& a : s) {
		tcnt[a - 'a']++;
	}
	int ccnt[26] {};
	for (int i = 0; i < n - 1; i++) {
		ccnt[t[i] - 'a']++;
	}
	set<MHasher::Node> ts;
	MHasher hasher(t);
	for (int i = n - 1; i < m; i++) {
		ccnt[t[i] - 'a']++;
		if (i - n >= 0) {
			ccnt[t[i - n] - 'a']--;
		}
		if (equal(begin(ccnt), end(ccnt), tcnt)) {
			ts.insert(hasher.query(i - n + 1, i + 1));
		}
	}
	cout << sz(ts) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}