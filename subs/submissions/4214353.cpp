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

void solve() {
	string s;
	cin >> s;
	int n = sz(s);
	int q;
	cin >> q;
	while (q--) {
		string t;
		int k;
		cin >> t >> k;
		int m = sz(t);
		int cnt = 0;
		for (int i = 0; i + m <= n; i++) {
			if (s.substr(i, m) == t && ++cnt == k) {
				cout << i + 1 << endl;
				goto loop;
			}
		}
		cout << -1 << endl;
		loop:;
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