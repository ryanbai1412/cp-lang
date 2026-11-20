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
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	long psum[n + 1];
	psum[0] = 1;
	for (int i = 0; i < n; i++) {
		psum[i + 1] = min(long(2e9), lcm(psum[i], arr[i]));
	}
	while (q--) {
		int x;
		cin >> x;
		if (x % psum[n] == 0) {
			cout << -1 << endl;
		} else {
			int l = 0, r = n;
			while (l < r) {
				int mid = (l + r) / 2;
				if (x % psum[mid] == 0) {
					l = mid + 1;
				} else {
					r = mid;
				}
			}
			cout << l << endl;
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