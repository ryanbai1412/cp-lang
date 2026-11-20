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
	int arr[n][n] {}, psum[n][n + 1] {};
	long ans = 0;
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			int x, y;
			cin >> x >> y;
			x--;
			y--;
			arr[x][y]++;
			partial_sum(arr[x], arr[x] + n, psum[x] + 1);
		} else {
			int x1, y1, x2, y2;
			cin >> x1 >> y1 >> x2 >> y2;
			x1--;
			y1--;
			for (int i = x1; i < x2; i++) {
				ans += psum[i][y2] - psum[i][y1];
			}
		}
	}
	cout << ans << endl;
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