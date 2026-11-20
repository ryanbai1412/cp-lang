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

const int maxn = 5e5 + 5, bsize = 400, bcnt = maxn / bsize + 1;

struct Query {
	int l, r, qind, lind;

	Query(int l, int r, int qind) : l(l), r(r), qind(qind), lind(l / bsize) {}

	bool operator<(const Query& e) const {
		if (lind == e.lind) {
			if (lind & 1) {
				return r > e.r;
			} else {
				return r < e.r;
			}
		}
		return lind < e.lind;
	}
};

void solve() {
	int n, q;
	cin >> n >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	vector<Query> queries;
	for (int i = 0; i < q; i++) {
		int l, r;
		cin >> l >> r;
		l--;
		queries.emplace_back(l, r, i);
	}
	sort(begin(queries), end(queries));
	long ans[q];
	int cl = 0, cr = 0;
	long cans = 0;
	set<pair<int, int>> vals;
	auto add = [&](int i) -> void {
		auto [it, _] = vals.emplace(arr[i], i);
		if (it == vals.begin()) {
			if (sz(vals) >= 2) {
				cans += abs((++it)->second - i);
			}
		} else if (it == (--vals.end())) {
			if (sz(vals) >= 2) {
				cans += abs((--it)->second - i);
			}
		} else {
			int a = prev(it)->second, b = (++it)->second;
			cans -= abs(a - b);
			cans += abs(i - a) + abs(b - i);
		}
	};
	auto rm = [&](int i) -> void {
		auto it = vals.erase(vals.find({arr[i], i}));
		if (!sz(vals)) {
			return;
		}
		if (it == vals.begin()) {
			cans -= abs(it->second - i);
		} else if (it == vals.end()) {
			cans -= abs((--it)->second - i);
		} else {
			int a = it->second, b = (--it)->second;
			cans -= abs(i - a) + abs(i - b);
			cans += abs(a - b);
		}
	};
	for (auto& [l, r, ind, _] : queries) {
		if (min(cr, r) <= max(cl, l)) {
			for (int i = cl; i < cr; i++) {
				rm(i);
			}
			for (int i = l; i < r; i++) {
				add(i);
			}
		} else {
			for (int i = cl; i < l; i++) {
				rm(i);
			}
			for (int i = l; i < cl; i++) {
				add(i);
			}
			for (int i = r; i < cr; i++) {
				rm(i);
			}
			for (int i = cr; i < r; i++) {
				add(i);
			}
		}
		ans[ind] = cans;
		tie(cl, cr) = tie(l, r);
	}
	for (auto& a : ans) {
		cout << a << endl;
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