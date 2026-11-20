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

struct BDS {
	vector<long> base, v;

	explicit BDS(const vector<long>& base) : base(base), v(sz(base)) {}

	void add(int l, int r, long x) {
		for (int i = l; i < r; i++) {
			v[i] += x;
		}
	}

	template <bool neg>
	void set(int l, int r, long x) {
		for (int i = l; i < r; i++) {
			v[i] = x;
			if (neg) {
				v[i] -= base[i];
			} else {
				v[i] += base[i];
			}
		}
	}

	const long& operator[](int ind) const {
		return v[ind];
	}

	template <typename T>
	int bsearchl(int r, const T& t) {
		int l = 0;
		while (l < r) {
			int mid = (l + r) / 2;
			if (t(mid, (*this)[mid])) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return l;
	}

	template <typename T>
	int bsearchr(int l, const T& t) {
		int r = sz(v);
		while (l < r) {
			int mid = (l + r) / 2;
			if (t(mid, (*this)[mid])) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return l;
	}
};

const long nadd = -4e18;

struct Lazy {
	long add;
	long badd;
	bool bneg;

	Lazy& operator+=(const Lazy& l) {
		if (l.badd != nadd) {
			return *this = l;
		}
		add += l.add;
		return *this;
	}

	Lazy friend operator+(Lazy a, const Lazy& b) {
		return a += b;
	}

	long apply(long base) const {
		long ans = add;
		if (badd != nadd) {
			ans += badd;
			if (bneg) {
				ans -= base;
			} else {
				ans += base;
			}
		}
		return ans;
	}

	Lazy static no_lazy() {
		return Lazy {0, nadd, false};
	}
};

struct DS {
	static int gsz(int n) {
		int clog = 1 + __lg(n - 1);
		return 1 << (clog + 1);
	}

	int n;
	vector<long> base;
	vector<Lazy> lazy;

	explicit DS(const vector<long>& base)
		: n(sz(base)), base(base), lazy(gsz(n), Lazy::no_lazy()) {}

	long get(int o, int l, int r, int ind, Lazy clazy) const {
		clazy = lazy[o] + clazy;
		if (l == r) {
			return clazy.apply(base[ind]);
		} else {
			int mid = (l + r) / 2;
			if (ind <= mid) {
				return get(o * 2, l, mid, ind, clazy);
			} else {
				return get(o * 2 + 1, mid + 1, r, ind, clazy);
			}
		}
	}

	long operator[](int ind) const {
		return get(1, 0, n - 1, ind, Lazy::no_lazy());
	}

	void update(int o, int l, int r, int ql, int qr, const Lazy& x) {
		if (ql <= l && r <= qr) {
			lazy[o] += x;
		} else {
			int mid = (l + r) / 2;
			lazy[o * 2] += lazy[o];
			lazy[o * 2 + 1] += lazy[o];
			lazy[o] = Lazy::no_lazy();
			if (ql <= mid) {
				update(o * 2, l, mid, ql, qr, x);
			}
			if (mid < qr) {
				update(o * 2 + 1, mid + 1, r, ql, qr, x);
			}
		}
	}

	void add(int l, int r, long x) {
		if (l < r) {
			update(1, 0, n - 1, l, r - 1, {x, nadd, false});
		}
	}

	template <bool neg>
	void set(int l, int r, long x) {
		if (l < r) {
			update(1, 0, n - 1, l, r - 1, {0, x, neg});
		}
	}

	template <typename T>
	int bsearchl(int r, const T& t) {
		int l = 0;
		while (l < r) {
			int mid = (l + r) / 2;
			if (t(mid, (*this)[mid])) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return l;
	}

	template <typename T>
	int bsearchr(int l, const T& t) {
		int r = n;
		while (l < r) {
			int mid = (l + r) / 2;
			if (t(mid, (*this)[mid])) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return l;
	}
};

template <typename T>
vector<T> comp(vector<T>& arr) {
	vector<T> ccomp;
	for (auto& a : arr) {
		ccomp.push_back(a);
	}
	sort(begin(ccomp), end(ccomp));
	ccomp.erase(unique(begin(ccomp), end(ccomp)), ccomp.end());
	for (auto& a : arr) {
		a = T(lower_bound(begin(ccomp), end(ccomp), a) - ccomp.begin());
	}
	return ccomp;
}

void solve() {
	int n;
	cin >> n;
	vector<long> ixs(n + 1), iys(n + 1);
	for (int i = 1; i <= n; i++) {
		cin >> ixs[i] >> iys[i];
	}
	auto xs = ixs;
	auto ys = iys;
	auto xcomp = comp(xs);
	auto ycomp = comp(ys);
	auto distx = [&](int i, int j) -> int {
		return abs(ixs[i] - ixs[j]);
	};
	auto disty = [&](int i, int j) -> int {
		return abs(iys[i] - iys[j]);
	};
	DS dpx(ycomp), dpy(xcomp);
	for (int ci = n - 1; ci >= 0; ci--) {
		long dx = distx(ci, ci + 1), dy = disty(ci, ci + 1);
		long ax = dpx[ys[ci]], ay = dpy[xs[ci]];
		{
			long ca = dx, cb = iys[ci + 1] + ay;
			int ind = dpx.bsearchl(ys[ci + 1], [&](int i, long x) -> bool {
				return cb - ycomp[i] <= ca + x;
			});
			dpx.add(0, ind, ca);
			dpx.set<true>(ind, ys[ci + 1], cb);
		}
		{
			long ca = dx, cb = -iys[ci + 1] + ay;
			int ind = dpx.bsearchr(ys[ci + 1], [&](int i, long x) -> bool {
				return ca + x <= cb + ycomp[i];
			});
			dpx.set<false>(ys[ci + 1], ind, cb);
			dpx.add(ind, sz(ycomp), ca);
		}
		{
			long ca = dy, cb = ixs[ci + 1] + ax;
			int ind = dpy.bsearchl(xs[ci + 1], [&](int i, long x) -> bool {
				return cb - xcomp[i] <= ca + x;
			});
			dpy.add(0, ind, ca);
			dpy.set<true>(ind, xs[ci + 1], cb);
		}
		{
			long ca = dy, cb = -ixs[ci + 1] + ax;
			int ind = dpy.bsearchr(xs[ci + 1], [&](int i, long x) -> bool {
				return ca + x <= cb + xcomp[i];
			});
			dpy.set<false>(xs[ci + 1], ind, cb);
			dpy.add(ind, sz(xcomp), ca);
		}
	}
	cout << dpx[ys[0]] << endl;
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