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

const int maxn = 1e3 + 5;

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	vector<int> graph[n];
	for (int i = 0; i < m; i++) {
		for (int j = 0; j < n; j++) {
			int v;
			cin >> v;
			v--;
			graph[j].push_back(v);
		}
	}
	bool ans[maxn][maxn] {};
	auto dfs = [&](int s) -> void {
		bool* vis = ans[s];
		vector<int> q;
		auto push = [&](int u) -> void {
			if (!vis[u]) {
				vis[u] = true;
				q.push_back(u);
			}
		};
		push(s);
		while (sz(q)) {
			int u = q.back();
			q.pop_back();
			for (auto& v : graph[u]) {
				push(v);
			}
		}
	};
	for (int i = 0; i < n; i++) {
		dfs(i);
	}
	while (q--) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		if (ans[u][v]) {
			cout << "DA" << endl;
		} else {
			cout << "NE" << endl;
		}
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