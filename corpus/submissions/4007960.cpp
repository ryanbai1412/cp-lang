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

struct TSAT {
	int n {};
	vector<vector<int>> graph;

	int new_node() {
		graph.emplace_back();
		graph.emplace_back();
		return n++;
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
		graph[a ^ 1].push_back(b);
		graph[b ^ 1].push_back(a);
	}

	void dfs(int u, vector<bool>& vis, vector<int>& order) {
		if (vis[u]) {
			return;
		}
		vis[u] = true;
		for (auto& v : graph[u]) {
			dfs(v, vis, order);
		}
		order.push_back(u);
	}

	vector<bool> solve() {
		int n = sz(graph);
		vector<bool> vis(n);
		vector<int> order;
		for (int i = 0; i < n; i++) {
			dfs(i, vis, order);
		}
		reverse(begin(order), end(order));
		vector<vector<int>> ngraph(n);
		for (int i = 0; i < n; i++) {
			for (auto& v : graph[i]) {
				ngraph[v].push_back(i);
			}
		}
		graph = ngraph;
		int comp[n], cind = 0;
		vis.assign(n, false);
		for (auto& a : order) {
			if (!vis[a]) {
				vector<int> corder;
				dfs(a, vis, corder);
				for (auto& a : corder) {
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

namespace kactl {
#define rep(i, a, b) for (int i = a; i < (b); ++i)
#define all(x) begin(x), end(x)
using ll = long;
using pii = pair<int, int>;
using vi = vector<int>;

struct TwoSat {
	int N;
	vector<vi> gr;
	vi values; // 0 = false, 1 = true
				
	TwoSat(int n = 0) : N(n), gr(2*n) {}

	int new_node() { // (optional)
		gr.emplace_back();
		gr.emplace_back();
		return N++;
	}

	void either(int f, int j) {
		f = max(2*f, -1-2*f);
		j = max(2*j, -1-2*j);
		gr[f].push_back(j^1);
		gr[j].push_back(f^1);
	}

	vi val, comp, z; int time = 0;
	int dfs(int i) {
		int low = val[i] = ++time, x; z.push_back(i);
		for(int e : gr[i]) if (!comp[e])
				low = min(low, val[e] ?: dfs(e));
		if (low == val[i]) do {
				x = z.back(); z.pop_back();
				comp[x] = low;
				if (values[x>>1] == -1)
					values[x>>1] = x&1;
			} while (x != i);
		return val[i] = low;
	}

	vi solve() {
		values.assign(N, -1);
		val.assign(2*N, 0); comp = val;
		rep(i,0,2*N) if (!comp[i]) dfs(i);
		rep(i,0,N) if (comp[2*i] == comp[2*i+1]) return {};
		return values;
	}
};

#undef rep
#undef all
}  // namespace kactl

void solve() {
	int n, m;
	cin >> n >> m;
	kactl::TwoSat solver;
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