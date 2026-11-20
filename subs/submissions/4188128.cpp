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

const int maxn = 2e5 + 5, bsize = 400;

struct Query {
	int l, r, qind, lind;

	Query(int l, int r, int qind) : l(l), r(r), qind(qind), lind(l / bsize) {}

	bool operator<(const Query& q) const {
		if (lind == q.lind) {
			if (lind % 2 == 0) {
				return r < q.r;
			}
			return q.r < r;
		}
		return lind < q.lind;
	}
};

struct BIT {
	int csum = 0, v[maxn + 1] {};

	void upd(int ind, int x) {
		csum += x;
		ind++;
		while (ind <= maxn) {
			v[ind] += x;
			ind += ind & -ind;
		}
	}

	int psum(int ind) const {
		int ans = 0;
		while (ind > 0) {
			ans += v[ind];
			ind -= ind & -ind;
		}
		return ans;
	}

	int query(int ind) const {
		return csum - psum(ind);
	}
};

void solve() {
	int n, q;
	cin >> n >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int ans[q];
	vector<Query> queries;
	for (int i = 0; i < q; i++) {
		int l, r;
		cin >> l >> r;
		l--;
		queries.emplace_back(l, r, i);
	}
	sort(begin(queries), end(queries));
	BIT bit;
	int cl = 0, cr = 0;
	auto add = [&](int i) -> void {
		bit.upd(arr[i], 1);
	};
	auto rm = [&](int i) -> void {
		bit.upd(arr[i], -1);
	};
	auto inter = [&](int l1, int r1, int l2, int r2) {
		return max(l1, l2) < min(r1, r2);
	};
	auto solve = [&]() -> int {
		int l = 0, r = n;
		while (l < r) {
			int mid = (l + r + 1) / 2;
			if (bit.query(mid) >= mid) {
				l = mid;
			} else {
				r = mid - 1;
			}
		}
		return l;
	};
	for (auto& [l, r, qind, _] : queries) {
		if (inter(cl, cr, l, r)) {
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
		} else {
			for (int i = cl; i < cr; i++) {
				rm(i);
			}
			for (int i = l; i < r; i++) {
				add(i);
			}
		}
		tie(cl, cr) = tie(l, r);
		ans[qind] = solve();
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