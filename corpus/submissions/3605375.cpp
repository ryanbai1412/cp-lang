#include "bits/extc++.h"

using namespace std;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

const int maxn = 1e5;

int type, mcyc, depth[maxn];
bool vis[maxn];
vector<int> graph[maxn];

void dfs(int u, int d = 0) {
	dbg(u, d);
	vis[u] = true;
	depth[u] = d;
	for(auto &v: graph[u]) {
		if(vis[v]) {
			mcyc = max(mcyc, d - depth[v]);
		}else {
			dfs(v, d + 1);
		}
	}
}

void solve() {
	int n, m;
	cin >> n >> m;
	for(int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	bool ans = false;
	if(type == 1) {

	}else if(type == 2) {
		dfs(0);
		ans = mcyc >= 2;
	}else if(type == 3) {
		dfs(0);
		ans = mcyc >= 3;
	}else if(type == 4) {
		
	}else {

	}
	if(ans) {
		cout << "YES" << endl;
	}else {
		cout << "NO" << endl;
	}
	for(int i = 0; i < n; i++) {
		graph[i].clear();
		vis[i] = false;
		depth[i] = 0;
	}
	mcyc = 0;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> type >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}