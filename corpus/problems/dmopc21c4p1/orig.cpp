// AC submission 4125256 with one fix: a point strictly inside the horizontal leg (y == y1) tests
// x against x1..x3, not x1..x2 (x2 = x1), so it counts as boundary. The judge's data has no such point.
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
	int n, q;
	cin >> n >> q;
	int arr[n][6];
	for (auto& [x1, y1, x2, y2, x3, y3] : arr) {
		cin >> x1 >> y1 >> x2 >> y2 >> x3 >> y3;
	}
	auto irange = [&](int l, int r, int x) -> bool {
		if (l > r) {
			swap(l, r);
		}
		return l <= x && x <= r;
	};
	while (q--) {
		int x, y;
		cin >> x >> y;
		int ans = 0;
		for (auto [x1, y1, x2, y2, x3, y3] : arr) {
			if (x == x1) {
				ans += irange(y1, y2, y);
			} else if (y == y1) {
				ans += irange(x1, x3, x);
			} else if ((x > x1) == (x3 > x1) && (y > y1) == (y2 > y1)) {
				long c = abs(long(x - x1) * (y2 - y1)) + abs(long(y - y1) * (x3 - x1));
				if (c <= abs(long(y2 - y1) * (x3 - x1))) {
					ans++;
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