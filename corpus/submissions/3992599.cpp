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
	int n, m, q;
	cin >> n >> m;
	vector<string> arr(n);
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	auto movel = [&]() -> bool {
		bool changed = false;
		for (int i = 0; i < n; i++) {
			for (int j = 1; j < m; j++) {
				if (arr[i][j] == 'W') {
					for (int k = j - 1; k >= 0; k--) {
						if (arr[i][k] == '.') {
							changed = true;
							swap(arr[i][k], arr[i][k + 1]);
						} else {
							break;
						}
					}
				}
			}
		}
		return changed;
	};
	auto moved = [&]() -> bool {
		bool changed = false;
		auto prev = arr;
		for (int i = 0; i < n - 1; i++) {
			for (int j = 0; j < m; j++) {
				if (prev[i][j] == 'W' && prev[i + 1][j] == '.') {
					changed = true;
					swap(arr[i + 1][j], arr[i][j]);
				}
			}
		}
		return changed;
	};
	cin >> q;
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			for (int i = n - 1; i >= 0; i--) {
				for (int j = 0; j < m; j++) {
					if (arr[i][j] == 'X') {
						arr[i][j] = '.';
						if (i + 1 < n) {
							arr[i + 1][j] = 'X';
						}
					}
				}
			}
			while(true) {
				bool changed = false;
				changed |= movel();
				changed |= moved();
				if (!changed) {
					break;
				}
			}
		} else {
			for (auto& a : arr) {
				cout << a << endl;
			}
		}
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