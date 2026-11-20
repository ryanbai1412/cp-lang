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
	int n, q;
	cin >> n >> q;
	int queries[q][3];
	for (auto& [l, r, x] : queries) {
		cin >> l >> r >> x;
		l--;
		r--;
	}
	long arr[n];
	fill(arr, arr + n, 1);
	for (auto& [l, r, x] : queries) {
		for (int i = l; i <= r; i++) {
			arr[i] = lcm(arr[i], x);
		}
	}
	for (auto& [l, r, x] : queries) {
		long cur = arr[l];
		for (int i = l; i <= r; i++) {
			cur = gcd(cur, arr[i]);
		}
		if (cur != x) {
			cout << "Impossible" << endl;
			return;
		}
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