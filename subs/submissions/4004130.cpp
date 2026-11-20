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

struct Solver {
	int n;
	vector<int> ans;
	vector<bool> vis;
	vector<vector<int>> graph;

	Solver(int n, const vector<pair<int, int>>& edges)
		: n(n), vis(n), graph(n) {
		for (auto& [u, v] : edges) {
			graph[u].push_back(v);
			graph[v].push_back(u);
		}
	}

	void dfs(int u) {
		if (vis[u]) {
			return;
		}
		vis[u] = true;
		ans.push_back(u);
		sort(begin(graph[u]), end(graph[u]));
		for (auto& v : graph[u]) {
			dfs(v);
		}
	}

	vector<int> solve() {
		dfs(0);
		return ans;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	vector<pair<int, int>> edges(m);
	for (auto& [u, v] : edges) {
		cin >> u >> v;
		u--;
		v--;
	}
	if (sz(edges) == n - 1) {
		Solver s(n, edges);
		auto ans = s.solve();
		assert(sz(ans) == n);
		for (auto& a : ans) {
			cout << a + 1 << " ";
		}
		cout << endl;
	} else {
		vector<int> mans {int(1e9)};
		for (int i = 0; i < m; i++) {
			auto cur(edges);
			cur.erase(cur.begin() + i);
			Solver s(n, cur);
			auto ans = s.solve();
			if (sz(ans) == n) {
				mans = min(mans, ans);
			}
		}
		assert(sz(mans) == n);
		for (auto& a : mans) {
			cout << a + 1 << " ";
		}
		cout << endl;
	}
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