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

int n, p[maxn], d[maxn], edges[maxn][2];
vector<int> graph[maxn];

void dfs(int u) {
	for(auto &v: graph[u]) {
		if(v != p[u]) {
			p[v] = u;
			d[v] = d[u] + 1;
			dfs(v);
		}
	}
}

void solve() {
	cin >> n;
	assert(n >= 3);
	for(int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
		auto &[_u, _v] = edges[i];
		_u = u;
		_v = v;
	}
	vector<int> leaves;
	for(int i = 0; i < n; i++) {
		if(sz(graph[i]) == 1) {
			leaves.push_back(i);
		}
	}
	p[leaves[0]] = -1;
	dfs(leaves[0]);
	cout << sz(leaves) - 1 << endl;
	vector<pair<int, int>> ans;
	bool vis[maxn] {};
	priority_queue<pair<int, int>> q;
	for(int i = 1; i < sz(leaves); i++) {
		q.emplace(d[leaves[i]], leaves[i]);
	}
	while(sz(q)) {
		auto[_, u] = q.top();
		q.pop();
		if(!vis[u]) {
			vis[u] = true;
			int cp = p[u];
			if(cp != -1) {
				ans.emplace_back(u, cp);
				q.emplace(d[cp], cp);
			}
		}
	}
	cout << sz(leaves) - 1 + sz(ans) << endl;
	for(int i = 1; i < sz(leaves); i++) {
		cout << "L " << leaves[i] + 1 << endl;
	}
	for(auto &[u, v]: ans) {
		cout << "M " << u + 1 << " " << v + 1 << endl;
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