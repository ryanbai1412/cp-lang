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

const int maxn = 2e5;
const long inf = 1e18;

using DS = map<long, long>;

long k, ans = inf;
vector<pair<int, int>> graph[maxn];

void put(DS &x, long k, long v) {
	auto it = x.find(k);
	if(it == x.end()) {
		x[k] = v;
	}else {
		it->second = min(it->second, v);
	}
}

long get(const DS &x, long k) {
	auto it = x.find(k);
	if(it == x.end()) {
		return inf;
	}
	return it->second;
}

long merge(DS &a, DS &b, long target) {
	if(sz(a) > sz(b)) {
		swap(a, b);
	}
	long ans = inf;
	for(auto &[k, v]: a) {
		ans = min(ans, v + get(b, target - k));
	}
	for(auto &[k, v]: a) {
		put(b, k, v);
	}
	return ans;
}

DS dfs(int u, int p = -1, long d1 = 0, long d2 = 0) {
	DS cur;
	put(cur, d1, d2);
	for(auto &[v, w]: graph[u]) {
		if(v != p) {
			DS next = dfs(v, u, d1 + w, d2 + 1);
			long cans = merge(next, cur, k + d1 * 2);
			if(cans != inf) {
				ans = min(ans, cans - d2 * 2);
			}
		}
	}
	return cur;
}

int best_path(int n, int k, int edges[][2], int weights[]) {
	::k = k;
	for(int i = 0; i < n - 1; i++) {
		auto &[u, v] = edges[i];
		graph[u].emplace_back(v, weights[i]);
		graph[v].emplace_back(u, weights[i]);
	}
	dfs(0);
	return ans == inf ? -1 : ans;
}

void solve() {
	int n, k;
	cin >> n >> k;
	auto edges = new int[n - 1][2];
	auto weights = new int[n - 1];
	for(int i = 0; i < n - 1; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		edges[i][0] = u;
		edges[i][1] = v;
		weights[i] = w;
	}
	cout << best_path(n, k, edges, weights) << endl;
	delete[] edges;
	delete[] weights;
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