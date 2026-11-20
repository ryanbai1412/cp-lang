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

const int inf = 1e9 + 1;

bool process(int arr[3][3]) {
	bool changed = false;
	do {
		changed = false;
		for (int i = 0; i < 3; i++) {
			int cnt = 0;
			for (int j = 0; j < 3; j++) {
				cnt += arr[i][j] == inf;
			}
			if (cnt == 1) {
				if (arr[i][0] == inf) {
					arr[i][0] = arr[i][1] * 2 - arr[i][2];
				} else if (arr[i][1] == inf) {
					int d = arr[i][2] - arr[i][0];
					if (d % 2) {
						return false;
					}
					d /= 2;
					arr[i][1] = arr[i][0] + d;
				} else {
					arr[i][2] = arr[i][1] * 2 - arr[i][0];
				}
				changed = true;
			}
		}
		for (int i = 0; i < 3; i++) {
			int cnt = 0;
			for (int j = 0; j < 3; j++) {
				cnt += arr[j][i] == inf;
			}
			if (cnt == 1) {
				if (arr[0][i] == inf) {
					arr[0][i] = arr[1][i] * 2 - arr[2][i];
				} else if (arr[1][i] == inf) {
					int d = arr[2][i] - arr[0][i];
					if (d % 2) {
						return false;
					}
					d /= 2;
					arr[1][i] = arr[0][i] + d;
				} else {
					arr[2][i] = arr[1][i] * 2 - arr[0][i];
				}
				changed = true;
			}
		}
	} while (changed);
	return true;
}

void solve() {
	int arr[3][3];
	for (int i = 0; i < 3; i++) {
		for (int j = 0; j < 3; j++) {
			string s;
			cin >> s;
			if (s == "X") {
				arr[i][j] = inf;
			} else {
				arr[i][j] = stoi(s);
			}
		}
	}
	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	while (true) {
		int cur[3][3];
		memcpy(cur, arr, sizeof(cur));
		auto check = [&]() -> bool {
			for (auto& a : cur) {
				for (auto& b : a) {
					if (b == inf) {
						return false;
					}
				}
			}
			return true;
		};
		while (true) {
			if (!process(cur)) {
				goto loop;
			}
			vector<pair<int, int>> pos;
			for (int i = 0; i < 3; i++) {
				for (int j = 0; j < 3; j++) {
					if (cur[i][j] == inf) {
						pos.emplace_back(i, j);
					}
				}
			}
			if (!sz(pos)) {
				break;
			}
			auto [x, y] = pos[cowng() % sz(pos)];
			cur[x][y] = cowng() % 1000 - 500;
		}
		for (int i = 0; i < 3; i++) {
			for (int j = 0; j < 3; j++) {
				cout << cur[i][j] << " \n"[j == 2];
			}
		}
		return;
	loop:;
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