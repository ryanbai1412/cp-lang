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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const long inf = 1e17, ninf = -inf;

void solve() {
	int n, m;
	cin >> n >> m;
	long arr[n][m];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
			if (!b) {
				b = ninf;
			}
		}
	}
	if (!arr[n - 1][m - 1]) {
		arr[n - 1][m - 1] = inf;
	}
	for (int i = n - 1; i >= 0; i--) {
		for (int j = m - 1; j >= 0; j--) {
			if (arr[i][j] == ninf) {
				arr[i][j] = inf;
				if (i + 1 < n) {
					arr[i][j] = arr[i + 1][j];
				}
				if (j + 1 < m) {
					arr[i][j] = min(arr[i][j], arr[i][j + 1]);
				}
				arr[i][j]--;
			}
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			cerr << arr[i][j] << " \n"[j == m - 1];
		}
	}
	cerr << "--------------" << endl;
	for (int i = 0; i < n; i++) {
		if (!is_sorted(arr[i], arr[i] + m)) {
			cout << -1 << endl;
			return;
		}
	}
	for (auto& a : arr) {
		for (auto& b : a) {
			if (!(0 < b && b <= long(1e18))) {
				cout << -1 << endl;
				return;
			}
		}
	}
	for (int i = 0; i < m; i++) {
		long prev = 0;
		for (int j = 0; j < n; j++) {
			if (prev > arr[j][i]) {
				cout << -1 << endl;
				return;
			}
			prev = arr[j][i];
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			cout << arr[i][j] << " \n"[j == m - 1];
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}