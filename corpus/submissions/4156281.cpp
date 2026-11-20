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

struct Lazy {
	int set = -1, add;
	
	Lazy& operator+=(const Lazy &l) {
		if (l.set != -1) {
			set = l.set;
			add = l.add;
		} else {
			add += l.add;
		}
		return *this;
	}

	friend Lazy operator+(Lazy a, const Lazy &b) {
		return a += b;
	}
	
	void apply(int &x) const {
		if (set == -1) {
			x += add;
		} else {
			x = set + add;
		}
	}
};

struct ST {
	int n;
	vector<pair<int, int>> v;
	vector<Lazy> lazy;
	
	explicit ST(int _n): n(_n), v(4 * n), lazy(4 * n) {
		build(1, 0, n - 1);
	}
	
	void build(int o, int l, int r) {
		v[o] = {0, l};
		if (l < r) {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
		}
	}
	
	void maintain(int o, int l, int r) {
		if (l < r) {
			v[o] = min(v[o * 2], v[o * 2 + 1]);
		} else {
			v[o].first = 0;
		}
		lazy[o].apply(v[o].first);
	}
	
	void update(int o, int l, int r, int ql, int qr, const Lazy &x) {
		if (ql <= l && r <= qr) {
			lazy[o] += x;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			lazy[lc] += lazy[o];
			lazy[rc] += lazy[o];
			lazy[o] = Lazy();
			if (ql <= mid) {
				update(lc, l, mid, ql, qr, x);
			} else {
				maintain(lc, l, mid);
			}
			if (mid < qr) {
				update(rc, mid + 1, r, ql, qr, x);
			} else {
				maintain(rc, mid + 1, r);
			}
		}
		maintain(o, l, r);
	}
	
	pair<int, int> query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
		if (ql <= l && r <= qr) {
			auto cur = v[o];
			clazy.apply(cur.first);
			return cur;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			clazy += lazy[o];
			pair<int, int> ans {1e9, 1e9};
			if (ql <= mid) {
				ans = min(ans, query(lc, l, mid, ql, qr, clazy));
			}
			if (mid < qr) {
				ans = min(ans, query(rc, mid + 1, r, ql, qr, clazy));
			}
			return ans;
		}
	}
	
	void radd(int x) {
		update(1, 0, n - 1, 0, n - 1, {-1, x});
	}
	
	void rset(int x) {
		update(1, 0, n - 1, 0, n - 1, {x, 0});
	}
	
	void pset(int ind, int x) {
		update(1, 0, n - 1, ind, ind, {x, 0});
	}
	
	int pquery(int ind) const {
		return query(1, 0, n - 1, ind, ind, {-1, 0}).first;
	}
	
	pair<int, int> rmin() const {
		return query(1, 0, n - 1, 0, n - 1, {-1, 0});
	}
};

vector<pair<int, int>> decode(const vector<int> &arr) {
	int e = 0, ptr = 0;
	auto read = [&]() -> int {
		assert(ptr < sz(arr));
		return arr[ptr++];
	};
	vector<pair<int, int>> ans;
	while (ptr < sz(arr)) {
		int c = read();
		if (c == e) {
			c = read();
			if (c == e) {
				dbg("E READ", e);
				ans.emplace_back(e, read() + 1);
			} else {
				int k = read();
				if (!k) {
					dbg("ASSIGN", c);
					e = c;
				} else {
					dbg("REP", c);
					ans.emplace_back(c, k + 3);
				}
			}
		} else {
			dbg("NORM", c);
			ans.emplace_back(c, 1);
		}
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<int> in(m);
	for (auto& a : in) {
		cin >> a;
	}
	auto arr = decode(in);
	vector<pair<int, int>> narr;
	for (int i = 0; i < sz(arr); ) {
		int s = i, cnt = 0;
		for (; i < sz(arr) && arr[i].first == arr[s].first; i++) {
			cnt += arr[i].second;
		}
		narr.emplace_back(arr[s].first, cnt);
	}
	for (auto& [k, v] : narr) {
		dbg(k, v);
	}
	m = sz(narr);
	ST st(n);
	int trans[m][2];
	memset(trans, -1, sizeof(trans));
	auto cdiv = [&](int a, int b) -> int {
		return (a + b - 1) / b;
	};
	for (int i = m - 1; i >= 0; i--) {
		auto [value, length] = narr[i];
		int x = st.pquery(value);
		st.radd(3 * (length / (n + 2)) + min(3, length % (n + 2)));
		st.pset(value, 1e9);
		auto [opt, opt_ind] = st.rmin();
		x += 3 * cdiv(length, n);
		st.pset(value, x);
		if (x > opt + 3) {
			trans[i][1] = opt_ind;
			st.pset(value, opt + 3);
		}
		if (opt > x + 3) {
			trans[i][0] = value;
			st.rset(x + 3);
			st.pset(value, x);
		}
	}
	vector<int> ans;
	int e = 0;
	for (int i = 0; i < m; i++) {
		auto [value, length] = narr[i];
		int cind = e == value;
		if (trans[i][cind] != -1) {
			ans.push_back(e);
			e = trans[i][cind];
			dbg("SWITCHED", e);
			ans.push_back(e);
			ans.push_back(0);
		}
		if (e == value) {
			dbg("ENC MASK");
			while (length) {
				int csub = min(length, n);
				length -= csub;
				csub--;
				ans.push_back(e);
				ans.push_back(e);
				ans.push_back(csub);
			}
		} else {
			dbg("ENC OTHER");
			while (length > 3) {
				int csub = min(length, n + 2);
				length -= csub;
				csub -= 3;
				ans.push_back(e);
				ans.push_back(value);
				ans.push_back(csub);
			}
			for (int j = 0; j < length; j++) {
				ans.push_back(value);
			}
		}
	}
	cout << sz(ans) << endl;
	for (int i = 0; i < sz(ans); i++) {
		cout << ans[i] << " \n"[i == sz(ans) - 1];
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