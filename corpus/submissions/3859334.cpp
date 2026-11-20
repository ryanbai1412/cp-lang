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

const int maxn = 3e5;

int t = 1, ans, vis1[maxn], vis2[maxn], tin[maxn], tout[maxn], depth[maxn];
vector<int> graph[maxn], graph1[maxn], graph2[maxn];

void dfs(int u, int p = -1) {
	tin[u] = t++;
	for(auto &v: graph[u]) {
		if(v != p) {
			depth[v] = depth[u] + 1;
			dfs(v, u);
		}
	}
	tout[u] = t - 1;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

void dfs1(int u) {
	vis1[u] = t;
	for(auto &v: graph1[u]) {
		if(vis1[v] != t) {
			dfs1(v);
		}
	}
}

void dfs2(int u) {
	vis2[u] = t;
	ans += vis1[u] == t;
	for(auto &v: graph2[u]) {
		if(vis2[v] != t) {
			dfs2(v);
		}
	}
}

void solve() {
	int n, m, q, k;
	cin >> n >> m >> q >> k;
	if(n <= 1000) {
		auto push = [&](int u, int v) -> void {
			graph1[u].push_back(v);
			graph2[v].push_back(u);
		};
		auto pop = [&](int u, int v) -> void {
			graph1[u].pop_back();
			graph2[v].pop_back();
		};
		for(int i = 0; i < m; i++) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			push(u, v);
		}
		while(q--) {
			t++;
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			pair<int, int> extra[k];
			for(auto &[u, v]: extra) {
				cin >> u >> v;
				u--;
				v--;
				push(u, v);
			}
			ans = 0;
			dfs1(u);
			dfs2(v);
			cout << ans << endl;
			for(auto &[u, v]: extra) {
				pop(u, v);
			}
		}
	}else {
		int in[n] {};
		for(int i = 0; i < m; i++) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			graph[u].push_back(v);
			in[v]++;
		}
		int root = -1;
		for(int i = 0; i < n; i++) {
			if(!in[i]) {
				root = i;
			}
		}
		assert(root != -1);
		depth[root] = 0;
		dfs(root);
		while(q--) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			if(anc(u, v)) {
				cout << depth[v] - depth[u] + 1 << endl;
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
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}