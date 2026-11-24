// Reference for UCRPC F21 G at the statement's bounds: the AC submission
// corpus/submissions/3944717.cpp with its memoised recursion replaced by the same search run
// forwards. Every change:
// - dp(t, x, y, p) was memoised in a gp_hash_table keyed by (cell, second, popcorn held), one
//   entry per reachable state. At n = 100, p = b = 1000 with the drops crowded round the start
//   (valid: times 1..=100, no popcorn on a boulder's cell and second, no repeated boulder) that is
//   about 300 MB and 3 s against the 1 s, 256 MB limit. The same states are now kept as one
//   bitset of held counts per cell, advanced one second at a time: held + k is a shift and
//   floor(held / 2) a re-pack. The search stays exhaustive over held counts; nothing assumes
//   more popcorn is better.
// - The answer, the best held count over every state at the last second, is unchanged, as are
//   the event rules (popcorn at a cell and second adds one each, a boulder halves), the moves
//   (stay or a side neighbour inside the grid), and the input and output formats.
#include "bits/extc++.h"

using namespace std;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int dx[] = {0, 0, -1, 1}, dy[] = {-1, 1, 0, 0};
const int maxn = 105, maxt = 105, maxp = 1005;

using Held = bitset<maxp>;
using __gnu_pbds::gp_hash_table;

int n;
gp_hash_table<int, int> events[maxt];

bool ibs(int x, int y) {
	return 0 <= x && x < n && 0 <= y && y < n;
}

void solve() {
	int p, b, x, y;
	cin >> n >> p >> b >> x >> y;
	for (int i = 0; i < p; i++) {
		int x, y, t;
		cin >> x >> y >> t;
		events[t][x * maxn + y]++;
	}
	for (int i = 0; i < b; i++) {
		int x, y, t;
		cin >> x >> y >> t;
		events[t][x * maxn + y] = -1;
	}
	// at[x][y] = the counts p with a state (t, x, y, p) reached before t's drops.
	vector<vector<Held>> at(n, vector<Held>(n)), nxt(n, vector<Held>(n));
	at[x][y][0] = 1;
	for (int t = 0; t < maxt; t++) {
		for (auto& [lkey, change] : events[t]) {
			Held& h = at[lkey / maxn][lkey % maxn];
			if (change == -1) {
				Held half;
				for (size_t j = h._Find_first(); j < maxp; j = h._Find_next(j)) {
					half[j / 2] = 1;
				}
				h = half;
			} else {
				h <<= change;
			}
		}
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				Held h = at[i][j];
				for (int d = 0; d < 4; d++) {
					if (ibs(i + dx[d], j + dy[d])) {
						h |= at[i + dx[d]][j + dy[d]];
					}
				}
				nxt[i][j] = h;
			}
		}
		swap(at, nxt);
	}
	int ans = 0;
	for (auto& row : at) {
		for (auto& h : row) {
			for (size_t j = h._Find_first(); j < maxp; j = h._Find_next(j)) {
				ans = max(ans, int(j));
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
	solve();
}
