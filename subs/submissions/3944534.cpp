#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
		 << ": ";                                          \
	dbgh(__VA_ARGS__)
#else
#define cerr   \
	if (false) \
	cerr
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
	int n, m;
	cin >> n >> m;
	vector<pair<int, long>> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	long dist[n];
	fill(dist, dist + n, long(4e18));
	min_pq<pair<long, int>> pq;
	auto push = [&](int u, long d) -> void {
		if (d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, u);
		}
	};
	push(0, 0);
	while(sz(pq)) {
		auto [d, u] = pq.top();
		pq.pop();
		for (auto& [v, w] : graph[u]) {
			push(v, d + w);
		}
	}
	sort(dist, dist + n, greater<>());
	long t[4] {};
	for (auto& a : dist) {
		a *= 2;
		dbg(a);
		*min_element(begin(t), end(t)) += a;
	}
	long ans = *max_element(begin(t), end(t));
	if (ans <= 600) {
		cout << ans << endl;
	} else {
		cout << "Impossible!" << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}