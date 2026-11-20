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
	int arr[n][2];
	for (auto& [l, r] : arr) {
		cin >> l >> r;
	}
	int q;
	cin >> q;
	while (q--) {
		int l, r;
		cin >> l >> r;
		l--;
		r--;
		vector<pair<int, int>> evts;
		for (int i = l; i <= r; i++) {
			evts.emplace_back(arr[i][0], 1);
			evts.emplace_back(arr[i][1] + 1, -1);
		}
		sort(begin(evts), end(evts));
		int cur = 0, ans = 0;
		for (auto& [_, a] : evts) {
			ans = max(ans, cur += a);
		}
		cout << ans << endl;
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