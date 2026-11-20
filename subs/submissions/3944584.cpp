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

const string walls = "+-|";

const int dx[] = {-1, 1, 0, 0}, dy[] = {0, 0, -1, 1};

void solve() {
	int n, m;
	cin >> n >> m;
	n *= 2;
	m *= 2;
	n++;
	m++;
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < m;
	};
	vector<bool> arr[n];
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		arr[i].resize(m);
		for (int j = 0; j < m; j++) {
			arr[i][j] = walls.find(s[j]) != string::npos;
		}
	}
	vector<bool> vis[n];
	for (auto& a : vis) {
		a.resize(m);
	}
	auto dfs = [&](int x, int y) -> int {
		int cnt = 0;
		vector<pair<int, int>> s;
		auto push = [&](int x, int y) -> void {
			if (!vis[x][y]) {
				vis[x][y] = true;
				cnt++;
				s.emplace_back(x, y);
			}
		};
		push(x, y);
		while(sz(s)) {
			auto [x, y] = s.back();
			s.pop_back();
			for (int i = 0; i < 4; i++) {
				int cx = x + dx[i];
				int cy = y + dy[i];
				if (ibs(cx, cy) && !arr[cx][cy]) {
					push(cx + dx[i], cy + dy[i]);
				}
			}
		}
		return cnt;
	};
	vector<int> ans;
	for (int i = 1; i < n; i += 2) {
		for (int j = 1; j < m; j += 2) {
			if (!vis[i][j]) {
				ans.push_back(dfs(i, j));
			}
		}
	}
	sort(begin(ans), end(ans), greater<>());
	for (int i = 0; i < 4; i++) {
		cout << ans[i] << " ";
	}
	cout << endl;
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