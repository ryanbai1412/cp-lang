#include "bits/extc++.h"

using namespace std;

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

#define sz(x) int((x).size())

const int maxn = 2e5, bsize = 400;

struct RMQ {
	int n, m;
	vector<vector<int>> vmin;

	RMQ(const vector<int> &arr): n(sz(arr)), m(__lg(n) + 1), vmin(m, vector<int>(n)) {
		vmin[0] = arr;
		for(int i = 1; i < m; i++) {
			for(int j = 0; j + (1 << i) <= n; j++) {
				vmin[i][j] = min(vmin[i - 1][j], vmin[i - 1][j + (1 << (i - 1))]);
			}
		}
	}

	int query(int l, int r) {
		r++;
		int x = __lg(r - l);
		return min(vmin[x][l], vmin[x][r - (1 << x)]);
	}
};

RMQ rmq{{0}};
int d[maxn], tin[maxn], cnt[maxn], siz[maxn];
long ans, dist[maxn];
vector<int> torder, order, graph[maxn];

void pdfs(int u, int p = -1) {
	tin[u] = sz(torder);
	order.push_back(u);
	torder.push_back(tin[u]);
	for(auto &v: graph[u]) {
		if(v != p) {
			d[v] = d[u] + 1;
			pdfs(v, u);
			order.push_back(u);
			torder.push_back(tin[u]);
		}
	}
}

void dfs1(int u, int p = -1) {
	dist[u] = 0;
	siz[u] = cnt[u];
	for(auto &v: graph[u]) {
		if(v != p) {
			dfs1(v, u);
			siz[u] += siz[v];
			dist[u] += siz[v] + dist[v];
		}
	}
}

void dfs2(int u, int p, long cdist) {
	ans = min(ans, cdist);
	for(auto &v: graph[u]) {
		if(v != p) {
			dfs2(v, u, cdist - siz[v]);
		}
	}
}

int lca(int u, int v) {
	if(tin[u] > tin[v]) {
		swap(u, v);
	}
	return order[rmq.query(tin[u], tin[v])];
}

void solve() {
	int n, q;
	cin >> n >> q;
	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	pdfs(0);
	rmq = {torder};
	while(q--) {
		int m;
		cin >> m;
		vector<int> v(m);
		for(int i = 0; i < m; i++) {
			cin >> v[i];
			v[i]--;
		}
		if(sz(v) <= bsize) {
			ans = LONG_MAX;
			for(auto &a: v) {
				long cur = 0;
				for(auto &b: v) {
					cur += d[b] - d[lca(a, b)];
				}
				ans = min(ans, cur);
				if(cur == 1) {
					dbg(a);
				}
			}
			cout << ans << endl;
		}else {
			for(auto &a: v) {
				cnt[a] = 1;
			}
			ans = LONG_MAX;
			dfs1(0);
			dbg(dist[0]);
			dfs2(0, -1, dist[0]);
			cout << ans << endl;
			for(auto &a: v) {
				cnt[a] = 0;
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
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}