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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct Line {
	int x, l, r;
};

bool inters(int l1, int r1, int l2, int r2) {
	return max(l1, l2) <= min(r1, r2);
}

bool in_range(int x, int l, int r) {
	return l <= x && x <= r;
}

vector<Line> merge_same(const vector<Line>& arr) {
	vector<Line> cur;
	for (auto& ax : arr) {
		Line a = ax;
		vector<Line> nxt;
		for (auto& b : cur) {
			if (a.x == b.x && inters(a.l, a.r, b.l, b.r)) {
				a.l = min(a.l, b.l);
				a.r = max(a.r, b.r);
			} else {
				nxt.push_back(b);
			}
		}
		nxt.push_back(a);
		swap(cur, nxt);
	}
	return cur;
}

vector<Line> shorten(const vector<Line> &arr, int q) {
	vector<Line> ans;
	for (auto& a : arr) {
		for (int i = a.l; i <= a.r; i += q) {
			ans.push_back({a.x, i, min(a.r, i + q - 1)});
		}
	}
	return ans;
}

vector<pair<int, int>> merge_diff(const vector<Line>& x, const vector<Line>& y) {
	vector<pair<int, int>> ans;
	for (auto& a : x) {
		for (auto& b : y) {
			if (in_range(b.x, a.l, a.r) && in_range(a.x, b.l, b.r)) {
				ans.emplace_back(a.x, b.x);
			}
		}
	}
	return ans;
}

long count_inters(vector<Line> arr, const vector<pair<int, int>>& pts, int q) {
	assert(q);
	long ans = 0;
	for (auto& a : arr) {
		assert(a.r - a.l <= 2 * q);
		vector<pair<int, int>> cur;
		bool split = false;
		for (auto& [x, y] : pts) {
			dbg(x, a.x);
			if (abs(a.x - x) < q) {
				int l = y - q + 1, r = y + q - 1;
				if (a.l < l && r < a.r) {
					assert(a.l + 1 == l && r + 1 == a.r);
					split = true;
				}
				cur.emplace_back(l, r);
			}
		}
		if (split) {
			assert(a.l < a.r);
			bool bl = true, br = true;
			for (auto& [l, r] : cur) {
				if (in_range(a.l, l, r)) {
					bl = false;
				}
				if (in_range(a.r, l, r)) {
					br = false;
				}
			}
			ans += bl;
			ans += br;
		} else {
			for (auto& [l, r] : cur) {
				if (inters(a.l, a.r, l, r)) {
					if (l <= a.l && a.r <= r) {
						goto loop;
					} else if (l <= a.l) {
						a.l = r + 1;
					} else if (a.r <= r) {
						a.r = l - 1;
					} else {
						assert(false);
					}
				}
			}
			ans += a.r - a.l + 1;
		}
	loop:;
	}
	return ans;
}

vector<char> dist1(const vector<pair<int, int>> &pts, const vector<pair<int, int>> &arr, int q) {
	vector<char> ans(sz(pts));
	vector<array<int, 3>> evts;
	for (int i = 0; i < sz(pts); i++) {
		evts.push_back({pts[i].first, pts[i].second, i});
	}
	for (auto& [x, y] : arr) {
		evts.push_back({x, y, -1});
	}
	sort(begin(evts), end(evts), [&](const auto& a, const auto& b) -> bool {
		if (a[0] == b[0]) {
			return a[2] < b[2];
		}
		return a[0] < b[0];
	});
	multiset<int> vals;
	queue<pair<int, int>> cq;
	for (auto& [x, y, i] : evts) {
		dbg(x, y, i);
		while (sz(cq) && x - cq.front().first >= q) {
			vals.erase(vals.find(cq.front().second));
			cq.pop();
		}
		if (i >= 0) {
			auto it = vals.lower_bound(y);
			if (it != vals.end() && *it - y < q) {
				ans[i] = true;
			} else if (it != vals.begin() && y - *(--it) < q) {
				ans[i] = true;
			}
		} else {
			cq.emplace(x, y);
			vals.insert(y);
		}
	}
	return ans;
}

vector<char> dist(vector<pair<int, int>> pts, vector<pair<int, int>> arr, int q) {
	vector<char> ans = dist1(pts, arr, q);
	for (auto& [x, y] : pts) {
		x = -x;
	}
	for (auto& [x, y] : arr) {
		x = -x;
	}
	auto ans1 = dist1(pts, arr, q);
	for (int i = 0; i < sz(pts); i++) {
		ans[i] |= ans1[i];
	}
	return ans;
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	vector<pair<int, int>> arr(k);
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	int q;
	cin >> q;
	if (!q) {
		cout << k << endl;
		return;
	}
	vector<Line> hori, vert;
	auto add_hori = [&](int x, int l, int r) -> void {
		l = max(l, 1);
		r = min(r, m);
		if (1 <= x && x <= n) {
			hori.push_back({x, l, r});
		}
	};
	auto add_vert = [&](int x, int l, int r) -> void {
		l = max(l, 1);
		r = min(r, n);
		if (1 <= x && x <= m) {
			vert.push_back({x, l, r});
		}
	};
	for (auto& [x, y] : arr) {
		add_hori(x - q, y - q, y + q);
		add_hori(x + q, y - q, y + q);
		add_vert(y - q, x - q, x + q);
		add_vert(y + q, x - q, x + q);
	}
	hori = shorten(merge_same(hori), q);
	vert = shorten(merge_same(vert), q);
	auto inters = merge_diff(hori, vert);
	for (auto& a : hori) {
		dbg("H", a.x, a.l, a.r);
	}
	for (auto& a : vert) {
		dbg("V", a.x, a.l, a.r);
	}
	for (auto& [x, y] : inters) {
		dbg("I", x, y);
	}
	long ans = count_inters(hori, arr, q);
	for (auto& [x, y] : arr) {
		swap(x, y);
	}
	dbg(ans);
	ans += count_inters(vert, arr, q);
	for (auto& [x, y] : arr) {
		swap(x, y);
	}
	dbg(ans);
	auto bad = dist(inters, arr, q);
	ans -= count(begin(bad), end(bad), false);
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}