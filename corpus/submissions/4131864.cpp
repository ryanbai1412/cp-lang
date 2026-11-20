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

const long mod = 1e9 + 7;
const int maxn = 1e4 + 5, maxd = 100 + 5;

string s;
int d;
long memo[maxn][2][maxd];

long dp(int ind, bool lim, int cmod) {
	if (ind == sz(s)) {
		return !cmod;
	}
	long &ans = memo[ind][lim][cmod];
	if (ans != -1) {
		return ans;
	}
	int cur = s[ind] - '0';
	if (!lim) {
		cur = 9;
	}
	ans = 0;
	for (int i = 0; i <= cur; i++) {
		ans += dp(ind + 1, lim && (i == cur), (cmod + i) % d);
		dbg(i, ans);
	}
	ans %= mod;
	dbg(ind, lim, cmod, ans);
	return ans;
}

void solve() {
	cin >> s >> d;
	memset(memo, -1, sizeof(memo));
	cout << (dp(0, true, 0) + mod - 1) % mod << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
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