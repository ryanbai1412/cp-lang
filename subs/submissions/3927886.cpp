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

const long mod = 998244353;

void solve(string s) {
	reverse(begin(s), end(s));
	long num = 1, den = 1;
	for (auto& a : s) {
		long sum = num + den;
		if (sum >= mod) {
			sum -= mod;
		}
		if (a == 'W') {
			den = sum;
		} else {
			num = sum;
		}
	}
	cout << num << " " << den << endl;
}

void solve() {
	int n, q;
	cin >> n >> q;
	string s;
	cin >> s;
	solve(s);
	while(q--) {
		string t;
		cin >> t;
		if (t == "APPEND") {
			char c;
			cin >> c;
			s += c;
		} else if(t == "FLIP") {
			int l, r;
			cin >> l >> r;
			l--;
			for (int i = l; i < r; i++) {
				if (s[i] == 'W') {
					s[i] = 'E';
				} else {
					s[i] = 'W';
				}
			}
		} else {
			int l, r;
			cin >> l >> r;
			l--;
			reverse(s.begin() + l, s.begin() + r);
		}
		solve(s);
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}