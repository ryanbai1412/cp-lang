#include "bits/extc++.h"

using namespace std;

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

const int maxn = 5e4;

int n, ind, bound;
bool vis[maxn];
vector<array<int, 3>> graph[maxn];

void dfs(int u) {
	vis[u] = true;
	for(auto &a: graph[u]) {
		if(!vis[a[0]] && a[ind] <= bound) {
			dfs(a[0]);
		}
	}
}

bool valid(int _bound) {
	bound = _bound;
	memset(vis, 0, sizeof(vis));
	dfs(0);
	return vis[n - 1];
}

int solve(int _ind) {
	ind = _ind;
	int l = 0, r = 1e5;
	while(l < r) {
		int mid = (l + r) / 2;
		if(valid(mid)) {
			r = mid;
		}else {
			l = mid + 1;
		}
	}
	return l;
}

void solve() {
	int m;
	cin >> n >> m;
	for(int i = 0; i < m; i++) {
		int u, v, a, b;
		cin >> u >> v >> a >> b;
		u--;
		v--;
		graph[u].push_back({v, a, b});
		graph[v].push_back({u, a, b});
	}
	int a = solve(1), b = solve(2);
	if(a == 100000 || b == 100000) {
		cout << -1 << endl;
		return;
	}
	cout << a + b << endl;
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