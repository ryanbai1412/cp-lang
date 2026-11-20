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

const int maxn = 1000;

int mans, ccur;
vector<int> graph[maxn];
vector<vector<int>> ans;

int tdfs(int u, int p = -1) {
	if(sz(graph[u]) - (p != -1) == 1) {
		int v = graph[u][0];
		if(v == p) {
			v = graph[u][1];
		}
		return tdfs(v, u);
	}else if(sz(graph[u]) == 1) {
		return 1;
	}
	int ans = 0;
	for(auto &v: graph[u]) {
		if(v != p) {
			ans = max(ans, tdfs(v, u));
		}
	}
	return 1 + ans;
}

void mk(int u) {
	ans.push_back({0, u});
	mans = max(mans, ++ccur);
}

void rm(int u) {
	ans.push_back({1, u});
	ccur--;
}

void mv(int u, int v) {
	ans.push_back({2, u, v});
}

void dfs(int u, int p = -1) {
	if(sz(graph[u]) - (p != -1) == 1) {
		int v = graph[u][0];
		if(v == p) {
			v = graph[u][1];
		}
		dfs(v, u);
		mv(v, u);
		return;
	}else if(sz(graph[u]) == 1) {
		mk(u);
		mv(u, p);
		return;
	}
	mk(u);
	for(auto &v: graph[u]) {
		if(v != p) {
			dfs(v, u);
			rm(u);
		}
	}
	if(p != -1) {
		mv(u, p);
	}
}

void solve() {
	int n;
	cin >> n;
	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	pair<int, int> opt {1e9, 1e9};
	for(int i = 0; i < n; i++) {
		opt = min(opt, pair<int, int> {tdfs(i), i});
	}
	dfs(opt.second);
	assert(mans == opt.first);
	cout << mans << endl;
	cout << sz(ans) << endl;
	for(auto &a: ans) {
		cout << "LBM"[a[0]] << " ";
		for(int i = 1; i < sz(a); i++) {
			cout << a[i] + 1 << " ";
		}
		cout << endl;
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