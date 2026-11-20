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

vector<vector<int>> rotate(const vector<vector<int>>& arr) {
	int n = sz(arr);
	vector<vector<int>> ans(n, vector<int>(n));
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			ans[j][n - i - 1] = arr[i][j];
		}
	}
	return ans;
}

bool check(const vector<vector<int>>& arr) {
	for (auto& a : arr) {
		int prev = -1;
		for (auto& b : a) {
			if (prev >= b) {
				return false;
			}
			prev = b;
		}
	}
	int n = sz(arr);
	for (int i = 0; i < n - 1; i++) {
		for (int k = 0; k < n; k++) {
			if (arr[i][k] >= arr[i + 1][k]) {
				return false;
			}
		}
	}
	return true;
}

void solve() {
	int n;
	cin >> n;
	vector<vector<int>> arr(n, vector<int>(n));
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	for (int i = 0; i < 4; i++) {
		if (check(arr)) {
			for (auto& a : arr) {
				for (int j = 0; j < n; j++) {
					cout << a[j] << " \n"[j == n - 1];
				}
			}
			return;
		}
		arr = rotate(arr);
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