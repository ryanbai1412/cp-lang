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

struct Game {
	int n;

	explicit Game(int n): n(n) {}
	
	tuple<bool, int, int> init() {
		if (n & 1) {
			int m = (n + 1) / 2;
			return {true, m, m};
		} else {
			return {false, 0, 0};
		}
	}

	pair<int, int> next(int x, int y) {
		return {n + 1 - x, n + 1 - y};
	}
};

void stress() {
	mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());
	auto rint = [&](long l, long r) -> long {
		return cowng() % (r - l + 1) + l;
	};
	while (true) {
		int n = rint(1, 10);
		vector<pair<int, int>> ord;
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				ord.emplace_back(i, j);
			}
		}
		shuffle(begin(ord), end(ord), cowng);
		cerr << n << endl;
		for (auto& [a, b] : ord) {
			cerr << a << " " << b << endl;
		}
		cerr << endl;
		Game g(n);
		bool vis[100][100] {};
		auto upd = [&](int x, int y) -> void {
			vis[x][y] = true;
			for (int i = 0; i < 4; i++) {
				int cx = x + dx[i], cy = y + dy[i];
				if (0 <= cx && cx < n && 0 <= cy && cy < n) {
					vis[cx][cy] = true;
				}
			}
		};
		{
			auto [first, x, y] = g.init();
			x--;
			y--;
			if (first) {
				dbg(x, y);
				upd(x, y);
			}
		}
		for (int i = 0; i < sz(ord); i++) {
			for (; i < sz(ord) && vis[ord[i].first][ord[i].second]; i++);
			if (i == sz(ord)) {
				break;
			}
			auto [x, y] = ord[i];
			upd(x, y);
			dbg(x, y);
			auto [nx, ny] = g.next(x + 1, y + 1);
			nx--;
			ny--;
			dbg(nx, ny);
			if (vis[nx][ny]) {
				return;
			}
			upd(nx, ny);
		}
	}
}

void solve() {
	int n, t;
	cin >> n >> t;
	while (t--) {
		Game g(n);
		{
			auto [first, x, y] = g.init();
			if (first) {
				cout << 1 << endl;
				cout << x << " " << y << endl;
			} else {
				cout << 2 << endl;
			}
		}
		while (true) {
			int x, y;
			cin >> x >> y;
			assert(x != -1 && y != -1);
			if (!x && !y) {
				break;
			}
			auto [nx, ny] = g.next(x, y);
			cout << nx << " " << ny << endl;
		}
	}
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
//	stress();
	solve();
}