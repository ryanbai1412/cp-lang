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
	long arr[n][3];
	for (auto& [p, w, d] : arr) {
		cin >> p >> w >> d;
	}
	long cpos = -1e10, cans = 0, cadd = 0, ans = 1e18;
	for (auto& [p, w, d] : arr) {
		cans += (p - cpos - d) * w;
		cadd -= w;
	}
	vector<pair<int, long>> evts;
	for (auto& [p, w, d] : arr) {
		evts.emplace_back(p - d, w);
		evts.emplace_back(p + d, w);
	}
	sort(begin(evts), end(evts));
	for (auto& [npos, add] : evts) {
		cans += (npos - cpos) * cadd;
		cadd += add;
		ans = min(ans, cans);
		cpos = npos;
	}
	cout << ans << endl;
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