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

const int dx[] = {-1, 1, 0, 0}, dy[] = {0, 0, -1, 1};

void solve() {
	int n, t;
	cin >> n >> t;
	string arr[n];
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < n;
	};
	auto conv = [&](int x, int y) -> int { return n * x + y; };
	long dist[n * n];
	fill(dist, dist + n * n, long(4e18));
	min_pq<tuple<long, int, int>> pq;
	auto push = [&](int x, int y, long d) -> void {
		int u = conv(x, y);
		if (d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, x, y);
		}
	};
	push(0, 0, 0);
	while (sz(pq)) {
		auto [d, x, y] = pq.top();
		pq.pop();
		for (int i = 0; i < 4; i++) {
			int cx = x + dx[i];
			int cy = y + dy[i];
			if (ibs(cx, cy)) {
				int w = 1;
				if (arr[cx][cy] == 'b') {
					w += t;
				}
				push(cx, cy, d + w);
			}
		}
	}
	cout << dist[n * n - 1] << endl;
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