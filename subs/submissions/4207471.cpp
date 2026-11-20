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

const int maxn = 50000, logn = 16;

struct Node {
	int opt1, opt2;

	void upd(int x) {
		if (x > opt2) {
			opt1 = opt2;
			opt2 = x;
		} else if (x != opt2 && x > opt1) {
			opt1 = x;
		}
	}

	Node& operator+=(const Node& n) {
		if (n.opt2 != -1) {
			upd(n.opt2);
			if (n.opt1 != -1) {
				upd(n.opt1);
			}
		}
		return *this;
	}

	friend Node operator+(Node a, const Node& b) {
		return a += b;
	}

	static Node def() {
		return {-1, -1};
	}
};

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
	int t = 0, tin[maxn], tout[maxn], lift[logn + 1][maxn];
	Node liftv[logn + 1][maxn];

	explicit DS(int n, const vector<tuple<int, int, int>>& edges) {
		for (auto& [u, v, w] : edges) {
			graph[u].emplace_back(v, w);
			graph[v].emplace_back(u, w);
		}
		dfs(0);
		for (int i = 1; i <= logn; i++) {
			for (int j = 0; j < n; j++) {
				if (lift[i - 1][j] == -1) {
					lift[i][j] = -1;
					liftv[i][j] = {-1, -1};
				} else {
					lift[i][j] = lift[i - 1][lift[i - 1][j]];
					liftv[i][j] = liftv[i - 1][j] + liftv[i - 1][lift[i - 1][j]];
				}
			}
		}
	}

	void dfs(int u, int p = -1) {
		tin[u] = t++;
		lift[0][u] = p;
		for (auto& [v, w] : graph[u]) {
			if (p != v) {
				liftv[0][v] = {-1, w};
				dfs(v, u);
			}
		}
		tout[u] = t - 1;
	}

	bool anc(int u, int v) const {
		return tin[u] <= tin[v] && tout[v] <= tout[u];
	}

	pair<int, int> query(int u, int v) const {
		Node ans {-1, -1};
		for (int i = logn; i >= 0; i--) {
			if (lift[i][u] != -1 && !anc(lift[i][u], v)) {
				ans += liftv[i][u];
				u = lift[i][u];
			}
		}
		if (!anc(u, v)) {
			ans += liftv[0][u];
			u = lift[0][u];
			assert(anc(u, v));
		}
		for (int i = logn; i >= 0; i--) {
			if (lift[i][v] != -1 && anc(u, lift[i][v])) {
				ans += liftv[i][v];
				v = lift[i][v];
			}
		}
		return {ans.opt1, ans.opt2};
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	vector<tuple<int, int, int>> edges(m), mstedges, oedges;
	for (auto& [u, v, w] : edges) {
		cin >> u >> v >> w;
		u--;
		v--;
	}
	sort(begin(edges), end(edges), [&](const auto& a, const auto& b) -> bool {
		return get<2>(a) < get<2>(b);
	});
	DSU dsu(n);
	int cans = 0, ans = 1e9, mcnt = 0;
	for (auto& [u, v, w] : edges) {
		if (dsu.merge(u, v)) {
			mstedges.emplace_back(u, v, w);
			cans += w;
			mcnt++;
		} else {
			oedges.emplace_back(u, v, w);
		}
	}
	if (mcnt != n - 1) {
		dbg(mcnt);
		cout << -1 << endl;
		return;
	}
	dbg(cans);
	DS ds(n, mstedges);
	for (auto& [u, v, w] : oedges) {
		auto [opt1, opt2] = ds.query(u, v);
		dbg(opt1, opt2, u, v, w);
		assert(opt2 <= w);
		ans = min(ans, cans + w - opt2);
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