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

const int maxn = 5e3;

struct DSU {
	vector<int> p;

	explicit DSU(int n) : p(n, -1) {}

	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}

	bool merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return false;
		}
		if (p[u] < p[v]) {
			swap(u, v);
		}
		p[v] += p[u];
		p[u] = v;
		return true;
	}
};

struct DS {
	vector<pair<int, int>> graph[maxn];
	pair<int, int> ans[maxn][maxn];

	explicit DS(const vector<tuple<int, int, int>>& edges) {
		for (auto& [u, v, w] : edges) {
			graph[u].emplace_back(v, w);
			graph[v].emplace_back(u, w);
		}
		for (int i = 0; i < maxn; i++) {
			bool vis[maxn] {};
			vector<tuple<int, int, int>> q;
			auto push = [&](int u, int opt1, int opt2) -> void {
				if (!vis[u]) {
					vis[u] = true;
					q.emplace_back(u, opt1, opt2);
				}
			};
			push(i, -1, -1);
			while (sz(q)) {
				auto [u, opt1, opt2] = q.back();
				ans[i][u] = {opt1, opt2};
				q.pop_back();
				for (auto& [v, w] : graph[u]) {
					int copt1 = opt1;
					int copt2 = opt2;
					if (w != copt1 && w != copt2) {
						if (w > copt2) {
							copt1 = copt2;
							copt2 = w;
						} else if (w > copt1) {
							copt1 = w;
						}
					}
					push(v, copt1, copt2);
				}
			}
		}
	}

	pair<int, int> query(int u, int v) const {
		return ans[u][v];
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	vector<tuple<int, int, int>> edges(m), mstedges;
	for (auto& [u, v, w] : edges) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	sort(begin(edges), end(edges), [&](const auto &a, const auto &b) -> bool {
	  return get<2>(a) < get<2>(b);
	});
	DSU dsu(n);
	int cans = 0, ans = 1e9, mcnt = 0;
	for (auto& [u, v, w] : edges) {
		if (dsu.merge(u, v)) {
			mstedges.emplace_back(u, v, w);
			cans += w;
			mcnt++;
		}
	}
	if (mcnt != n - 1) {
		dbg(mcnt);
		cout << -1 << endl;
		return;
	}
	dbg(cans);
	DS ds(mstedges);
	for (auto& [u, v, w] : edges) {
		auto [opt1, opt2] = ds.query(u, v);
		dbg(opt1, opt2, u, v, w);
		assert(opt2 <= w);
		if (opt2 != w && opt2 != -1) {
			ans = min(ans, cans + w - opt2);
		} else if (opt1 != w && opt1 != -1) {
			ans = min(ans, cans + w - opt1);
		}
	}
	if (ans < 1e9) {
		cout << ans << endl;
	} else {
		cout << -1 << endl;
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