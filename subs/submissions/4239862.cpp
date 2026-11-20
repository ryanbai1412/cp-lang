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

const int maxn = 2e5 + 5;

int n, m, ans[maxn], arr[maxn];
vector<pair<int, int>> graph[maxn];

int cnt(int l, int r) {
	return int(lower_bound(arr, arr + m, r) - lower_bound(arr, arr + m, l));
}

int mpoint(int l, int r) {
	return (l + r) / 2 + 1;
}

void dfs(int u, int l, int r) {
	if (l >= r) {
		return;
	}
	dbg(u + 1, l, r);
	ans[u] = cnt(l, r);
	sort(begin(graph[u]), end(graph[u]), [&](const auto &a, const auto &b) -> bool {
		return a.second < b.second;
	});
	for (int i = 0; i < sz(graph[u]); i++) {
		auto &[v, w] = graph[u][i];
		graph[v].erase(find(begin(graph[v]), end(graph[v]), pair<int, int> {u, w}));
		int cl = i ? mpoint(graph[u][i - 1].second, w) : l;
		int cr = i + 1 < sz(graph[u]) ? mpoint(w, graph[u][i + 1].second) : r;
		cl = max(cl, l);
		cr = min(cr, r);
		dbg(u + 1, v + 1, cl, cr, l, r);
		dfs(v, cl, cr);
	}
}

void solve() {
	cin >> n >> m;
	for (int i = 0; i < n - 1; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	for (int i = 0; i < m; i++) {
		cin >> arr[i];
	}
	sort(arr, arr + m);
	dfs(0, 0, int(1e9 + 5));
	for (int i = 0; i < n; i++) {
		cout << ans[i] << " \n"[i == n - 1];
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