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

const int maxn = 1 << 17;

vector<pair<int, long>> graph[maxn];

long dp(int u) {
	if (!sz(graph[u])) {
		return 0;
	}
	long ans = -1e18;
	for (auto& [v, w] : graph[u]) {
		ans = max(ans, w + dp(v));
	}
	return ans;
}

void solve() {
	int n, r;
	cin >> n >> r;
	n = 1 << n;
	for (int i = 0; i < n - 1; i++) {
		int u, a, b, w;
		cin >> u >> a >> b >> w;
		u--;
		a--;
		b--;
		graph[u].emplace_back(a, w);
		graph[u].emplace_back(b, 0);
	}
	cout << r + dp(0) << endl;
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