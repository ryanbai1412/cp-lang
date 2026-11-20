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

const int maxn = 205, dx[] = {0, 0, 1, -1}, dy[] = {1, -1, 0, 0};

void solve() {
	int n, m;
	cin >> n >> m;
	string arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	pair<int, int> portal[26], period[5];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (islower(arr[i][j])) {
				portal[arr[i][j] - 'a'] = {i, j};
			} else if (isdigit(arr[i][j])) {
				dbg(arr[i][j] - '0');
				period[arr[i][j] - '0'] = {i, j};
			}
		}
	}
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < m;
	};
	auto bfs = [&](pair<int, int> s, pair<int, int> e) -> int {
		int dist[maxn][maxn];
		memset(dist, 0x3f, sizeof(dist));
		deque<pair<int, int>> q;
		auto push = [&](int u, int v, int d) -> void {
			if (d < dist[u][v]) {
				dist[u][v] = d;
				q.emplace_back(u, v);
			}
		};
		push(s.first, s.second, 0);
		while (sz(q)) {
			auto [x, y] = q.front();
			q.pop_front();
			if (dist[x][y] == -1) {
				continue;
			}
			if (pair<int, int> {x, y} == e) {
				return dist[x][y];
			}
			for (int i = 0; i < 4; i++) {
				int cx = x + dx[i], cy = y + dy[i];
				if (ibs(cx, cy) && arr[cx][cy] != '#') {
					if (islower(arr[cx][cy])) {
						continue;
					} else if (isupper(arr[cx][cy])) {
						tie(cx, cy) = portal[arr[cx][cy] - 'A'];
					}
					push(cx, cy, dist[x][y] + 1);
				}
			}
			dist[x][y] = -1;
		}
		return -1;
	};
	int ans = 0;
	for (int i = 0; i < 4; i++) {
		int cur = bfs(period[i], period[i + 1]);
		if (cur == -1) {
			cout << -1 << endl;
			return;
		}
		ans += cur;
	}
	cout << ans << endl;
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