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

namespace s1 {

void solve(int n, int m, const vector<int> &queries) {
	int perm[2][4];
	for (auto& a : perm) {
		iota(begin(a), end(a), 0);
	}
	for (auto& a : queries) {
		if (a == 1) {
			for (auto& b : perm) {
				swap(b[0], b[1]);
				swap(b[2], b[3]);
			}
		} else if (a == 3) {
			for (auto& b : perm) {
				swap(b[0], b[2]);
				swap(b[1], b[3]);
			}
		} else if (a == 5) {
			int prev[2][4];
			memcpy(prev, perm, sizeof(prev));
			perm[0][0] = prev[0][2];
			perm[0][1] = prev[0][0];
			perm[0][2] = prev[0][3];
			perm[0][3] = prev[0][1];
			perm[1][0] = prev[1][1];
			perm[1][1] = prev[1][3];
			perm[1][2] = prev[1][0];
			perm[1][3] = prev[1][2];
		}
	}
	int ans[2 * n][2 * m];
	for (int i = 0; i < 2 * n; i++) {
		for (int j = 0; j < 2 * m; j++) {
			ans[i][j] = i * 2 * m + j + 1;
		}
	}
	for (int i = 0; i < 2 * n; i += 2) {
		for (int j = 0; j < 2 * m; j += 2) {
			int cur[4] {ans[i][j], ans[i][j + 1], ans[i + 1][j], ans[i + 1][j + 1]};
			int cind = ((i / 2) ^ (j / 2)) & 1;
			ans[i][j] = cur[perm[cind][0]];
			ans[i][j + 1] = cur[perm[cind][1]];
			ans[i + 1][j] = cur[perm[cind][2]];
			ans[i + 1][j + 1] = cur[perm[cind][3]];
		}
	}
	for (int i = 0; i < 2 * n; i++) {
		for (int j = 0; j < 2 * m; j++) {
			cout << ans[i][j] << " \n"[j == 2 * m - 1];
		}
	}
}

}

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	vector<int> queries(q);
	for (auto& a : queries) {
		cin >> a;
	}
	bool occur[6] {};
	for (auto& a : queries) {
		occur[a] = true;
	}
	if (!occur[2] && !occur[4]) {
		s1::solve(n, m, queries);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}