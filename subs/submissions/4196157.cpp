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

void solve() {
	int n, m;
	cin >> n >> m;
	vector<int> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	bool vis1[n] {}, vis2[n] {};
	auto bfs = [&](int s, bool* vis) -> pair<int, int> {
		pair<int, int> ans {};
		vector<pair<int, int>> q;
		auto push = [&](int u, int d) -> void {
			if (!vis[u]) {
				vis[u] = true;
				ans = max(ans, pair<int, int> {d, u});
				q.emplace_back(u, d);
			}
		};
		push(s, 0);
		while (sz(q)) {
			auto [u, d] = q.back();
			q.pop_back();
			for (auto& v : graph[u]) {
				push(v, d + 1);
			}
		}
		return ans;
	};
	int ans = 0;
	for (int i = 0; i < n; i++) {
		if (!vis1[i]) {
			dbg(i);
			auto [_, u] = bfs(i, vis1);
			auto [d, _v] = bfs(u, vis2);
			dbg(u, d);
			ans += d + 1;
		}
	}
	cout << ans << endl;
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