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

template<typename T, typename Operation>
struct SparseTable {
	vector<vector<T>> v;
	const Operation operation;
 
	SparseTable(const vector<T> &arr, const Operation &operation = Operation()): operation(operation) {
		int n = sz(arr), m = __lg(n) + 1;
		v.resize(m, vector<T>(n));
		v[0] = arr;
		for(int i = 1; i < m; i++) {
			for(int j = 0; j + (1 << i) <= n; j++) {
				v[i][j] = operation(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
			}
		}
	}
 
	T query(int l, int r) const {
		int x = __lg(r - l + 1);
		return operation(v[x][l], v[x][r - (1 << x) + 1]);
	}
};
 
struct Operation {
	pair<int, int> operator()(const pair<int, int> &a, const pair<int, int> &b) const {
		return min(a, b);
	}
};
 
const int maxn = 2e5;
 
int tin[maxn], depth[maxn];
vector<int> graph[maxn];
vector<pair<int, int>> order;
SparseTable<pair<int, int>, Operation>* rmq;
 
void dfs(int u, int d = 0, int p = -1) {
	tin[u] = sz(order);
	depth[u] = d;
	order.emplace_back(d, u);
	for(auto &v: graph[u]) {
		if(v != p) {
			dfs(v, d + 1, u);
			order.emplace_back(d, u);
		}
	}
}
 
int lca(int u, int v) {
	u = tin[u], v = tin[v];
	if(u > v) {
		swap(u, v);
	}
	return rmq->query(u, v).second;
}
 
int dist(int u, int v) {
	return depth[u] + depth[v] - 2 * depth[lca(u, v)];
}

struct DSU {
	vector<int> p;
	
	explicit DSU(int n): p(n, -1) {}
	
	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}
	
	bool merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return false;
		}
		p[u] = v;
		return true;
	}
};

void solve() {
	int n, q;
	cin >> n >> q;
	pair<int, int> edges[n - 1];
	for (auto& [u, v] : edges) {
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	tuple<int, int, int> queries[q];
	for (auto& [t, u, v] : queries) {
		cin >> t >> u >> v;
		u--;
		v--;
	}
	reverse(queries, queries + q);
	dfs(0);
	rmq = new SparseTable<pair<int, int>, Operation>(order);
	vector<int> ans;
	DSU dsu(n);
	for (auto& [t, u, v] : queries) {
		if (t == 1) {
			if (dsu.find(u) == dsu.find(v)) {
				ans.push_back(dist(u, v));
			} else {
				ans.push_back(-1);
			}
		} else {
			dsu.merge(u, v);
		}
	}
	reverse(begin(ans), end(ans));
	for (auto& a : ans) {
		cout << a << endl;
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