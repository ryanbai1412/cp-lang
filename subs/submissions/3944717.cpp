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

const int dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};
const int maxn = 105, maxt = 105, maxp = 1005;

using __gnu_pbds::gp_hash_table;

int n;
gp_hash_table<int, int> memo;
gp_hash_table<int, int> events[maxt];

bool ibs(int x, int y) {
	return 0 <= x && x < n && 0 <= y && y < n;
}

int dp(int t, int x, int y, int p) {
	if (t == maxt) {
		return p;
	}
	int lkey = x * maxn + y;
	int key = (lkey * maxt + t) * maxp + p;
	{
		auto it = memo.find(key);
		if (it != memo.end()) {
			return it->second;
		}
	}
	{
		auto it = events[t].find(lkey);
		if (it != events[t].end()) {
			int change = it->second;
			if (change == -1) {
				p /= 2;
			} else {
				p += change;
			}
		}
	}
	int ans = dp(t + 1, x, y, p);
	for (int i = 0; i < 4; i++) {
		int cx = x + dx[i];
		int cy = y + dy[i];
		if (ibs(cx, cy)) {
			ans = max(ans, dp(t + 1, cx, cy, p));
		}
	}
	memo[key] = ans;
	return ans;
}

void solve() {
	int p, b, x, y;
	cin >> n >> p >> b >> x >> y;
	for (int i = 0; i < p; i++) {
		int x, y, t;
		cin >> x >> y >> t;
		events[t][x * maxn + y]++;
	}
	for (int i = 0; i < b; i++) {
		int x, y, t;
		cin >> x >> y >> t;
		events[t][x * maxn + y] = -1;
	}
	cout << dp(0, x, y, 0) << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}