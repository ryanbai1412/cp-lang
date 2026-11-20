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
	int n, m, s, f, l, r;
	cin >> n >> m >> s >> f >> l >> r;
	int arr[m];
	for (auto& a : arr) {
		cin >> a;
	}
	int ind = -1;
	for (int i = 0; i < m; i++) {
		if (arr[i] == f) {
			ind = i;
		}
	}
	if (ind == -1) {
		if (s != f) {
			cout << -1 << endl;
			return;
		}
		if (l == 0) {
			cout << l << endl;
			for (int a = 0; a < n; a++) {
				for (int b = 0; b < m; b++) {
					cout << 0 << " \n"[b == m - 1];
				}
			}
			return;
		}
		int cnt = 0;
		bool ans[n][m] {};
		for (int i = 0; i < n; i++) {
			if (i != s) {
				for (int j = 0; j < m; j++) {
					ans[i][j] = true;
					if (++cnt == l) {
						cout << l << endl;
						for (int a = 0; a < n; a++) {
							for (int b = 0; b < m; b++) {
								cout << ans[a][b] << " \n"[b == m - 1];
							}
						}
						return;
					}
				}
			}
		}
		cout << -1 << endl;
		return;
	}
	if (r == 0) {
		if (s == f) {
			cout << 0 << endl;
			for (int a = 0; a < n; a++) {
				for (int b = 0; b < m; b++) {
					cout << 0 << " \n"[b == m - 1];
				}
			}
		} else {
			cout << -1 << endl;
		}
		return;
	}
	if (l <= n) {
		assert(false);
		bool ans[n][m] {};
		ans[s][ind] = true;
		int cnt = 1;
		if (cnt < l) {
			for (int i = 0; i < n; i++) {
				if (i != s) {
					ans[i][s] = true;
					if (++cnt == l) {
						break;
					}
				}
			}
		}
		cout << cnt << endl;
		for (int a = 0; a < n; a++) {
			for (int b = 0; b < m; b++) {
				cout << ans[a][b] << " \n"[b == m - 1];
			}
		}
		return;
	}
	int cnt = 0;
	bool ans[n][m] {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (i == s && j > ind) {
				break;
			}
			ans[i][j] = true;
			if (++cnt == l) {
				cout << l << endl;
				for (int a = 0; a < n; a++) {
					for (int b = 0; b < m; b++) {
						cout << ans[a][b] << " \n"[b == m - 1];
					}
				}
				return;
			}
		}
	}
	cout << -1 << endl;
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