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
	string s;
	cin >> s;
	vector<int> vals;
	for (int i = 0; i < n;) {
		for (; i < n && s[i] == '1'; i++)
			;
		if (i == n) {
			break;
		}
		int l = i;
		for (; i < n && s[i] == '0'; i++)
			;
		vals.push_back(i - l);
	}
	sort(begin(vals), end(vals), greater<>());
	int ans = 0;
	for (int i = 0; i < min(sz(vals), k); i++) {
		ans += vals[i];
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