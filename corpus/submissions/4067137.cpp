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

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	int hori[n][m - 1];
	int vert[n - 1][m];
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for (int j = 0; j < m - 1; j++) {
			hori[i][j] = s[j] - '0';
		}
	}
	for (int i = 0; i < n - 1; i++) {
		string s;
		cin >> s;
		for (int j = 0; j < m; j++) {
			vert[i][j] = s[j] - '0';
		}
	}
	vector<vector<pair<int, int>>> pieces(n,
										  vector<pair<int, int>>(m, {-1, -1}));
	vector<vector<int>> vis(n, vector<int>(m));
	vector<vector<int>> avis(n, vector<int>(m));
	int clock = 0;
	while (q--) {
		dbg(q);
		clock++;
		int color, level, x, y;
		cin >> color >> level >> x >> y;
		x--;
		y--;
		int ans = -1;
		auto add = [&](int x, int y) -> void {
			if (avis[x][y] != clock) {
				dbg(x + 1, y + 1);
				ans++;
				avis[x][y] = clock;
			}
		};
		add(x, y);
		auto can_land = [&](int x, int y) -> int {
			if (x < 0 || x >= n || y < 0 || y >= m) {
				return false;
			}
			auto [xcolor, xlevel] = pieces[x][y];
			if (xcolor == -1) {
				return 1;
			}
			return (xcolor != color && xlevel <= level) ? 2 : 0;
		};
		dbg("stage 1");
		if (can_land(x - 1, y) && vert[x - 1][y] == 1) {
			add(x - 1, y);
		}
		if (can_land(x + 1, y) && vert[x][y] == 1) {
			add(x + 1, y);
		}
		if (can_land(x, y - 1) && hori[x][y - 1] == 1) {
			add(x, y - 1);
		}
		if (can_land(x, y + 1) && hori[x][y] == 1) {
			add(x, y + 1);
		}
		dbg("stage 2");
		for (int i = x + 1; i < n; i++) {
			if (vert[i - 1][y] != 2) {
				break;
			}
			int t = can_land(i, y);
			if (t) {
				add(i, y);
				if (t == 2) {
					break;
				}
			} else {
				break;
			}
		}
		for (int i = x - 1; i >= 0; i--) {
			if (vert[i][y] != 2) {
				break;
			}
			int t = can_land(i, y);
			if (t) {
				add(i, y);
				if (t == 2) {
					break;
				}
			} else {
				break;
			}
		}
		for (int i = y + 1; i < m; i++) {
			if (hori[x][i - 1] != 2) {
				break;
			}
			int t = can_land(x, i);
			if (t) {
				add(x, i);
				if (t == 2) {
					break;
				}
			} else {
				break;
			}
		}
		for (int i = y - 1; i >= 0; i--) {
			if (hori[x][i] != 2) {
				break;
			}
			int t = can_land(x, i);
			if (t) {
				add(x, i);
				if (t == 2) {
					break;
				}
			} else {
				break;
			}
		}
		dbg("stage 3");
		{
			vector<pair<int, int>> q;
			auto push = [&](int x, int y) -> void {
				if (vis[x][y] != clock) {
					add(x, y);
					vis[x][y] = clock;
					q.emplace_back(x, y);
				}
			};
			push(x, y);
			while (sz(q)) {
				auto [x, y] = q.back();
				q.pop_back();
				int i;
				i = x + 1;
				{
					int t = can_land(i, y);
					if (t && vert[i - 1][y] == 3) {
						add(i, y);
						if (t != 2) {
							push(i, y);
						}
					}
				}
				i = x - 1;
				{
					int t = can_land(i, y);
					if (t && vert[i][y] == 3) {
						add(i, y);
						if (t != 2) {
							push(i, y);
						}
					}
				}
				i = y + 1;
				{
					int t = can_land(x, i);
					if (t && hori[x][i - 1] == 3) {
						add(x, i);
						if (t != 2) {
							push(x, i);
						}
					}
				}
				i = y - 1;
				{
					int t = can_land(x, i);
					if (t && hori[x][i] == 3) {
						add(x, i);
						if (t != 2) {
							push(x, i);
						}
					}
				}
			}
		}
		cout << ans << endl;
		pieces[x][y] = {color, level};
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}