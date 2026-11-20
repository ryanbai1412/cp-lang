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

const int maxn = 1 << 20, maxq = 1 << 17;

struct ST {
	int v[maxn << 1] {};
	vector<vector<int>> rb;

	void update(int o, int l, int r, int ind, int x) {
		rb.back().push_back(v[o]);
		if (l == r) {
			v[o] = x;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ind <= mid) {
				update(lc, l, mid, ind, x);
			} else {
				update(rc, mid + 1, r, ind, x);
			}
			v[o] = max(v[lc], v[rc]);
		}
	}

	void update(int ind, int x) {
		rb.push_back({ind});
		update(1, 0, maxn - 1, ind, x);
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
			return ans;
		}
	}

	int query(int l, int r) const {
		if (l > r) {
			return 0;
		}
		return query(1, 0, maxn - 1, l, r);
	}

	int pop() {
		int x = rb.back().back();
		rb.back().pop_back();
		return x;
	}

	void rollback(int o, int l, int r, int ind) {
		v[o] = pop();
		if (l < r) {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ind <= mid) {
				rollback(lc, l, mid, ind);
			} else {
				rollback(rc, mid + 1, r, ind);
			}
		}
	}

	void rollback() {
		reverse(begin(rb.back()), end(rb.back()));
		rollback(1, 0, maxn - 1, pop());
		rb.pop_back();
	}
};

int n, k, q;

struct DS {
	ST st;
	int ans = 0;
	vector<int> rb;

	void update(int ind, int x) {
		rb.push_back(ans);
		st.update(ind, x);
		dbg(ind - k + 1, ind - 1, st.query(max(0, ind - k + 1), ind - 1));
		ans = max({ans, x + st.query(max(0, ind - k + 1), ind - 1),
				   x + st.query(ind + 1, min(maxn - 1, ind + k - 1))});
	}

	int query() const {
		return ans;
	}

	void rollback() {
		st.rollback();
		ans = rb.back();
		rb.pop_back();
	}
};

vector<pair<int, int>> qv[maxq << 1];

void update(int o, int l, int r, int ql, int qr, const pair<int, int> &x) {
	if (ql <= l && r <= qr) {
		qv[o].push_back(x);
	} else {
		int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
		if (ql <= mid) {
			update(lc, l, mid, ql, qr, x);
		}
		if (mid < qr) {
			update(rc, mid + 1, r, ql, qr, x);
		}
	}
}

void update(int l, int r, const pair<int, int> &x) {
	update(1, 0, q, l, r, x);
}

int ans[maxq];
DS ds;

void solve(int o, int l, int r) {
	for (auto& [ind, x] : qv[o]) {
		ds.update(ind, x);
		dbg("+", ind, x);
	}
	if (l == r) {
		dbg("Q", l, ds.query());
		ans[l] = ds.query();
	} else {
		int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
		solve(lc, l, mid);
		solve(rc, mid + 1, r);
	}
	for (auto& _ : qv[o]) {
		ds.rollback();
		dbg("-");
	}
}

void solve() {
	{
		k = 2;
		DS ds;
		ds.update(0, 1);
		ds.update(1, 9);
		ds.update(2, 3);
		dbg(ds.query());
	}
	cin >> n >> k >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int t[n] {};
	for (int i = 1; i <= q; i++) {
		int ind, x;
		cin >> ind >> x;
		ind--;
		update(t[ind], i - 1, {ind, arr[ind]});
		t[ind] = i;
		arr[ind] = x;
	}
	for (int i = 0; i < n; i++) {
		update(t[i], q, {i, arr[i]});
	}
	solve(1, 0, q);
	for (int i = 0; i <= q; i++) {
		cout << ans[i] << endl;
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}