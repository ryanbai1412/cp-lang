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

const int maxn = 1500 + 5, dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	int arr[n][m];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < m;
	};
	bool vis[maxn][maxn] {};
	queue<tuple<int, int, int, int>> q;
	auto push = [&](int x, int y, int from, int d) {
		if (d <= k && ibs(x, y) && !vis[x][y]) {
			vis[x][y] = true;
			q.emplace(x, y, from, d);
		}
	};
	vector<tuple<int, int, int>> ord;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (arr[i][j]) {
				ord.emplace_back(arr[i][j], i, j);
			}
		}
	}
	sort(begin(ord), end(ord));
	for (auto& [v, x, y] : ord) {
		push(x, y, v, 0);
	}
	while (sz(q)) {
		auto [x, y, from, d] = q.front();
		if (!arr[x][y]) {
			arr[x][y] = from;
		}
		q.pop();
		for (int i = 0; i < 4; i++) {
			push(x + dx[i], y + dy[i], from, d + 1);
		}
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			cout << arr[i][j] << " \n"[j == m - 1];
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