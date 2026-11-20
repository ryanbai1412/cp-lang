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
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	for (int i = 1; ; i++) {
		pair<string, int> x[n];
		bool good = false;
		for (int j = 0; j < n; j++) {
			string s = to_string(arr[j]);
			if (sz(s) >= i) {
				x[j] = {s.substr(sz(s) - i), j};
				good = true;
			} else {
				x[j] = {string(i - sz(s), '0') + s, j};
			};
		}
		if (!good) {
			break;
		}
		sort(x, x + n);
		for (int j = 0; j < n; j++) {
			cout << arr[x[j].second] << " \n"[j == n - 1];
		}
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