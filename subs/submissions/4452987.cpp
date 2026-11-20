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

const int maxn = 1e6 + 5;

int vis[maxn], depth[maxn], ones;
bool arr[maxn];
vector<int> graph[maxn], igraph[maxn];

void dfs(int u) {
	if (vis[u]) {
		if (vis[u] == 1) {
			if (ones >= depth[u]) {
				cout << -1 << endl;
				exit(0);
			}
		}
		return;
	}
	vis[u] = 1;
	if (arr[u]) {
		ones++;
	}
	depth[u] = ones;
	for (auto& v : graph[u]) {
		dfs(v);
	}
	if (arr[u]) {
		ones--;
	}
	vis[u] = 2;
}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		igraph[v].push_back(u);
	}
	for (int i = 0; i < n; i++) {
		dfs(i);
	}
	int ans = 0;
	for (int i = 0; i < n; i++) {
		int cur = arr[i];
		for (auto& a : igraph[i]) {
			cur ^= arr[a];
		}
		ans += cur;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}