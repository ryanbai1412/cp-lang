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

struct ST {
	int n;
	vector<int> v, lazy;

	ST(int n): n(n), v(4 * n), lazy(4 * n, -1) {}

	inline int apply(int l, int r, int clazy, int x) {
		if(clazy == -1) {
			return x;
		}
		return clazy * (r - l + 1);
	}

	inline void pushdown(int o) {
		if(lazy[o] != -1) {
			lazy[o * 2] = lazy[o * 2 + 1] = lazy[o];
			lazy[o] = -1;
		}
	}

	inline void maintain(int o, int l, int r) {
		int children = 0;
		if(l < r) {
			children = v[o * 2] + v[o * 2 + 1];
		}
		v[o] = apply(l, r, lazy[o], children);
	}

	void update(int o, int l, int r, int ql, int qr, int x) {
		if(ql <= l && r <= qr) {
			lazy[o] = x;
		}else {
			int mid = (l + r) / 2;
			pushdown(o);
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

	int query(int o, int l, int r, int ql, int qr, int clazy) {
		if(ql <= l && r <= qr) {
			return apply(l, r, clazy, v[o]);
		}
		if(clazy == -1) {
			clazy = lazy[o];
		}
		int ans = 0;
		int mid = (l + r) / 2;
		if(ql <= mid) {
			ans += query(o * 2, l, mid, ql, qr, clazy);
		}
		if(mid < qr) {
			ans += query(o * 2 + 1, mid + 1, r, ql, qr, clazy);
		}
		return ans;
	}

	int query(int l, int r) {
		return query(1, 0, n - 1, l, r, -1);
	}
};

const int maxn = 100000;

int n, t, p[maxn], tin[maxn], tout[maxn], siz[maxn], chain[maxn];
vector<int> graph[maxn];

void pdfs(int u) {
	siz[u] = 1;
	for(auto &v: graph[u]) {
		pdfs(v);
		p[v] = u;
		siz[u] += siz[v];
		if(siz[v] > siz[graph[u][0]]) {
			swap(v, graph[u][0]);
		}
	}
}

void dfs(int u) {
	tin[u] = t++;
	if(sz(graph[u])) {
		int h = graph[u][0];
		chain[h] = chain[u];
		for(auto &v: graph[u]) {
			if(v != h) {
				chain[v] = v;
			}
			dfs(v);
		}
	}
	tout[u] = t - 1;
}

struct DS {
	ST st;

	DS(): st(n) {}

	int rootPath(int u) {
		int ans = 0;
		while(u != -1) {
			int nu = chain[u];
			dbg(u, nu);
			ans += (tin[u] - tin[nu] + 1) - st.query(tin[nu], tin[u]);
			st.update(tin[nu], tin[u], 1);
			u = p[nu];
		}
		return ans;
	}

	int subtree(int u) {
		int ans = st.query(tin[u], tout[u]);
		st.update(tin[u], tout[u], 0);
		return ans;
	}
};

void solve() {
	cin >> n;
	for(int i = 1; i < n; i++) {
		int u;
		cin >> u;
		graph[u].push_back(i);
	}
	p[0] = -1;
	pdfs(0);
	dfs(0);
	DS ds;
	int q;
	cin >> q;
	while(q--) {
		string s;
		int u;
		cin >> s >> u;
		if(s == "install") {
			cout << ds.rootPath(u) << endl;
		}else {
			cout << ds.subtree(u) << endl;
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