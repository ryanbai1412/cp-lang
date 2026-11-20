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

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

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

template<typename T, typename Operation = plus<T>>
struct SegTree {
	static constexpr T noT {};

	const int n;
	const Operation operation;

	vector<T> v, arr;

	SegTree(const vector<T> &arr, const Operation &operation = Operation()): n(sz(arr)), operation(operation), v(4 * n),
																			 arr(arr) {
		build(1, 0, n - 1);
	}

	void build(int o, int l, int r) {
		if(l == r) {
			v[o] = arr[l];
		}else {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	void set(int o, int l, int r, int ind, const T &x) {
		if(l == r) {
			v[o] = arr[l] = x;
		}else {
			int mid = (l + r) / 2;
			if(ind <= mid) {
				set(o * 2, l, mid, ind, x);
			}else {
				set(o * 2 + 1, mid + 1, r, ind, x);
			}
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	inline void set(int ind, const T &x) {
		set(1, 0, n - 1, ind, x);
	}

	inline T get(int ind) {
		return arr[ind];
	}

	T query(int o, int l, int r, int ql, int qr) const {
		if(ql <= l && r <= qr) {
			return v[o];
		}else {
			int mid = (l + r) / 2;
			if(ql <= mid) {
				if(mid < qr) {
					return operation(query(o * 2, l, mid, ql, qr), query(o * 2 + 1, mid + 1, r, ql, qr));
				}
				return query(o * 2, l, mid, ql, qr);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr);
		}
	}

	inline T query(int l, int r) const {
		return query(1, 0, n - 1, l, r);
	}

	template<typename Predicate, bool first>
	int bsearch(int o, int l, int r, const T &pref, const Predicate &predicate) const {
		if(l == r) {
			return l;
		}else {
			int mid = (l + r) / 2;
			T npref;
			if(first) {
				npref = operation(pref, v[o * 2]);
			}else {
				npref = v[o * 2];
			}
			if(predicate(npref)) {
				return bsearch<Predicate, first>(o * 2, l, mid, pref, predicate);
			}
			return bsearch<Predicate, true>(o * 2 + 1, mid + 1, r, npref, predicate);
		}
	}

	template<typename Predicate>
	inline int bsearch(const Predicate &predicate) const {
		if(!predicate(v[1])) {
			return n;
		}
		return bsearch<Predicate, false>(1, 0, n - 1, noT, predicate);
	}

	template<typename Predicate>
	pair<int, T> bsearchl(int o, int l, int r, int ql, const T &pref, const Predicate &predicate) const {
		if(ql <= l) {
			T npref;
			if(ql < l) {
				npref = operation(pref, v[o]);
			}else {
				npref = v[o];
			}
			if(!predicate(npref)) {
				return {r + 1, npref};
			}
			if(ql < l) {
				return {bsearch<Predicate, true>(o, l, r, pref, predicate), npref};
			}else {
				return {bsearch<Predicate, false>(o, l, r, pref, predicate), npref};
			}
		}else {
			int mid = (l + r) / 2;
			if(ql <= mid) {
				auto[ans, npref] = bsearchl(o * 2, l, mid, ql, pref, predicate);
				if(ans <= mid) {
					return {ans, operation(npref, v[o * 2 + 1])};
				}
				return bsearchl(o * 2 + 1, mid + 1, r, ql, npref, predicate);
			}
			return bsearchl(o * 2 + 1, mid + 1, r, ql, pref, predicate);
		}
	}

	template<typename Predicate>
	inline int bsearchl(int l, const Predicate &predicate) const {
		return bsearchl(1, 0, n - 1, l, noT, predicate).first;
	}
};

struct Operation {
	pair<int, int> operator()(const pair<int, int> &a, const pair<int, int> &b) const {
		return min(a, b);
	}
};

const int maxn = 2e5;

int n, t, tin[maxn], tout[maxn], rmqin[maxn], par[maxn], arr[maxn];
vector<int> graph[maxn];
vector<pair<int, int>> order;

SparseTable<pair<int, int>, Operation> *rmq;

void dfs(int u, int d = 0, int p = -1) {
	tin[u] = t++;
	rmqin[u] = sz(order);
	par[u] = p;
	order.emplace_back(d, u);
	for(auto &v: graph[u]) {
		if(v != p) {
			dfs(v, d + 1, u);
			order.emplace_back(d, u);
		}
	}
	tout[u] = t++;
}

int lca(int u, int v) {
	u = rmqin[u], v = rmqin[v];
	if(u > v) {
		swap(u, v);
	}
	return rmq->query(u, v).second;
}

struct DS {
	vector<int> arr;
	SegTree<int> st;

	DS(): arr(n), st(vector<int>(2 * n)) {}

	void update(int u, int x) {
		arr[u] += x;
		st.set(tin[u], arr[u]);
		st.set(tout[u], -arr[u]);
	}

	int query(int u, int v) {
		int l = lca(u, v);
		return -arr[l] + st.query(tin[l], tin[u]) + st.query(tin[l], tin[v]);
	}
};

void build() {
	rmq = new SparseTable<pair<int, int>, Operation>(order);
}

void solve() {
	int q;
	cin >> n >> q;
	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	dfs(0);
	build();
	{
		DS ds;
		ds.update(0, 1);
		dbg(ds.query(1, 2));
	}
	int safe[n] {};
	DS child, actual;
	while(q--) {
		int t;
		cin >> t;
		if(t == 1) {
			int u;
			cin >> u;
			u--;
			int cur = safe[u], ncur = safe[u] ^ 1;
			int change = ncur - cur;
			child.update(u, change);
			actual.update(u, change);
			if(par[u] != -1) {
				child.update(par[u], change);
			}
			safe[u] = ncur;
		}else {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			int l = lca(u, v);
			int ans = child.query(u, v) - actual.query(u, v);
			dbg(child.query(u, v), actual.query(u, v), actual.arr[l]);
			ans += actual.arr[l];
			if(par[l] != -1) {
				ans += actual.arr[par[l]];
			}
			cout << ans << endl;
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
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}