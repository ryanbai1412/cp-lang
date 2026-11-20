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
const int maxn = 1e5 + 5;

int memo[maxn][2];
vector<int> graph[maxn];

int dp(int u, bool b, int p = -1) {
	int &ans = memo[u][b];
	if (ans != -1) {
		return ans;
	}
	long cans = 1;
	for (auto& v : graph[u]) {
		if (v != p) {
			long cur = dp(v, false, u);
			if (!b) {
				cur += dp(v, true, u);
			}
			cans = (cans * cur) % mod;
		}
	}
	ans = int(cans);
	dbg(u, b, ans);
	return ans;
}

void solve() {
	int n;
	cin >> n;
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	memset(memo, -1, sizeof(memo));
	cout << (dp(0, false) + dp(0, true)) % mod << endl;
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