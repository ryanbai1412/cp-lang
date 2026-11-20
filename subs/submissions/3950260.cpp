#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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

const int maxm = 305, inf = 1e9;

bitset<maxm> cur[maxm][maxm], pre[maxm][maxm];

long solve(vector<long> arr) {
	if (*max_element(begin(arr), end(arr)) > maxm ||
		accumulate(begin(arr), end(arr), long(0)) > maxm * 4) {
		return inf;
	}

	int n = sz(arr);
	vector<int> psum(n);
	partial_sum(begin(arr), end(arr), psum.begin());
	pre[0][0][0] = true;
	for (int i = 0; i < n; i++) {
		for (int a = 0; a < maxm; a++) {
			for (int b = 0; b < maxm; b++) {
				for (int c = 0, d = psum[i] - a - b; c < maxm && d >= 0;
					 c++, d--) {
					cur[a][b][c] = pre[a][b][d] || pre[a][c][d] || pre[b][c][d];
				}
			}
		}
		memcpy(pre, cur, sizeof(cur));
	}
	int ans = 1e9;
	for (int a = 0; a < maxm; a++) {
		for (int b = 0; b < maxm; b++) {
			for (int c = 0; c < maxm; c++) {
				if (cur[a][b][c]) {
					ans = min(ans, max({a, b, c, psum[n - 1] - a - b - c}));
				}
			}
		}
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<pair<int, long>> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		if (w > 300) {
			cout << "Impossible!" << endl;
			return;
		}
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	vector<long> dist(n);
	fill(begin(dist), end(dist), long(4e18));
	min_pq<pair<long, int>> pq;
	auto push = [&](int u, long d) -> void {
		if (d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, u);
		}
	};
	push(0, 0);
	while (sz(pq)) {
		auto [d, u] = pq.top();
		pq.pop();
		for (auto& [v, w] : graph[u]) {
			push(v, d + w);
		}
	}
	assert(*max_element(begin(dist), end(dist)) < 4e18);
	long ans = solve(dist) * 2;
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