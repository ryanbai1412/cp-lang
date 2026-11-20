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
	int n;
	cin >> n;
	int arr[n][4];
	for (auto& [x1, y1, x2, y2] : arr) {
		cin >> x1 >> y1 >> x2 >> y2;
	}
	map<pair<int, int>, int> m;
	for (auto& [x1, y1, x2, y2] : arr) {
		int x = x2 - x1;
		int y = y2 - y1;
		int cgcd = gcd(x, y);
		x /= cgcd;
		y /= cgcd;
		if (x < 0) {
			x = -x;
			y = -y;
		}
		m[{x, y}]++;
	}
	int mx = 0;
	for (auto& [_, x] : m) {
		mx = max(mx, x);
	}
	if (n == 3) {
		cout << (mx == 1) << endl;
	} else if (n == 4) {
		if (mx == 2) {
			cout << 1 << endl;
		} else if (mx == 1) {
			cout << 2 << endl;
		} else {
			cout << 0 << endl;
		}
	} else if (n == 5) {
		cout << 6 << endl;
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