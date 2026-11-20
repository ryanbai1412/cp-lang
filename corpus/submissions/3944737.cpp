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

const int maxm = 600, inf = 1e9;
const int maxa = maxm / 4 + 5, maxb = maxm / 3 + 5, maxc = maxm / 2 + 5;

int cur[maxa][maxb][maxc], pre[maxa][maxb][maxc];

long solve(vector<long> arr) {
	if (*max_element(begin(arr), end(arr)) > 600 ||
		accumulate(begin(arr), end(arr), long(0)) > 2400) {
		return inf;
	}

	int n = sz(arr);
	vector<long> psum(n + 1);
	partial_sum(begin(arr), end(arr), psum.begin() + 1);
	int tmp[4];
	bool f = true;
	auto gprev = [&](int a, int b, int c, int d) -> int {
		if (a + b + c + d > maxm) {
			return inf;
		}
		if (f) {
			return max({a, b, c, d});
		}
		tmp[0] = a;
		tmp[1] = b;
		tmp[2] = c;
		tmp[3] = d;
		sort(begin(tmp), end(tmp));
		return pre[tmp[0]][tmp[1]][tmp[2]];
	};
	for (int i = n - 1; i >= 0; i--) {
		memset(cur, 0x3f, sizeof(cur));
		for (int a = 0; a < maxa; a++) {
			for (int b = a; b < maxb; b++) {
				for (int c = b; c < maxc; c++) {
					int d = psum[i] - a - b - c;
					if (c <= d && a + b + c + d < maxm) {
						int x = arr[i];
						cur[a][b][c] =
							min({gprev(a + x, b, c, d), gprev(a, b + x, c, d),
								 gprev(a, b, c + x, d), gprev(a, b, c, d + x)});
					}
				}
			}
		}
		memcpy(pre, cur, sizeof(cur));
		f = false;
	}
	return cur[0][0][0];
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<pair<int, long>> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		if (w > 300) {
//			cout << "Impossible!" << endl;
//			return;
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
	for (auto& a : dist) {
		a *= 2;
	}
	long ans = solve(dist);
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