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

const int maxn = 1e6;

int csz;
bool vis[maxn];
array<int, 2> graph[maxn];

void dfs(int src) {
	stack<int> s;
	s.push(src);
	vis[src] = true;
	while(sz(s)) {
		int u = s.top();
		s.pop();
		csz++;
		for(auto &v: graph[u]) {
			if(v == -1) {
				break;
			}
			if(!vis[v]) {
				vis[v] = true;
				s.push(v);
			}
		}
	}
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	for(int i = 0; i < n; i++) {
		graph[i].fill(-1);
	}
	auto pb = [&](int ind, int u) {
		for(auto &a: graph[ind]) {
			if(a == -1) {
				a = u;
				break;
			}
		}
	};
	for(int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		pb(u, v);
		pb(v, u);
	}
	int ans1 = 0, ans2 = 0;
	for(int i = 0; i < n; i++) {
		if(graph[i][1] == -1 && !vis[i]) {
			csz = 0;
			dfs(i);
			ans1 += csz / k;
			ans2 += (csz - 1) / k;
		}
	}
	for(int i = 0; i < n; i++) {
		if(!vis[i]) {
			csz = 0;
			dfs(i);
			ans1 += csz / k;
			if(csz != k) {
				ans2 += (csz + k - 1) / k;
			}
			dbg(i, csz, ans2);
		}
	}
	cout << ans1 * k << " " << ans2 << endl;
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