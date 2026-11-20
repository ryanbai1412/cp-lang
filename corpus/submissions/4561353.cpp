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
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

const int maxn = 5e5 + 5;

int diam(int n, const vector<pair<int, int>>& edges) {
	assert(sz(edges) == n - 1);
	vector<int> graph[n];
	for (auto& [u, v] : edges) {
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	auto bfs = [&](int src) -> pair<int, int> {
		int dist[n], vcnt = 0;
		memset(dist, -1, sizeof(dist));
		vector<int> q;
		auto push = [&](int u, int d) -> void {
			if (dist[u] == -1) {
				vcnt++;
				dist[u] = d;
				q.push_back(u);
			}
		};
		push(src, 0);
		while (sz(q)) {
			int u = q.back();
			q.pop_back();
			for (auto& v : graph[u]) {
				push(v, dist[u] + 1);
			}
		}
		if (vcnt < n) {
			return {-1e9, 0};
		}
		int ans = int(max_element(dist, dist + n) - dist);
		return {dist[ans], ans};
	};
	return bfs(bfs(0).second).first;
}

namespace s3 {

bool vis[maxn];
vector<int> graph[maxn], st, cyc;

void dfs(int u, int p = -1) {
	dbg(u, p, vis[u]);
	if (vis[u]) {
		cyc = vector<int>(find(begin(st), end(st), u), st.end());
		return;
	}
	vis[u] = true;
	st.push_back(u);
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u);
		}
	}
	st.pop_back();
}

int solve(int n, const vector<pair<int, int>>& edges) {
	for (int i = 0; i < n; i++) {
		graph[i].clear();
	}
	for (auto& [u, v] : edges) {
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	fill(vis, vis + n, false);
	dfs(0);
	for (auto& a : cyc) {
		dbg(a);
	}
	return 1e9;
}

}  // namespace s3

vector<pair<int, int>> graph[maxn];

namespace s1 {

void solve(int n) {
	vector<pair<int, int>> edges;
	for (int i = 0; i < n; i++) {
		for (auto& [v, t] : graph[i]) {
			if (t == 1 && i < v) {
				edges.emplace_back(i, v);
			}
		}
	}
	cout << 2 * (n - 1) - diam(n, edges) << endl;
}

}  // namespace s1

namespace s2 {

void solve(int n) {
	int dist[n << (n + 1)];
	memset(dist, 0x3f, sizeof(dist));
	min_pq<pair<int, int>> pq;
	auto push = [&](int u, int d) -> void {
		if (d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, u);
		}
	};
	for (int i = 0; i < n; i++) {
		push((i << (n + 1)) | (1 << i), 0);
	}
	int msk = (1 << (n + 1)) - 1, msk2 = (1 << n) - 1;
	while (sz(pq)) {
		auto [d, u] = pq.top();
		pq.pop();
		if ((u & msk2) == msk2) {
			cout << d << endl;
			return;
		}
		if (dist[u] == d) {
			for (auto& [v, w] : graph[u >> (n + 1)]) {
				int nu = 0;
				if (w != 1) {
					if ((u >> n) & 1) {
						continue;
					}
					nu = 1 << n;
				}
				push((v << (n + 1)) | (u & msk) | nu | (1 << v), d + w);
			}
		}
	}
}

}  // namespace s2

void solve() {
	int n, m;
	cin >> n >> m;
	int mn = 1e9;
	vector<array<int, 3>> edges;
	for (int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
		edges.push_back({u, v, w});
		if (w != 1) {
			mn = min(mn, w);
		}
	}
	if (mn >= (n + 1) / 2) {
		s1::solve(n);
		return;
	}
	vector<pair<int, int>> base;
	for (auto& [u, v, w] : edges) {
		if (w == 1) {
			base.emplace_back(u, v);
		}
	}
	int ans = 2 * (n - 1) - diam(n, base);
	vector<int> cgraph[n];
	for (auto& [u, v] : base) {
		cgraph[u].push_back(v);
		cgraph[v].push_back(u);
	}
	for (auto& [cu, cv, cw] : edges) {
		if (cw != 1) {
			vector<int> cyc;
			{
				int from[n];
				memset(from, -1, sizeof(from));
				vector<int> q;
				auto push = [&](int u, int f) -> void {
					if (from[u] == -1) {
						from[u] = f;
						q.push_back(u);
					}
				};
				push(cu, -2);
				while (sz(q)) {
					int u = q.back();
					q.pop_back();
					for (auto& v : cgraph[u]) {
						push(v, u);
					}
				}
				int u = cv;
				while (u != -2) {
					cyc.push_back(u);
					u = from[u];
				}
			}
			int dist[n];
			memset(dist, -1, sizeof(dist));
			for (auto& a : cyc) {
				dist[a] = 0;
			}
			int tmx[sz(cyc)];
			for (int i = 0; i < sz(cyc); i++) {
				vector<int> mx {0, 0};
				for (auto& b : cgraph[cyc[i]]) {
					int cur = 0;
					vector<int> q;
					auto push = [&](int u, int d) -> void {
						if (dist[u] == -1) {
							dist[u] = d;
							cur = max(cur, d);
							q.push_back(u);
						}
					};
					push(b, 1);
					while (sz(q)) {
						int u = q.back();
						q.pop_back();
						for (auto& v : cgraph[u]) {
							push(v, dist[u] + 1);
						}
					}
					mx.push_back(cur);
				}
				sort(begin(mx), end(mx), greater<>());
				tmx[i] = mx[0];
				dbg(mx[0], mx[1]);
				ans = min(ans, 2 * (n - 1) - sz(cyc) + 1 + cw - mx[0] - mx[1]);
			}
			int oa = 1e9, ob = 1e9;
			for (int i = sz(cyc) - 1; i >= 0; i--) {
				ans = min(ans, 2 * (n - 1) - sz(cyc) - 1 - i + cw - tmx[i] + oa);
				ans = min(ans, 2 * (n - 1) - sz(cyc) - 1 + sz(cyc) + i + cw - tmx[i] + ob);
				oa = min(oa, i - tmx[i]);
				ob = min(ob, -i - tmx[i]);
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}