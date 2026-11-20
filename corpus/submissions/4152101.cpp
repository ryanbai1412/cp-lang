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

pair<int, int> rotate(int x1, int y1, int x2, int y2, int x, int y) {
	if (x1 <= x && x <= x2 && y1 <= y && y <= y2) {
		return {x1 + y - y1, y2 - (x - x1)};
	}
	return {x, y};
}

void solve() {
	int n, k, q;
	cin >> n >> k >> q;
	int arr[k][4];
	for (auto& [x1, y1, x2, y2] : arr) {
		cin >> x1 >> y1 >> x2 >> y2;
	}
	while (q--) {
		int x, y;
		cin >> y >> x;
		for (int i = k - 1; i >= 0; i--) {
			auto [x1, y1, x2, y2] = arr[i];
			tie(x, y) = rotate(x1, y1, x2, y2, x, y);
		}
		cout << (y - 1) * n + x << endl;
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