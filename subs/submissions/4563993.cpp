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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};

void solve() {
	int n, m;
	cin >> n >> m;
	auto ibs = [&](int x, int y) -> bool {
		return 0 <= x && x < n && 0 <= y && y < m;
	};
	bool arr[n][m], vis[n][m] {};
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for (int j = 0; j < m; j++) {
			arr[i][j] = s[j] == 'H';
		}
	}
	set<pair<int, int>> h, s;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			if (arr[i][j]) {
				h.emplace(i, j);
			} else {
				s.emplace(i, j);
			}
		}
	}
	vector<pair<int, int>> ans;
	while (sz(h)) {
		auto [x, y] = *h.begin();
		ans.emplace_back(x, y);
		h.erase(h.begin());
		vis[x][y] = true;
		for (int i = 0; i < 4; i++) {
			int cx = x + dx[i], cy = y + dy[i];
			if (ibs(cx, cy) && !vis[cx][cy]) {
				if (arr[cx][cy]) {
					h.erase({cx, cy});
					s.emplace(cx, cy);
				} else {
					s.erase({cx, cy});
					h.emplace(cx, cy);
				}
				arr[cx][cy] ^= true;
			}
		}
	}
	if (sz(ans) == n * m) {
		for (auto& [x, y] : ans) {
			cout << x + 1 << " " << y + 1 << endl;
		}
	} else {
		cout << -1 << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}