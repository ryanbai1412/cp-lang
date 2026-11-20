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

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
	tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

const int maxn = 2e5 + 10;

struct Piece {
	int lvl;
	bool team;

	bool exists() {
		return !!lvl;
	}
} pieces[maxn];

template <typename T>
ost<T>& operator+=(ost<T>& a, const ost<T>& b) {
	for (auto& x : b) {
		a.insert(x);
	}
	return a;
}

struct Range {
	ost<pair<int, int>> data;

	void insert(int x, int y) {
		data.insert({x, y});
	}

	int count(int x, int l, int r) const {
		auto ans = data.order_of_key({x, r + 1}) - data.order_of_key({x, l});
		dbg(x, l, r, ans);
		return int(ans);
	}

	Range& operator+=(const Range& x) {
		data += x.data;
		return *this;
	}

	void clear() {
		data.clear();
	}
};

struct DS {
	int open;
	ost<pair<int, int>> teams[2];
	Range vert, hori;

	DS& operator+=(const DS& ds) {
		open += ds.open;
		teams[0] += ds.teams[0];
		teams[1] += ds.teams[1];
		vert += ds.vert;
		hori += ds.hori;
		return *this;
	}

	void clear() {
		open = 0;
		teams[0].clear();
		teams[1].clear();
		vert.clear();
		hori.clear();
	}
} dss[maxn];
int par[maxn];

void init() {
	fill(begin(par), end(par), -1);
}

int find(int u) {
	return par[u] < 0 ? u : (par[u] = find(par[u]));
}

void merge(int u, int v) {
	u = find(u);
	v = find(v);
	if (u == v) {
		return;
	}
	if (par[u] > par[v]) {
		swap(u, v);
	}
	par[u] += par[v];
	par[v] = u;
	dss[u] += dss[v];
	dss[v].clear();
}

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	string hori[n], vert[n - 1];
	for (int i = 0; i < n; i++) {
		cin >> hori[i];
	}
	for (int i = 0; i < n - 1; i++) {
		cin >> vert[i];
	}
	for (int i = 0; i < n * m; i++) {
		dss[i].clear();
	}
	memset(pieces, 0, sizeof(pieces));
	init();
	int queries[q][2];
	set<int> vpieces[m], hpieces[n];
	for (auto& [x, y] : queries) {
		int team, lvl;
		cin >> team >> lvl >> x >> y;
		x--;
		y--;
		int ind = x * m + y;
		dss[ind].open = 0;
		dss[ind].teams[team].insert({lvl, ind});
		hpieces[x].insert(y);
		vpieces[y].insert(x);
		pieces[ind] = {lvl, !!team};
	}
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			int ind = i * m + j;
			if (!pieces[ind].exists()) {
				dss[ind].open = 1;
				dss[ind].hori.insert(i, j);
				dss[ind].vert.insert(j, i);
			}
		}
	}
	auto up_eq = [&](int x, int y, char c) -> bool {
		return x && vert[x - 1][y] == c;
	};
	auto down_eq = [&](int x, int y, char c) -> bool {
		return x < n - 1 && vert[x][y] == c;
	};
	auto left_eq = [&](int x, int y, char c) -> bool {
		return y && hori[x][y - 1] == c;
	};
	auto right_eq = [&](int x, int y, char c) -> bool {
		return y < m - 1 && hori[x][y] == c;
	};
	int mleft[n][m], mright[n][m], mup[n][m], mdown[n][m];
	for (int i = 0; i < n; i++) {
		int ans = 1e9;
		for (int j = 0; j < m; j++) {
			mleft[i][j] = min(ans, j);
			if (left_eq(i, j, '1')) {
				mleft[i][j] = j - 1;
			}
			if (right_eq(i, j, '2')) {
				ans = min(ans, j);
			} else {
				ans = 1e9;
			}
		}
		ans = -1e9;
		for (int j = m - 1; j >= 0; j--) {
			mright[i][j] = max(ans, j);
			if (right_eq(i, j, '1')) {
				mright[i][j] = j + 1;
			}
			if (left_eq(i, j, '2')) {
				ans = max(ans, j);
			} else {
				ans = -1e9;
			}
		}
	}
	for (int i = 0; i < m; i++) {
		int ans = 1e9;
		for (int j = 0; j < n; j++) {
			mup[j][i] = min(ans, j);
			if (up_eq(j, i, '1')) {
				mup[j][i] = j - 1;
			}
			if (down_eq(j, i, '2')) {
				ans = min(ans, j);
			} else {
				ans = 1e9;
			}
		}
		ans = -1e9;
		for (int j = n - 1; j >= 0; j--) {
			mdown[j][i] = max(ans, j);
			if (down_eq(j, i, '1')) {
				mdown[j][i] = j + 1;
			}
			if (up_eq(j, i, '2')) {
				ans = max(ans, j);
			} else {
				ans = -1e9;
			}
		}
	}
	auto mmerge = [&](int ind1, int ind2) -> void {
		bool a = pieces[ind1].exists(), b = pieces[ind2].exists();
		if (!a && !b) {
			merge(ind1, ind2);
		} else {
			if (a) {
				dss[find(ind2)].teams[pieces[ind1].team].insert(
					{pieces[ind1].lvl, ind1});
			}
			if (b) {
				dss[find(ind1)].teams[pieces[ind2].team].insert(
					{pieces[ind2].lvl, ind2});
			}
		}
	};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m - 1; j++) {
			if (hori[i][j] == '3') {
				mmerge(i * m + j, i * m + j + 1);
			}
		}
	}
	for (int i = 0; i < n - 1; i++) {
		for (int j = 0; j < m; j++) {
			if (vert[i][j] == '3') {
				mmerge(i * m + j, (i + 1) * m + j);
			}
		}
	}
	auto floor_ceil = [&](const set<int>& s, int x) -> pair<int, int> {
		auto it = s.lower_bound(x);
		int floor = -1e9, ceil = 1e9;
		if (it != s.end()) {
			ceil = *it;
		}
		if (it != s.begin()) {
			floor = *(--it);
		}
		return {floor, ceil};
	};
	int ans[q];
	for (int i = q - 1; i >= 0; i--) {
		auto [x, y] = queries[i];
		int ind = x * m + y;
		auto [lvl, team] = pieces[ind];
		{
			pieces[ind] = {0, false};
			auto upd = [&](int cx, int cy) -> void {
				dbg(cx, cy);
				int nind = cx * m + cy;
				dss[find(nind)].teams[team].erase({lvl, ind});
				if (!pieces[nind].exists()) {
					merge(ind, nind);
				}
			};
			if (up_eq(x, y, '3')) {
				upd(x - 1, y);
			}
			if (down_eq(x, y, '3')) {
				upd(x + 1, y);
			}
			if (left_eq(x, y, '3')) {
				upd(x, y - 1);
			}
			if (right_eq(x, y, '3')) {
				upd(x, y + 1);
			}
		}
		auto& cds = dss[find(ind)];
		cds.teams[team].erase({lvl, ind});
		int cans = cds.open + cds.teams[team ^ 1].order_of_key({lvl + 1, 0});
		cds.vert.insert(y, x);
		cds.hori.insert(x, y);
		dbg("INIT", cans, cds.open);
		cds.open++;
		auto upd_piece = [&](int cx, int cy) -> void {
			if (0 <= cx && cx < n && 0 <= cy && cy < m) {
				int cind = cx * m + cy;
				if (pieces[cind].exists()) {
					auto [clvl, cteam] = pieces[cind];
					if (cteam != team && clvl <= lvl &&
						cds.teams[cteam].find({clvl, cind}) ==
							cds.teams[cteam].end()) {
						cans++;
					}
				}
			}
		};
		hpieces[x].erase(y);
		vpieces[y].erase(x);
		{
			auto [piecel, piecer] = floor_ceil(hpieces[x], y);
			int movel = mleft[x][y], mover = mright[x][y];
			dbg(piecel, piecer, movel, mover);
			if (piecel >= movel) {
				movel = piecel + 1;
				upd_piece(x, piecel);
			}
			if (piecer <= mover) {
				mover = piecer - 1;
				upd_piece(x, piecer);
			}
			cans += mover - movel + 1;
			cans -= cds.hori.count(x, movel, mover);
		}
		dbg("HORI", cans);
		{
			auto [pieceu, pieced] = floor_ceil(vpieces[y], x);
			int moveu = mup[x][y], moved = mdown[x][y];
			dbg(pieceu, pieced, moveu, moved);
			if (pieceu >= moveu) {
				moveu = pieceu + 1;
				upd_piece(pieceu, y);
			}
			if (pieced <= moved) {
				moved = pieced - 1;
				upd_piece(pieced, y);
			}
			cans += moved - moveu + 1;
			cans -= cds.vert.count(y, moveu, moved);
		}
		dbg("VERT", cans);
		ans[i] = cans;
	}
	for (auto& a : ans) {
		cout << a << endl;
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