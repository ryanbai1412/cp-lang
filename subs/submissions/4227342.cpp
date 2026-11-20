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
	int n, k;
	cin >> n >> k;
	int m = min(n, k);
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	if (!k) {
		for (int i = 0; i < n; i++) {
			cout << arr[i] << " \n"[i == n - 1];
		}
		return;
	}
	vector<int> ds;
	{
		int j = 0, csum = 0;
		for (int i = n - 1; i >= 0 && j < m; i--) {
			if (csum += arr[i]) {
				csum--;
			} else {
				ds.push_back(j);
			}
			j++;
		}
		reverse(begin(ds), end(ds));
	}
	for (int i = 0; i < n; i++) {
		int pi = arr[i];
		arr[i] -= int(ds.end() - upper_bound(begin(ds), end(ds), m - i, greater<>()));
		arr[i] = max(arr[i], 0);
		if (!binary_search(begin(ds), end(ds), m - i - 1, greater<>())) {
			arr[i] = max(arr[i], 1);
		}
		ds.push_back(-i - 1);
		for (int j = 0; j < pi; j++) {
			if (sz(ds)) {
				ds.pop_back();
			}
		}
	}
	if (k > n && !*min_element(arr, arr + n)) {
		rotate(arr, arr + n - k % n, arr + n);
	}
	for (int i = 0; i < n; i++) {
		cout << arr[i] << " \n"[i == n - 1];
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