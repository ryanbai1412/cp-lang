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

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e5 + 5, logn = 20;

bool village[maxn];
int t, tin[maxn], tout[maxn], blift[logn][maxn];
long st[maxn], dist[logn][maxn], bliftv[logn][maxn];
vector<pair<int, long>> graph[maxn];

long dfs(int u, int par = -1) {
	tin[u] = t++;
	blift[0][u] = par;
	long ans = 1e18;
	if (village[u]) {
		ans = 0;
	}
	for (auto& [v, w] : graph[u]) {
		if (v != par) {
			dist[0][v] = w;
			ans = min(ans, w + dfs(v, u));
		}
	}
	tout[u] = t++;
	return bliftv[0][u] = st[u] = ans;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

void solve() {
	int n, s, q, e;
	cin >> n >> s >> q >> e;
	e--;
	int edges[n - 1][2];
	for (auto& [u, v] : edges) {
		int w;
		cin >> u >> v >> w;
		u--;
		v--;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	for (int i = 0; i < s; i++) {
		int u;
		cin >> u;
		u--;
		village[u] = true;
	}
	dfs(e);
	for (int i = 1; i < logn; i++) {
		for (int j = 0; j < n; j++) {
			int v = blift[i - 1][j];
			if (v != -1) {
				blift[i][j] = blift[i - 1][v];
				dist[i][j] = dist[i - 1][j] + dist[i - 1][v];
				bliftv[i][j] = min(bliftv[i - 1][j], dist[i - 1][j] + bliftv[i - 1][v]);
			} else {
				blift[i][j] = -1;
			}
		}
	}
	dbg(bliftv[0][6]);
	while (q--) {
		int road, x;
		cin >> road >> x;
		road--;
		x--;
		auto [u, v] = edges[road];
		if (anc(u, x) && anc(v, x)) {
			if (anc(v, u)) {
				swap(u, v);
			}
			if (st[v] >= long(1e18)) {
				cout << "oo" << endl;
				continue;
			}
			long ans = 1e18, cd = 0;
			for (int i = logn - 1; i >= 0; i--) {
				int y = blift[i][x];
				if (y != -1 && (anc(v, y) || v == y)) {
					ans = min(ans, cd + bliftv[i][x]);
					cd += dist[i][x];
					x = y;
				}
			}
			ans = min(ans, cd + st[v]);
			cout << ans << endl;
		} else {
			cout << "escaped" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}