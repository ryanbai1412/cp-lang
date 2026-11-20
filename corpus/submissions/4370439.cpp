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

const int maxn = 6e4;

int n, m, source, sink, dist[maxn], ptr[maxn], edges[1 << 20];
vector<pair<int, int>> graph[maxn];

bool bfs() {
	queue<int> q;
	memset(dist, -1, sizeof(dist));
	auto push = [&](int u, int d) -> void {
		if (dist[u] == -1) {
			dist[u] = d;
			q.push(u);
		}
	};
	push(source, 0);
	while (sz(q)) {
		int u = q.front();
		q.pop();
		for (auto& [v, e] : graph[u]) {
			if (edges[e]) {
				push(v, dist[u] + 1);
			}
		}
	}
	return dist[sink] != -1;
}

int dfs(int u, int mn) {
	static vector<int> st;
	if (u == sink) {
		return mn;
	}
	for (int &i = ptr[u]; i < sz(graph[u]); i++) {
		auto [v, e] = graph[u][i];
		int w = edges[e];
		if (w && dist[v] == dist[u] + 1) {
			int ans = dfs(v, min(mn, w));
			if (ans) {
				edges[e] -= ans;
				edges[e ^ 1] += ans;
				return ans;
			}
		}
	}
	return 0;
}

void add_edge(int u, int v, int w) {
	static int eind = 0;
	int a = eind++, b = eind++;
	edges[a] = w;
	graph[u].emplace_back(v, a);
	graph[v].emplace_back(u, b);
}

void solve() {
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
	for (int i = 0; i < n; i++) {
		add_edge(i, sink, arr[i]);
	}
	for (int i = 0; i < m; i++) {
		auto& [u, v, _] = vals[i];
		add_edge(i + n, u, 1e6);
		add_edge(i + n, v, 1e6);
	}
	for (int i = 0; i < m; i++) {
		add_edge(source, n + i, vals[i][2]);
	}
	int ans = 0;
	while (bfs()) {
		memset(ptr, 0, sizeof(ptr));
		while (int x = dfs(source, 1e9)) {
			ans += x;
		}
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