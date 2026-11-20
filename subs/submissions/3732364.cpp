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

#define int long

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

#ifdef LOCAL
void answer(int x) {
	cout << x << endl;
}
#endif

const int maxn = 300100;

int graph[maxn], cnt[maxn], vcnt[maxn], cycind = 1, cyclen[maxn], vis[maxn], chaind = 0, dist[maxn], gcind, cmod, target;
pair<int, int> cycle[maxn], chain[maxn];
vector<int> path, chains[maxn];
vector<pair<int, int>> igraph[maxn];
rt<pair<int, int>> treach[maxn];

void dfs(int u) {
	if(vis[u]) {
		if(vis[u] == 1) {
			path.push_back(u);
			int i = sz(path) - 1;
			do {
				cycle[path[i--]] = {cycind, 0};
			}while(path[i] != u);
			cyclen[cycind++] = sz(path) - i - 1;
			path.pop_back();
		}
		return;
	}
	vis[u] = 1;
	path.push_back(u);
	dfs(graph[u]);
	if(!cycle[u].first) {
		cycle[u] = cycle[graph[u]];
		cycle[u].second++;
	}
	vis[u] = 2;
	path.pop_back();
}

void cdfs(int u, int d = 0) {
	if(!cycle[u].second) {
		return;
	}
	assert(cycle[u].second);
	if(d >= sz(chains[chaind])) {
		chains[chaind].push_back(cnt[u]);
	}else {
		chains[chaind][d] += cnt[u];
	}
	chain[u] = {chaind, d};
	for(auto &[v, _]: igraph[u]) {
		cdfs(v, d + 1);
	}
}

void ddfs(int u) {
	if(dist[u] != -1) {
		return;
	}
	if(u == target) {
		dist[u] = 0;
	}else {
		ddfs(graph[u]);
	}
	dist[u] = dist[graph[u]] + 1;
	int cur = dist[u] % cmod;
	for(int i = 0; i < cnt[u]; i++) {
		treach[cur].insert({dist[u], gcind++});
	}
}

void count_routes(int n, int m, int k, int edges[][2], int q, int queries[]) {
	memset(graph, -1, sizeof(graph));
	auto process = [&] (int u, int v) -> void {
		if(vcnt[u] == 0) {
			if(vcnt[v] == 0) {
				graph[u * 2] = graph[u * 2 + 1] = v * 2 + 1;
			}else {
				graph[u * 2] = graph[u * 2 + 1] = v * 2;
			}
		}else if(vcnt[u] == 1) {
			if(vcnt[v] == 0) {
				graph[u * 2 + 1] = v * 2 + 1;
			}else {
				graph[u * 2 + 1] = v * 2;
			}
		}
	};
	for(int i = 0; i < m; i++) {
		auto &[u, v] = edges[i];
		process(u, v);
		process(v, u);
		igraph[u].emplace_back(v, i);
		igraph[v].emplace_back(u, i);
		vcnt[u]++;
		vcnt[v]++;
	}
	for(int i = 0; i < n; i++) {
		cnt[i * 2] = 1;
		cnt[i * 2 + 1] = 0;
	}
	for(int i = 0; i < n * 2; i++) {
		igraph[i].clear();
	}
	for(int i = 0; i < n * 2; i++) {
		dbg(i, graph[i]);
		dfs(i);
		igraph[graph[i]].emplace_back(i, 0);
	}
	for(int i = 0; i < n * 2; i++) {
		if(!cycle[graph[i]].second && cycle[i].second) {
			cdfs(i);
			chaind++;
		}
	}
	int ans[q] {};
	for(int target: {k * 2, k * 2 + 1}) {
		::target = target;
		if(!cycle[target].second) {
			for(auto &a: treach) {
				a.clear();
			}
			cmod = cyclen[cycle[target].first];
			memset(dist, -1, sizeof(dist));
			for(int i = 0; i < n * 2; i++) {
				if(cycle[i].first == cycle[target].first) {
					ddfs(i);
				}
			}
		}
		for(int i = 0; i < q; i++) {
			int x = queries[i];
			if(cycle[target].second) {
				auto[chaind, icind] = chain[target];
				icind += x;
				if(icind < sz(chains[chaind])) {
					ans[i] += chains[chaind][icind];
				}
			}else {
				ans[i] += treach[x % cmod].order_of_key({x, 1e9});
			}
		}
	}
	for(int i = 0; i < q; i++) {
		answer(ans[i]);
	}
}

void solve() {
	int n, m, k, q;
	cin >> n >> m >> k >> q;
	auto edges = new int[m][2];
	for(int i = 0; i < m; i++) {
		cin >> edges[i][0] >> edges[i][1];
	}
	auto queries = new int[q];
	for(int i = 0; i < q; i++) {
		cin >> queries[i];
	}
	count_routes(n, m, k, edges, q, queries);
	delete[] edges;
	delete[] queries;
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