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

const int maxn = 100000;

vector<pair<int, int>> graph[maxn];

int travel_plan(int n, int m, int edges[][2], int weights[], int k, int escapes[]) {
	for(int i = 0; i < m; i++) {
		auto &[u, v] = edges[i];
		int w = weights[i];
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	int dist[n], used[n] {};
	fill(dist, dist + n, INT_MAX);
	mpq<pair<int, int>> pq;
	auto push = [&](int u, int d) -> void {
		if(d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, u);
		}
	};
	for(int i = 0; i < k; i++) {
		push(escapes[i], 0);
	}
	while(sz(pq)) {
		auto [d, u] = pq.top();
		dbg(u, d);
		pq.pop();
		if(d == dist[u]) {
			if(!u) {
				return d;
			}
			for(auto &[v, w]: graph[u]) {
				if(!used[v]) {
					used[v] = d + w;
				}else if(d < dist[v]) {
					push(v, max(used[v], d + w));
					used[v] = min(used[v], d + w);
				}
			}
		}
	}
	assert(false);
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	auto edges = new int[m][2];
	auto weights = new int[m];
	for(int i = 0; i < m; i++) {
		cin >> edges[i][0] >> edges[i][1] >> weights[i];
	}
	auto exits = new int[k];
	for(int i = 0; i < k; i++) {
		cin >> exits[i];
	}
	cout << travel_plan(n, m, edges, weights, k, exits) << endl;
	delete[] edges;
	delete[] weights;
	delete[] exits;
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