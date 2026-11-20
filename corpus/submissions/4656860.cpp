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

//#define GEN

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());

int rint(int n) {
	return int(cowng() % n);
}

int rint(int l, int r) {
	return rint(r - l + 1) + l;
}

namespace d3 {
const int dx[] = {0, 0, 0, 0, -1, 1}, dy[] = {0, 0, -1, 1, 0, 0}, dz[] = {-1, 1, 0, 0, 0, 0};
}

namespace wrapper {

using namespace d3;

int n, m, k, q, queries;
vector<vector<vector<int>>> arrv, cache;

#ifdef GEN
void gen() {
	for (auto& a : arrv) {
		for (auto& b : a) {
			for (auto& c : b) {
				cin >> c;
			}
		}
	}
}
#endif

bool ibs(int x, int y, int z) {
	return 0 <= x && x < n && 0 <= y && y < m && 0 <= z && z < k;
}

void init(int _n, int _m, int _k, int _q) {
	tie(n, m, k, q) = tie(_n, _m, _k, _q);
	queries = 0;
	cache.resize(n, vector<vector<int>>(m, vector<int>(k, -1)));
#ifdef GEN
	arrv = cache;
	gen();
#endif
}

int query(int x, int y, int z) {
	int& ans = cache[x][y][z];
	if (ans != -1) {
		return ans;
	}
	queries++;
	assert(queries <= q);
#ifndef GEN
	cout << "? " << x + 1 << " " << y + 1 << " " << z + 1 << endl;
	cin >> ans;
#else
	ans = arrv[x][y][z];
#endif
	return ans;
};

void answer(int x, int y, int z) {
#ifdef GEN
	cout << "queries: " << queries << endl;
	for (int i = 0; i < 6; i++) {
		int cx = x + dx[i], cy = y + dy[i], cz = z + dz[i];
		if (ibs(cx, cy, cz)) {
			assert(arrv[x][y][z] >= arrv[cx][cy][cz]);
		}
	}
#endif
	cout << "! " << x + 1 << " " << y + 1 << " " << z + 1 << endl;
	exit(0);
};

}  // namespace wrapper

using wrapper::query, wrapper::answer, wrapper::ibs;

namespace s1 {

const double ra = (sqrt(5) - 1) / 2, rb = 1 - ra;

int query(int x) {
	if (ibs(x, 0, 0)) {
		return wrapper::query(x, 0, 0);
	}
	return 0;
}

void answer(int x) {
	wrapper::answer(x, 0, 0);
}

bool solve(int n, int m, int k, int q) {
	if (m != 1 || k != 1) {
		return false;
	}
	auto check = [&](int i) -> void {
		if (query(i) >= max(query(i - 1), query(i + 1))) {
			dbg(query(i), query(i - 1), query(i + 1));
			answer(i);
		}
	};
	int l = 0, r = n - 1, m1, m2;
	auto compute = [&]() -> void {
		m1 = int(l * ra + r * rb);
		m2 = int(l * rb + r * ra);
	};
	compute();
	while (m1 < m2) {
		dbg(l, r, m1, m2);
		int a = query(m1), b = query(m2);
		if (a < b) {
			l = m1;
			int tmp = m2;
			compute();
			m1 = tmp;
		} else {
			r = m2;
			int tmp = m1;
			compute();
			m2 = tmp;
		}
	}
	dbg(l, r);
	for (int i = l; i < r; i++) {
		check(i);
	}
	answer(r);
	return true;
}

}  // namespace s1

namespace s2 {

using namespace d3;

int query(int x, int y) {
	if (ibs(x, y, 0)) {
		return wrapper::query(x, y, 0);
	}
	return 0;
}

void answer(int x, int y) {
	wrapper::answer(x, y, 0);
}

void dfs(int x1, int y1, int x2, int y2, int px, int py, int pv) {
	if (x1 > x2 || y1 > y2) {
		return;
	}
	dbg(x1, y1, x2, y2);
	if (x2 - x1 <= y2 - y1) {
		int mid = (y1 + y2) / 2;
		pair<int, int> opt {-1, -1};
		for (int i = x1; i <= x2; i++) {
			opt = max(opt, pair<int, int> {query(i, mid), i});
		}
		int x = opt.second;
		if (pv >= opt.first) {
			if (py <= mid) {
				dfs(x1, y1, x2, mid - 1, px, py, pv);
			} else {
				dfs(x1, mid + 1, x2, y2, px, py, pv);
			}
		}
		int a = mid - 1 >= y1 ? query(x, mid - 1) : 0, b = query(x, mid),
			c = mid + 1 <= y2 ? query(x, mid + 1) : 0;
		if (b >= a && b >= c) {
			answer(x, mid);
		}
		if (a >= c) {
			dfs(x1, y1, x2, mid - 1, x, mid, opt.first);
		} else {
			dfs(x1, mid + 1, x2, y2, x, mid, opt.first);
		}
	} else {
		int mid = (x1 + x2) / 2;
		pair<int, int> opt {-1, -1};
		for (int i = y1; i <= y2; i++) {
			opt = max(opt, pair<int, int> {query(mid, i), i});
		}
		int x = opt.second;
		if (pv >= opt.first) {
			if (px <= mid) {
				dfs(x1, y1, mid - 1, y2, px, py, pv);
			} else {
				dfs(mid + 1, y1, x2, y2, px, py, pv);
			}
		}
		int a = mid - 1 >= x1 ? query(mid - 1, x) : 0, b = query(mid, x),
			c = mid + 1 <= x2 ? query(mid + 1, x) : 0;
		if (b >= a && b >= c) {
			answer(mid, x);
		}
		if (a >= c) {
			dfs(x1, y1, mid - 1, y2, mid, x, opt.first);
		} else {
			dfs(mid + 1, y1, x2, y2, mid, x, opt.first);
		}
	}
}

bool solve(int n, int m, int l, int) {
	if (l != 1) {
		return false;
	}
	dfs(0, 0, n - 1, m - 1, -1, -1, -1);
	assert(false);
	return true;
}

}  // namespace s2

namespace s3 {

using namespace d3;

bool solve(int n, int m, int l, int q) {
	if (q < int(1e5)) {
		return false;
	}
	array<int, 4> opt {};
	for (int i = 0; i < q / 2; i++) {
		int x = rint(n), y = rint(m), z = rint(l);
		opt = max(opt, {query(x, y, z), x, y, z});
	}
	auto [_, x, y, z] = opt;
	while (true) {
		for (int k = 0; k < 6; k++) {
			int cx = x + dx[k], cy = y + dy[k], cz = z + dz[k];
			if (ibs(cx, cy, cz) && query(cx, cy, cz) > query(x, y, z)) {
				tie(x, y, z) = tie(cx, cy, cz);
				goto loop;
			}
		}
		answer(x, y, z);
		return true;
	loop:;
	}
}

}  // namespace s3

void solve() {
	int n, m, k, q;
	cin >> n >> m >> k >> q;
	wrapper::init(n, m, k, q);
	s1::solve(n, m, k, q) || s3::solve(n, m, k, q) || s2::solve(n, m, k, q);
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}