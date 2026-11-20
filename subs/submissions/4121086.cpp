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

const int maxn = 1e3;
const int dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};

bool arr[maxn][maxn];
int dist[maxn][maxn];

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for (int j = 0; j < m; j++) {
			arr[i][j] = s[j] == 'C';
		}
	}
	memset(dist, 0x3f, sizeof(dist));
	min_pq<tuple<int, int, int>> pq;
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < m;
	};
	auto push = [&](int x, int y, int d) -> void {
		if (ibs(x, y) && d < dist[x][y]) {
			dist[x][y] = d;
			pq.emplace(d, x, y);
		}
	};
	push(n - 1, m - 1, 0);
	while (sz(pq)) {
		auto [d, x, y] = pq.top();
		pq.pop();
		if (d == dist[x][y]) {
			for (int i = 0; i < 4; i++) {
				int cx = x + dx[i], cy = y + dy[i];
				push(cx, cy, d + arr[x][y]);
			}
		}
	}
	cout << dist[0][0] + arr[0][0] << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}