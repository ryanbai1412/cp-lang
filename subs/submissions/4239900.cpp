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

int solve2(int n, int x) {
	int ans = n / 2;
	if (n & 1) {
		n -= 3;
		n /= 2;
		if (n % 4 == 0 || n % 4 == 3) {
			ans += !(x & 1);
		} else {
			ans += x & 1;
		}
	}
	return ans;
}

void solve() {
	int n, x;
	cin >> n >> x;
	if (n == 1) {
		cout << 0 << endl;
		return;
	}
	int ans = solve2(n, x);
	cout << ans << endl;
	for (int i = 0; i < ans; i++) {
		cout << i + 1 << endl;
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