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

struct LazySegTree {
	int n;
	vector<int> v, lazy;

	LazySegTree(int n): n(n), v(4 * n, 0), lazy(4 * n, -1) {}

	void maintain(int o, int l, int r) {
		if(lazy[o] != -1) {
			v[o] = (r - l + 1) * lazy[o];
		}else {
			if(l < r) {
				v[o] = v[o * 2] + v[o * 2 + 1];
			}else {
				v[o] = 0;
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

	int query(int o, int l, int r, int ql, int qr, int clazy) {
		if(clazy == -1) {
			clazy = lazy[o];
		}
		if(ql <= l && r <= qr) {
			if(clazy != -1) {
				return clazy * (r - l + 1);
			}
			return v[o];
		}else {
			int mid = (l + r) / 2;
			int ans = 0;
			if(ql <= mid) {
				ans += query(o * 2, l, mid, ql, qr, clazy);
			}
			if(mid < qr) {
				ans += query(o * 2 + 1, mid + 1, r, ql, qr, clazy);
			}
			return ans;
		}
	}

	int query(int l, int r) {
		return query(1, 0, n - 1, l, r, -1);
	}
};

const int maxn = 1e5;

int t = 0;
map<int, int> order;
vector<int> graph[maxn];

void dfs(int u, int p = -1) {
	order[u] = t++;
	for(auto &v: graph[u]) {
		if(v != p) {
			dfs(v, u);
		}
	}
}

void solve() {
	t = 0;
	order.clear();
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
	dfs(0);
	LazySegTree st(n);
	while(q--) {
		int t, u, v;
		cin >> t >> u >> v;
		u--;
		v--;
		u = order[u];
		v = order[v];
		if(u > v) {
			swap(u, v);
		}
		if(t == 1) {
			if(u < v) {
				st.update(u + 1, v, 1);
			}
			st.update(u, u, 0);
			if(v + 1 < n) {
				st.update(v + 1, v + 1, 0);
			}
		}else {
			if(u < v) {
				cout << st.query(u + 1, v) << endl;
			}else {
				cout << 0 << endl;
			}
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