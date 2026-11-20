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

const int MAXN = 100 + 5, DX[] = {0, 0, -1, 1}, DY[] = {-1, 1, 0, 0};
const string DIRS = "LRUD";

void solve() {
	int n, m;
	cin >> n >> m;
	string arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < m;
	};
	bool bad[MAXN][MAXN];
	int sx, sy;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			bad[i][j] = arr[i][j] == 'W';
			if (arr[i][j] == 'S') {
				tie(sx, sy) = tie(i, j);
			}
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (arr[i][j] == 'C') {
				auto go = [&](int dx, int dy) -> void {
					int x = i, y = j;
					while (ibs(x, y) && arr[x][y] != 'W') {
						if (arr[x][y] == '.' || arr[x][y] == 'C' || arr[x][y] == 'S') {
							bad[x][y] = true;
						}
						x += dx;
						y += dy;
					}
				};
				for (int k = 0; k < 4; k++) {
					go(DX[k], DY[k]);
				}
			}
		}
	}
	int dist[MAXN][MAXN];
	bool vis[MAXN][MAXN] {};
	memset(dist, 0x3f, sizeof(dist));
	deque<pair<int, int>> q;
	auto push = [&](bool front, int x, int y, int d) -> void {
		if (ibs(x, y) && d < dist[x][y]) {
			dist[x][y] = d;
			if (front) {
				q.emplace_front(x, y);
			} else {
				q.emplace_back(x, y);
			}
		}
	};
	if (!bad[sx][sy]) {
		push(false, sx, sy, 0);
	}
	while (sz(q)) {
		auto [x, y] = q.front();
		q.pop_front();
		if (vis[x][y]) {
			continue;
		}
		vis[x][y] = true;
		dbg(x, y);
		int d = dist[x][y];
		auto go = [&](bool front, int k) -> void {
			int cx = x + DX[k], cy = y + DY[k];
			if (ibs(cx, cy) && !bad[cx][cy]) {
				push(front, cx, cy, d);
			}
		};
		auto dir = DIRS.find(arr[x][y]);
		if (dir != string::npos) {
			dbg(x, y, int(dir), x + DX[dir], y + DY[dir]);
			go(true, int(dir));
		} else {
			d++;
			for (int k = 0; k < 4; k++) {
				go(false, k);
			}
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (arr[i][j] == '.') {
				if (dist[i][j] >= 1e9) {
					cout << -1 << endl;
				} else {
					cout << dist[i][j] << endl;
				}
			}
		}
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