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

struct Comp {
	int ind = 0;
	map<int, int> m {};

	int operator()(int x) {
		int &ans = m[x];
		if(!ans) {
			ans = ++ind;
		}
		return ans - 1;
	}
};

const int maxn = 1e6;

bool vis[maxn];
int ccomp[maxn];
vector<int> graph[maxn];

void dfs(int u, int c) {
	ccomp[u] = c;
	vis[u] = true;
	for(auto &v: graph[u]) {
		if(!vis[v]) {
			dfs(v, c);
		}
	}
}

void solve() {
	int q;
	cin >> q;
	Comp comp;
	vector<pair<int, int>> eq, neq;
	for(int i = 0; i < q; i++) {
		int u, v, t;
		cin >> u >> v >> t;
		u = comp(u);
		v = comp(v);
		if(t) {
			eq.emplace_back(u, v);
		}else {
			neq.emplace_back(u, v);
		}
	}
	int n = comp.ind;
	for(int i = 0; i < n; i++) {
		graph[i].clear();
		vis[i] = false;
	}
	for(auto &[u, v]: eq) {
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	int cind = 0;
	for(int i = 0; i < n; i++) {
		if(!vis[i]) {
			dfs(i, cind++);
		}
	}
	for(auto &[u, v]: neq) {
		if(ccomp[u] == ccomp[v]) {
			cout << "NO" << endl;
			return;
		}
	}
	cout << "YES" << endl;
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