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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n;
	cin >> n;
	ld ans = 1;
	for (int i = 0; i < n; i++) {
		char c;
		cin >> c;
		if (c == 'A') {
			ans *= 1;
		} else if (c == 'B') {
			ans *= 0.8;
		} else if (c == 'C') {
			ans *= 0.6;
		} else if (c == 'D') {
			ans *= 0.4;
		} else {
			ans *= 0.2;
		}
	}
	cout << ans << endl;
}

int main() {
	cout << fixed << setprecision(6);
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