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

const int maxn = 1e6;

struct TSAT {
	int n {};
	vector<int> order, graph[2][maxn];
	bool vis[maxn] {};

	int new_node() {
		return n++;
	}
	
	void add_edge(int u, int v) {
		graph[0][u].push_back(v);
		graph[1][v].push_back(u);
	}

	void either(int a, int b) {
		if (a < 0) {
			a = 2 * (~a) + 1;
		} else {
			a *= 2;
		}
		if (b < 0) {
			b = 2 * (~b) + 1;
		} else {
			b *= 2;
		}
		add_edge(a ^ 1, b);
		add_edge(b ^ 1, a);
	}

	template<int ind>
	void dfs(int u) {
		if (vis[u]) {
			return;
		}
		vis[u] = true;
		for (auto& v : graph[ind][u]) {
			dfs<ind>(v);
		}
		order.push_back(u);
	}

	vector<bool> solve() {
		int n = this->n * 2;
		for (int i = 0; i < n; i++) {
			dfs<0>(i);
		}
		vector<int> corder(order.rbegin(), order.rend());
		int comp[n], cind = 0;
		memset(vis, 0, sizeof(vis));
		for (auto& a : corder) {
			if (!vis[a]) {
				order.clear();
				dfs<1>(a);
				for (auto& a : order) {
					comp[a] = cind;
				}
				cind++;
			}
		}
		vector<bool> ans(n / 2);
		for (int i = 0; i < n / 2; i++) {
			int ti = i * 2, fi = i * 2 + 1;
			if (comp[fi] == comp[ti]) {
				return {};
			} else if (comp[fi] < comp[ti]) {
				ans[i] = true;
			} else {
				ans[i] = false;
			}
		}
		return ans;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	TSAT solver;
	map<int, int> constraints[n];
	auto node_base = [&](int i, int x) -> int {
		auto [it, inserted] = constraints[i].try_emplace(x, 0);
		if (inserted) {
			it->second = solver.new_node();
			solver.new_node();
		}
		return it->second;
	};
	auto lt = [&](int nb) -> int { return nb; };
	auto gt = [&](int nb) -> int { return nb + 1; };
	for (int _ = 0; _ < m; _++) {
		int i, j, k, x;
		cin >> i >> j >> k >> x;
		i--;
		j--;
		k--;
		int inb = node_base(i, x);
		int jnb = node_base(j, x);
		int knb = node_base(k, x);
		solver.either(~lt(inb), ~lt(jnb));
		solver.either(~lt(inb), ~lt(knb));
		solver.either(~lt(jnb), ~lt(knb));
		solver.either(~gt(inb), ~gt(jnb));
		solver.either(~gt(inb), ~gt(knb));
		solver.either(~gt(jnb), ~gt(knb));
	}
	for (int i = 0; i < n; i++) {
		vector<int> nbs;
		for (auto& [_, nb] : constraints[i]) {
			nbs.push_back(nb);
		}
		for (auto& nb : nbs) {
			solver.either(~lt(nb), ~gt(nb));
		}
		for (int i = 0; i < sz(nbs) - 1; i++) {
			int inb = nbs[i], nnb = nbs[i + 1];
			solver.either(~gt(inb), ~lt(nnb));
			solver.either(gt(inb), lt(nnb));
		}
	}
	auto ans = solver.solve();
	if (sz(ans)) {
		cout << "YES" << endl;
		for (int i = 0; i < n; i++) {
			dbg(i);
			int l = 1, r = 1e9;
			for (auto& [x, nb] : constraints[i]) {
				if (ans[lt(nb)]) {
					dbg("<", x);
					r = min(r, x - 1);
				} else {
					dbg(">=", x);
					l = max(l, x);
				}
				if (ans[gt(nb)]) {
					dbg(">", x);
					l = max(l, x + 1);
				} else {
					dbg("<=", x);
					r = min(r, x);
				}
			}
			assert(l <= r);
			cout << l << " ";
		}
		cout << endl;
	} else {
		cout << "NO" << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}