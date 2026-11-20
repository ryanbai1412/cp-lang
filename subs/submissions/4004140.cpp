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

const int maxn = 5000;

vector<int> graph[maxn];

struct Solver {
	int n;
	pair<const int&, const int&> ign;
	vector<int> ans;
	vector<bool> vis;

	Solver(int n, pair<const int&, const int&> ign) : n(n), ign(ign), vis(n) {}

	void dfs(int u) {
		if (vis[u]) {
			return;
		}
		vis[u] = true;
		ans.push_back(u);
		for (auto& v : graph[u]) {
			if (minmax(u, v) != ign) {
				dfs(v);
			}
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
	pair<int, int> edges[m];
	for (auto& [u, v] : edges) {
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		sort(begin(graph[i]), end(graph[i]));
	}
	if (m == n - 1) {
		Solver s(n, {-1, -1});
		auto ans = s.solve();
		assert(sz(ans) == n);
		for (auto& a : ans) {
			cout << a + 1 << " ";
		}
		cout << endl;
	} else {
		vector<int> mans {int(1e9)};
		for (auto& [u, v] : edges) {
			Solver s(n, minmax(u, v));
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