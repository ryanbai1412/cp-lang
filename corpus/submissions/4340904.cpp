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

using u64 = uint64_t;

u64 xorswap(u64 x) {
	x ^= x << 49;
	x ^= x >> 23;
	x ^= x << 37;
	return x;
}

struct custom_hash {
	u64 operator()(u64 x) const {
		return xorswap(x);
	}
};

namespace __gnu_pbds {
template <typename T>
using set = gp_hash_table<T, null_type, custom_hash>;
}

const u64 mod = 1712485131884479, base = 31;

u64 hsh(const string& s) {
	u64 ans = 0;
	for (auto& a : s) {
		ans = (ans * base + a - 'a' + 1) % mod;
	}
	return xorswap(ans);
}

string fix(string s) {
	for (auto& a : s) {
		if ('A' <= a && a <= 'Z') {
			a = char('a' + a - 'A');
		}
	}
	return s;
}

const int hsize = 1 << 19, hmask = hsize - 1, hmx = 6;
const int vsize = 1 << 21, vmask = vsize - 1;

int hind[hsize], vis[vsize];
u64 hashes[hsize][hmx];

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		s = fix(s);
		u64 x = hsh(s);
		u64 key = x & hmask;
		hashes[key][hind[key]++] = x;
	}
	for (int i = 1; i <= m; i++) {
		string s;
		cin >> s;
		s = fix(s);
		int ans = 0;
		for (int j = 0; j < sz(s); j++) {
			u64 chsh = 0;
			for (int k = 1; j + k <= sz(s) && k <= 25; k++) {
				chsh = (chsh * base + s[j + k - 1] - 'a' + 1) % mod;
				u64 x = xorswap(chsh);
				u64 key = x & hmask;
#pragma GCC unroll hmx
				for (int a = 0; a < hind[key]; a++) {
					if (hashes[key][a] == x) {
						u64 xkey = x & vmask;
						ans += vis[xkey] != i;
						vis[xkey] = i;
						break;
					}
				}
			}
		}
		cout << ans << endl;
	}
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