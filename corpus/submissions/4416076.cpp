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
#define dbg(...)
#endif

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	int n, angle;
	cin >> n >> angle;
	pair<ld, int> ans {1e9, 0};
	for (int i = 0; i < n; i++) {
		ld x, y;
		cin >> x >> y;
		ld ang = atan2(y, x) * 180 / M_PIl;
		ld c = angle - ang;
		if (c < 0) {
			c += 360;
		}
		if (c >= 360) {
			c -= 360;
		}
		c = min(c, 360 - c);
		dbg(i, c);
		ans = min(ans, pair<ld, int> {c, i + 1});
	}
	cout << ans.second << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}