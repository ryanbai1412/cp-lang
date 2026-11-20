#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).siz())

#define inline inline __attribute__((always_inline))

struct Node {
	int l, r, v;

	Node operator+(const Node &n) const {
		return {l, n.r, v + n.v + (r == n.l)};
	}

	static Node from(int lazy, int cnt = 0) {
		return {lazy, lazy, cnt};
	}
};

struct LazySegTree {
	int n;
	vector<Node> v;
	vector<int> lazy;

	LazySegTree(int n): n(n), v(4 * n, {0, 0, 0}), lazy(4 * n, -1) {
		build(1, 0, n - 1);
	}

	void build(int o, int l, int r) {
		if(l == r) {
			v[o] = Node::from(-(l + 1));
		}else {
			v[o] = v[o * 2] + v[o * 2 + 1];
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
		}
	}

	void maintain(int o, int l, int r) {
		if(lazy[o] != -1) {
			v[o] = Node::from(lazy[o], r - l);
		}else {
			if(l < r) {
				v[o] = v[o * 2] + v[o * 2 + 1];
			}else {
				v[o] = Node::from(-(l + 1));
			}
		}
	}

	void update(int o, int l, int r, int ql, int qr, int x) {
		if(ql <= l && r <= qr) {
			lazy[o] = x;
		}else {
			if(lazy[o] != -1) {
				lazy[o * 2] = lazy[o * 2 + 1] = lazy[o];
				lazy[o] = -1;
			}
			int mid = (l + r) / 2;
			if(ql <= mid) {
				update(o * 2, l, mid, ql, qr, x);
			}else {
				maintain(o * 2, l, mid);
			}
			if(mid < qr) {
				update(o * 2 + 1, mid + 1, r, ql, qr, x);
			}else {
				maintain(o * 2 + 1, mid + 1, r);
			}
		}
		maintain(o, l, r);
	}

	void update(int l, int r, int x) {
		update(1, 0, n - 1, l, r, x);
	}

	Node query(int o, int l, int r, int ql, int qr, int clazy) {
		if(clazy == -1) {
			clazy = lazy[o];
		}
		if(ql <= l && r <= qr) {
			if(clazy != -1) {
				return Node::from(clazy, r - l);
			}
			return v[o];
		}else {
			int mid = (l + r) / 2;
			if(ql <= mid) {
				if(mid < qr) {
					return query(o * 2, l, mid, ql, qr, clazy)
						   + query(o * 2 + 1, mid + 1, r, ql, qr, clazy);
				}
				return query(o * 2, l, mid, ql, qr, clazy);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr, clazy);
		}
	}

	Node query(int l, int r) {
		return query(1, 0, n - 1, l, r, -1);
	}
};

const int maxn = 1e5, logn = 17;

int t, siz[maxn], tin[maxn], tout[maxn], lift[logn][maxn], chain[maxn];
vector<int> order;
vector<int> graph[maxn];

void pdfs(int u) {
	siz[u] = 1;

	for(auto &v: graph[u]) {
		lift[0][v] = u;
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));

		pdfs(v);

		siz[u] += siz[v];
		if(siz[v] > siz[graph[u][0]]) {
			swap(v, graph[u][0]);
		}
	}
}

void dfs(int u) {
	tin[u] = t++;
	order.push_back(u);

	for(auto &v: graph[u]) {
		if(v == graph[u][0]) {
			chain[v] = chain[u];
		}else {
			chain[v] = v;
		}

		dfs(v);
	}

	tout[u] = t - 1;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

int lca(int u, int v) {
	if(anc(u, v)) {
		return u;
	}
	for(int i = logn - 1; i >= 0; i--) {
		if(lift[i][u] != -1 && !anc(lift[i][u], v)) {
			u = lift[i][u];
		}
	}
	return lift[0][u];
}

void solve() {
	t = 0;

	int n, q;
	cin >> n >> q;

	for(int i = 0; i < n; i++) {
		graph[i].clear();
	}

	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}

	lift[0][0] = -1;
	chain[0] = 0;

	pdfs(0);
	dfs(0);

	for(int i = 1; i < logn; i++) {
		for(int j = 0; j < maxn; j++) {
			if(lift[i - 1][j] == -1) {
				lift[i][j] = -1;
			}else {
				lift[i][j] = lift[i - 1][lift[i - 1][j]];
			}
		}
	}

	LazySegTree st(n);
	int qind = 1;
	while(q--) {
		int t, u, v;
		cin >> t >> u >> v;
		u--;
		v--;
		u = tin[u];
		v = tin[v];
		if(u > v) {
			swap(u, v);
		}
		if(t == 1) {
			st.update(u, v, qind++);
		}else {
			cout << st.query(u, v).v << endl;
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}