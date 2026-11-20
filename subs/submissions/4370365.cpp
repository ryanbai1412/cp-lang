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

const int maxn = 1e4 + 5;

bool found, vis[maxn];
int source, sink, c, ans, edges[1 << 20];
vector<pair<int, int>> graph[maxn];

void dfs(int u, int mn) {
	static vector<int> st;
	if (found || vis[u]) {
		return;
	}
	vis[u] = true;
	if (u == sink) {
		found = true;
		ans += mn;
		for (auto& a : st) {
			edges[a] -= mn;
			edges[a ^ 1] += mn;
		}
		return;
	}
	for (auto& [v, e] : graph[u]) {
		if (!found && edges[e] >= c) {
			st.push_back(e);
			dfs(v, min(mn, edges[e]));
			st.pop_back();
		}
	}
}

void add_edge(int u, int v, int w) {
	static int eind = 0;
	int a = eind++, b = eind++;
	edges[a] = w;
	graph[u].emplace_back(v, a);
	graph[v].emplace_back(u, b);
}

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int vals[m][3];
	for (auto& [u, v, w] : vals) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	source = n + m;
	sink = n + m + 1;
	for (int i = 0; i < m; i++) {
		auto &[u, v, _] = vals[i];
		add_edge(i + n, u, 1e6);
		add_edge(i + n, v, 1e6);
	}
	for (int i = 0; i < m; i++) {
		add_edge(source, n + i, vals[i][2]);
	}
	for (int i = 0; i < n; i++) {
		add_edge(i, sink, arr[i]);
	}
	c = 1e6;
	while (c) {
		do {
			found = false;
			memset(vis, 0, sizeof(vis));
			dfs(source, 1e9);
		} while (found);
		c /= 2;
	}
	int cans = 0;
	for (auto& [_u, _v, w] : vals) {
		cans += w;
	}
	cout << cans - ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}