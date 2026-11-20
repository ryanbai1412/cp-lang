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

int ctoi(char c) {
	switch (c) {
		case 'R': return 0;
		case 'G': return 1;
		case 'B': return 2;
		default: assert(false);
	}
}

long solve(const string& a, const string &b, const string &c, const string &d) {
	int n = sz(a);
	int cnt[3] {};
	long ans = 0;
	for (int i = 0; i < n; i++) {
		int cx = ctoi(c[i]), dx = ctoi(d[i]);
		cnt[cx]++;
		if (cx != dx) {
			cnt[dx]++;
		}
	}
	for (int i = 0; i < n; i++) {
		int ax = ctoi(a[i]), bx = ctoi(b[i]);
		if (ax != bx) {
			int targ = 0 ^ 1 ^ 2 ^ ax ^ bx;
			ans += cnt[targ];
		}
	}
	return ans;
}

long sx(const string& a, const string &b, const string &c, const string &d) {
	int n = sz(a);
	int cnt[3] {};
	long ans = 0;
	for (int i = 0; i < n; i++) {
		int cx = ctoi(c[i]), dx = ctoi(d[i]);
		if (cx != dx) {
			cnt[cx]++;
			cnt[dx]++;
		}
	}
	for (int i = 0; i < n; i++) {
		int ax = ctoi(a[i]), bx = ctoi(b[i]);
		if (ax != bx) {
			int targ = 0 ^ 1 ^ 2 ^ ax ^ bx;
			ans += cnt[targ];
		}
	}
	return ans;
}

void solve() {
	int n;
	cin >> n;
	string a, b, c, d;
	cin >> a >> c >> b >> d;
	cout << solve(a, b, c, d) + solve(c, d, a, b) - sx(a, b, c, d) << endl;
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