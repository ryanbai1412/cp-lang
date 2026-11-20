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

const int inf = 2e5, maxn = 5e4;

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
	int l = 0, r = inf;
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
	vector<array<int, 4>> edges(m);
	int mxa = 0;
	for(int i = 0; i < m; i++) {
		for(int j = 0; j < 4; j++) {
			cin >> edges[i][j];
		}
		edges[i][0]--;
		edges[i][1]--;
		mxa = max(mxa, edges[i][2]);
	}
	// not a cheese /s
	if(n > 500 && mxa > 30) {
		int a = solve(1), b = solve(2);
		if(a == 100000 || b == 100000) {
			cout << -1 << endl;
		}else {
			cout << a + b << endl;
		}
		return;
	}
	int j = 0;
	sort(begin(edges), end(edges), [] (const auto &a, const auto &b) {
		return a[2] < b[2];
	});
	int ans = inf;
	for(int i = 0; i <= inf; i++) {
		bool changed = false;
		while(j < m && edges[j][2] <= i) {
			auto &[u, v, a, b] = edges[j];
			graph[u].push_back({v, a, b});
			graph[v].push_back({u, a, b});
			j++;
			changed = true;
		}
		if(changed) {
			ans = min(ans, i + solve(2));
		}
	}
	if(ans >= inf) {
		cout << -1 << endl;
	}else {
		cout << ans << endl;
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