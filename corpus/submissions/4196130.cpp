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
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	for (int l = 0; l < n; l++) {
		for (int r = l + 1; r <= n; r++) {
			int len = r - l;
			if (l + len * k <= n) {
				dbg(l, len, k);
				for (int a = 1; a < k; a++) {
					if (!equal(arr + l, arr + r, arr + l + a * len)) {
						goto loop;
					}
				}
				cout << len << endl;
				for (int i = l; i < r; i++) {
					cout << arr[i] << " \n"[i == r - 1];
				}
				return;
				loop:;
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