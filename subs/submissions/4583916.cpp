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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

vector<vector<int>> graph;

namespace cycle {

const int maxn = 1e5 + 5;

bool vis[maxn];
int depth[maxn], ans;

void dfs(int u, int d = 0) {
	vis[u] = true;
	depth[u] = d++;
	for (auto& v : graph[u]) {
		if (vis[v]) {
			ans = max(ans, d - depth[v]);
		} else {
			dfs(v, d);
		}
	}
}

int solve() {
	int n = sz(graph);
	fill(vis, vis + n, false);
	ans = 0;
	dfs(0);
	return ans;
}

}  // namespace cycle

bool solve2() {
	return cycle::solve() >= 3;
}

bool solve3() {
	return cycle::solve() >= 4;
}

bool solve4() {
	int mx = 0;
	for (auto& a : graph) {
		mx = max(mx, sz(a));
	}
	return mx >= 3;
}

bool solve1() {
	return false;
}

bool solve5() {
	return false;
}

void solve() {
	int s;
	cin >> s;
	int tcs;
	cin >> tcs;
	while (tcs--) {
		int n, m;
		cin >> n >> m;
		graph.clear();
		graph.resize(n);
		for (int i = 0; i < m; i++) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			graph[u].push_back(v);
			graph[v].push_back(u);
		}
		bool ans;
		if (s == 1) {
			ans = solve1();
		} else if (s == 2) {
			ans = solve2();
		} else if (s == 3) {
			ans = solve3();
		} else if (s == 4) {
			ans = solve4();
		} else if (s == 5) {
			ans = solve5();
		} else {
			assert(false);
		}
		if (ans) {
			cout << "YES" << endl;
		} else {
			cout << "NO" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}