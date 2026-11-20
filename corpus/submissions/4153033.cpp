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

const int maxn = 2e3 + 5;

int cnt = 0;
vector<int> graph[maxn];
bool vis[maxn];

void dfs(int u) {
	if (vis[u]) {
		return;
	}
	cnt++;
	vis[u] = true;
	for (auto& v : graph[u]) {
		dfs(v);
	}
}

void solve() {
	set<int> vis;
	int m;
	cin >> m;
	int edges[m][2];
	for (auto& [u, v] : edges) {
		char c;
		cin >> c >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
		vis.insert(u);
		vis.insert(v);
	}
	int n = sz(vis);
	if (m != n - 1) {
		cout << -1 << endl;
		return;
	}
	dfs(0);
	if (cnt != n) {
		cout << -1 << endl;
		return;
	}
	dbg(cnt);
	int deg[n] {};
	for (auto& [u, v] : edges) {
		deg[u]++;
		deg[v]++;
	}
	set<pair<int, int>> ans;
	for (auto& [u, v] : edges) {
		if (u == v) {
			cout << -1 << endl;
			return;
		}
		if (u > v) {
			swap(u, v);
		}
		ans.emplace(u, v);
	}
	if (sz(ans) != m) {
		cout << -1 << endl;
		return;
	}
	for (auto& [u, v] : edges) {
		if (!ans.count({u, v})) {
			cout << -1 << endl;
			return;
		}
		ans.erase({u, v});
		deg[u]--;
		deg[v]--;
		if (min(deg[u], deg[v]) != 0) {
			cout << -1 << endl;
			return;
		}
	}
	cout << n << endl;
	for (auto& [u, v] : edges) {
		cout << u + 1 << " " << v + 1 << endl;
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