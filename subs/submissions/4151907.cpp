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
	int n, m;
	cin >> n >> m;
	string arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			char a = arr[i][j], b = arr[i][m - j - 1];
			if (a != '.' && b != '.' && a != b) {
				cout << -1 << endl;
				return;
			} else if (a != '.') {
				arr[i][m - j - 1] = a;
			} else if (b != '.') {
				arr[i][j] = b;
			}
		}
	}
	for (int j = 0; j < m; j++) {
		for (int i = 0; i < n; i++) {
			char a = arr[i][j], b = arr[n - i - 1][j];
			if (a != '.' && b != '.' && a != b) {
				cout << -1 << endl;
				return;
			} else if (a != '.') {
				arr[n - i - 1][j] = a;
			} else if (b != '.') {
				arr[i][j] = b;
			}
		}
	}
	for (int i = 0; i < n; i++) {
		for (auto& a : arr[i]) {
			if (a == '.') {
				a = 'a';
			}
		}
	}
	for (int i = 0; i < n; i++) {
		cout << arr[i] << endl;
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