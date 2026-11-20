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

int n;
vector<pair<int, int>> graph[maxn];

vector<long> bfs(int u) {
	vector<long> dist(n, -1);
	queue<int> q;
	dist[u] = 0;
	q.push(u);
	while(sz(q)) {
		int u = q.front();
		q.pop();
		for(auto &[v, w]: graph[u]) {
			if(dist[v] == -1) {
				dist[v] = dist[u] + w;
				q.push(v);
			}
		}
	}
	return dist;
}

int farthest(int u) {
	auto cur = bfs(u);
	return max_element(begin(cur), end(cur)) - cur.begin();
}

void solve() {
	int m;
	cin >> n >> m;
	for(int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	int a = farthest(0), b = farthest(a);
	auto bfsa = bfs(a), bfsb = bfs(b);
	long ans = 0;
	for(int i = 0; i < n; i++) {
		ans = max(ans, min(bfsa[i], bfsb[i]));
	}
	cout << ans + bfsa[b] << endl;
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