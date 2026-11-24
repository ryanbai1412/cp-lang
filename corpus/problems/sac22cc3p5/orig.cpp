// Reference: matches a brute force on the --max case; most other AC submissions fail at the full bounds.
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

struct custom_hash {
	uint64_t operator()(uint64_t x) const {
		x ^= x << 49;
		x ^= x >> 23;
		x ^= x << 37;
		return x;
	}
};

namespace __gnu_pbds {
template <typename T>
using set = gp_hash_table<T, null_type, custom_hash>;
}

const long mod = 1712485131884479, base = 31;

long hsh(const string& s) {
	long ans = 0;
	for (auto& a : s) {
		ans = (ans * base + a - 'a' + 1) % mod;
	}
	return ans;
}

string fix(string s) {
	for (auto& a : s) {
		if ('A' <= a && a <= 'Z') {
			a = char('a' + a - 'A');
		}
	}
	return s;
}

void solve() {
	int n, m;
	cin >> n >> m;
	__gnu_pbds::set<long> bad;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		s = fix(s);
		bad.insert(hsh(s));
	}
	assert(sz(bad) == n);
	for (int i = 0; i < m; i++) {
		string s;
		cin >> s;
		s = fix(s);
		__gnu_pbds::set<long> cur;
		for (int j = 0; j < sz(s); j++) {
			long chsh = 0;
			for (int k = 1; j + k <= sz(s) && k <= 25; k++) {
				chsh = (chsh * base + s[j + k - 1] - 'a' + 1) % mod;
				if (bad.find(chsh) != bad.end()) {
					cur.insert(chsh);
				}
			}
		}
		cout << sz(cur) << endl;
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