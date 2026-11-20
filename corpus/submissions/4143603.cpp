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

struct LST {
	int n;
	vector<int> v, lazy;

	explicit LST(int n) : n(n), v(4 * n), lazy(4 * n) {}

	void update(int o, int l, int r, int ql, int qr, int x) {
		if (ql <= l && r <= qr) {
			lazy[o] += x;
			v[o] += x;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ql <= mid) {
				update(lc, l, mid, ql, qr, x);
			}
			if (mid < qr) {
				update(rc, mid + 1, r, ql, qr, x);
			}
			v[o] = lazy[o] + max(v[lc], v[rc]);
		}
	}

	void update(int l, int r, int x) {
		update(1, 0, n - 1, l, r, x);
	}

	int query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 0;
			if (ql <= mid) {
				ans = max(ans, query(lc, l, mid, ql, qr));
			}
			if (mid < qr) {
				ans = max(ans, query(rc, mid + 1, r, ql, qr));
			}
			return ans + lazy[o];
		}
	}

	int query(int l, int r) const {
		return query(1, 0, n - 1, l, r);
	}
	
	int query_all() const {
		return v[1];
	}
};

const int bsize = 400, maxn = 2e5;

struct Event {
	int l, r, ans_ind, l_ind;

	Event(int _l, int _r, int _ans_ind)
		: l(_l),
		  r(_r),
		  ans_ind(_ans_ind),
		  l_ind((l + bsize - 1) / bsize) {}

	bool operator<(const Event& e) const {
		if (l_ind == e.l_ind) {
			if (l_ind & 1) {
				return e.r < r;
			}
			return r < e.r;
		} else {
			return l_ind < e.l_ind;
		}
	}
};

LST st(maxn * 2);

void solve() {
	int n;
	cin >> n;
	int arr[n][2];
	for (auto& [l, r] : arr) {
		cin >> l >> r;
	}
	vector<int> comp;
	for (auto& [l, r] : arr) {
		comp.push_back(l);
		comp.push_back(r);
	}
	sort(begin(comp), end(comp));
	comp.erase(unique(begin(comp), end(comp)), comp.end());
	for (auto& [l, r] : arr) {
		l = int(lower_bound(begin(comp), end(comp), l) - comp.begin());
		r = int(lower_bound(begin(comp), end(comp), r) - comp.begin());
	}
	int q;
	cin >> q;
	vector<Event> evts;
	for (int i = 0; i < q; i++) {
		int l, r;
		cin >> l >> r;
		l--;
		r--;
		evts.emplace_back(l, r, i);
	}
	sort(begin(evts), end(evts));
	int ans[q];
	auto add = [&](int i) -> void {
		dbg("ADD", i);
		st.update(arr[i][0], arr[i][1], 1);
	};
	auto rm = [&](int i) -> void {
		dbg("RM", i);
		st.update(arr[i][0], arr[i][1], -1);
	};
	auto isects = [&](int l1, int r1, int l2, int r2) -> bool {
		return max(l1, l2) <= min(r1, r2);
	};
	int cl = 0, cr = -1;
	for (auto& [nl, nr, ans_ind, _] : evts) {
		if (isects(cl, cr, nl, nr)) {
			dbg("type 1");
			for (int i = cl; i < nl; i++) {
				rm(i);
			}
			for (int i = nl; i < cl; i++) {
				add(i);
			}
			for (int i = cr + 1; i <= nr; i++) {
				add(i);
			}
			for (int i = nr + 1; i <= cr; i++) {
				rm(i);
			}
		} else {
			dbg("type 2");
			for (int i = cl; i <= cr; i++) {
				rm(i);
			}
			for (int i = nl; i <= nr; i++) {
				add(i);
			}
		}
		tie(cl, cr) = tie(nl, nr);
		dbg("QUERY", ans_ind);
		ans[ans_ind] = st.query_all();
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