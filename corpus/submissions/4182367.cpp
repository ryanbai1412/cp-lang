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

template <int mod>
struct Hash {
	static constexpr int base = 67, ibase = inv<mod>(base);
	vector<long> pow, ipow, psum;

	explicit Hash(const string& s) {
		int n = sz(s);
		pow.resize(n + 1);
		ipow.resize(n + 1);
		psum.resize(n + 1);
		pow[0] = ipow[0] = 1;
		for (int i = 0; i < n; i++) {
			pow[i + 1] = (pow[i] * base) % mod;
			ipow[i + 1] = (ipow[i] * ibase) % mod;
			psum[i + 1] = (psum[i] + conv(s[i]) * pow[i]) % mod;
		}
	}

	static int conv(char c) {
		if (isupper(c)) {
			return c - 'A';
		} else if (islower(c)) {
			return 26 + c - 'a';
		}
		return 52 + c - '0';
	}

	long hash(int l, int r) const {
		return ((psum[r] - psum[l] + mod) * ipow[l]) % mod;
	}
};

const int mod = 10000019;

int t = 1;
int vis[mod];

void solve() {
	string s;
	cin >> s;
	Hash<mod> hasher(s);
	int ans = 1;
	for (int i = 1; i <= sz(s); i++) {
		t++;
		for (int j = 0; j + i <= sz(s); j++) {
			long x = hasher.hash(j, j + i);
			ans += vis[x] != t;
			vis[x] = t;
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}