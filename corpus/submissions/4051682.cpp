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
	int a, b, c;
	cin >> a >> b >> c;
	int sum = a + b * 2 + c * 3;
	if (sum % 2) {
		cout << "NO" << endl;
		return;
	}
	if (c) {
		if (a >= c) {
			cout << "YES" << endl;
        } else {
            cout << "NO" << endl;
		}
	} else {
		if (a % 2) {
			cout << "NO" << endl;
		} else if (b % 2 && a < 2) {
			cout << "NO" << endl;
		} else {
			cout << "YES" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}