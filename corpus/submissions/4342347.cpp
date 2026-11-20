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
	pair<int, int> edges[m];
	vector<int> graph[n];
	for (auto& [u, v] : edges) {
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	int a, b, c, d;
	cin >> a >> b >> c >> d;
	a--;
	b--;
	c--;
	d--;
	int ans[m] {};
	for (int i = 0; i < m; i++) {
		auto [u, v] = edges[i];
		bool cx = u == c || v == c;
		bool dx = u == d || v == d;
		if (cx && dx) {
			cout << -1 << endl;
			return;
		} else if (cx) {
			ans[i] = 1;
		} else if (dx) {
			ans[i] = 2;
		}
	}
	int dist[4 * n];
	memset(dist, -1, sizeof(dist));
	queue<int> q;
	auto push = [&](int u, int desc, int dt) -> void {
		if (u == c) {
			desc |= 1;
		}
		if (u == d) {
			desc |= 2;
		}
		u = (u << 2) | desc;
		if (dist[u] == -1) {
			dbg(u, desc);
			dist[u] = dt;
			q.push(u);
		}
	};
	push(a, 0, 0);
	while (sz(q)) {
		int u = q.front();
		int cd = dist[u];
		q.pop();
		int desc = u & 3;
		u >>= 2;
		dbg(u, desc);
		for (auto& v : graph[u]) {
			push(v, desc, cd + 1);
		}
	}
	vector<pair<int, int>> poss;
	for (int i = 0; i < 3; i++) {
		int key = (b << 2) | i;
		if (dist[key] != -1) {
			poss.emplace_back(dist[key], i);
		}
	}
	if (!sz(poss)) {
		cout << -1 << endl;
		return;
	}
	sort(begin(poss), end(poss));
	int color = 1;
	if (poss[0].second & 2) {
		color = 2;
	}
	cout << 2 << endl;
	for (auto& x : ans) {
		if (!x) {
			x = color;
		}
		cout << x << endl;
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