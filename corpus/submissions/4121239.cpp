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

const int maxn = 200;
const int dx[] = {0, 0, 1, -1}, dy[] = {1, -1, 0, 0};

int n, m;
char arr[maxn][maxn];
pair<int, int> port[maxn][maxn];

int bfs(const pair<int, int>& start, const pair<int, int>& end) {
	queue<tuple<int, int, int>> q;
	int dist[maxn][maxn];
	memset(dist, 0x3f, sizeof(dist));
	auto ibs = [&](int x, int y) { return 0 <= x && x < n && 0 <= y && y < m; };
	auto push = [&](int x, int y, int d) -> void {
		if (ibs(x, y) && arr[x][y] != '#' && !islower(arr[x][y])) {
			if (isupper(arr[x][y])) {
				auto [nx, ny] = port[x][y];
				x = nx;
				y = ny;
			}
			if (d < dist[x][y]) {
				dist[x][y] = d;
				q.emplace(d, x, y);
			}
		}
	};
	push(start.first, start.second, 0);
	while (sz(q)) {
		auto [d, x, y] = q.front();
		q.pop();
		if (d == dist[x][y]) {
			for (int i = 0; i < 4; i++) {
				push(x + dx[i], y + dy[i], d + 1);
			}
		}
	}
	return dist[end.first][end.second];
}

void solve() {
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	pair<int, int> pos[26];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (islower(arr[i][j])) {
				pos[arr[i][j] - 'a'] = {i, j};
			}
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (isupper(arr[i][j])) {
				port[i][j] = pos[arr[i][j] - 'A'];
			}
		}
	}
	pair<int, int> classes[5];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (isdigit(arr[i][j])) {
				classes[arr[i][j] - '0'] = {i, j};
			}
		}
	}
	long ans = 0;
	for (int i = 0; i < 4; i++) {
		ans += bfs(classes[i], classes[i + 1]);
		dbg(i, ans);
	}
	if (ans > long(1e9)) {
		cout << -1 << endl;
	} else {
		cout << ans << endl;
	}
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