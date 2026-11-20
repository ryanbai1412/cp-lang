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
	int n, m, k;
	cin >> n >> m >> k;
	int psum[n + 1][m + 1] {};
	for (int i = 0; i < k; i++) {
		int x1, y1, x2, y2;
		cin >> x1 >> y1 >> x2 >> y2;
		x1--;
		y1--;
		psum[x1][y1]++;
		psum[x2][y1]--;
		psum[x1][y2]--;
		psum[x2][y2]++;
	}
	vector<vector<int>> arr(n + 1, vector<int> (m + 1));
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			arr[i + 1][j + 1] = psum[i][j] + arr[i][j + 1] + arr[i + 1][j] - arr[i][j];
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			arr[i][j] = arr[i + 1][j + 1];
		}
	}
	for (auto& a : arr) {
		for (auto& b : a) {
			b = !!b;
		}
	}
	auto valid = [&](int x) -> bool {
		x = 1 << x;
		if (n % x != 0 || m % x != 0) {
			return false;
		}
		for (int i = 0; i < n; i += x) {
			for (int j = 0; j < m; j += x) {
				int cur = arr[i][j];
				for (int a = 0; a < x; a++) {
					for (int b = 0; b < x; b++) {
						if (arr[i + a][j + b] != cur) {
							dbg(i, j, a, b, arr[i][j], arr[i + a][j + b]);
							return false;
						}
					}
				}
			}
		}
		return true;
	};
	dbg(valid(1));
	int l = 0, r = 32;
	while (l < r) {
		int mid = (l + r + 1) / 2;
		if (valid(mid)) {
			l = mid;
		} else {
			r = mid - 1;
		}
	}
	cout << l << endl;
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