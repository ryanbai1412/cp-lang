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

const long mod = 1e9 + 7;
const int maxn = 305;

long pow2[maxn + 1];

void pcomp() {
	pow2[0] = 1;
	for (int i = 0; i < maxn; i++) {
		pow2[i + 1] = (pow2[i] * 2) % mod;
	}
}

long solve1d(const string& s) {
	dbg(s);
	long ans = 1;
	for (int i = 0; i < sz(s);) {
		int l = i;
		for (; i < sz(s) && s[i] == s[l]; i++)
			;
		dbg(i - l);
		ans = (ans * pow2[i - l - 1]) % mod;
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	assert(n == 1);
	string s;
	cin >> s;
	for (int i = 0; i < m; i++) {
		char c = s[i];
		if (c == 'N') {
			s[i] = 'S';
		} else {
			s[i] = 'N';
		}
		cout << solve1d(s) << " \n"[i == m - 1];
		s[i] = c;
	}
}

int main() {
	pcomp();
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