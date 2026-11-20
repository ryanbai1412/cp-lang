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
	vector<int> base;
	vector<long> v;

	explicit BDS(const vector<int>& base) : base(base), v(sz(base)) {}

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

	long& operator[](int ind) {
		return v[ind];
	}

	const long& operator[](int ind) const {
		return v[ind];
	}
	
	template<typename T>
	int bsearchl(int r, const T &t) {
		int l = 0;
		while (l < r) {
			int mid = (l + r) / 2;
			if (t(mid)) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return l;
	}
	
	template<typename T>
	int bsearchr(int l, const T &t) {
		int r = sz(v);
		while (l < r) {
			int mid = (l + r) / 2;
			if (t(mid)) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return l;
	}
};

vector<int> comp(vector<int>& arr) {
	vector<int> ccomp;
	for (auto& a : arr) {
		ccomp.push_back(a);
	}
	sort(begin(ccomp), end(ccomp));
	ccomp.erase(unique(begin(ccomp), end(ccomp)), ccomp.end());
	for (auto& a : arr) {
		a = int(lower_bound(begin(ccomp), end(ccomp), a) - ccomp.begin());
	}
	return ccomp;
}

void solve() {
	int n;
	cin >> n;
	vector<int> ixs(n + 1), iys(n + 1);
	for (int i = 1; i <= n; i++) {
		cin >> ixs[i] >> iys[i];
	}
	auto xs = ixs;
	auto ys = iys;
	auto xcomp = comp(xs);
	auto ycomp = comp(ys);
	auto distx = [&](int i, int j) -> int { return abs(ixs[i] - ixs[j]); };
	auto disty = [&](int i, int j) -> int { return abs(iys[i] - iys[j]); };
	BDS dpx(ycomp), dpy(xcomp);
	for (int ci = n - 1; ci >= 0; ci--) {
		long dx = distx(ci, ci + 1), dy = disty(ci, ci + 1);
		long ax = dpx[ys[ci]], ay = dpy[xs[ci]];
		{
			long ca = dx, cb = iys[ci + 1] + ay;
			int ind = dpx.bsearchl(ys[ci + 1], [&](int i) -> bool {
				return cb - ycomp[i] <= ca + dpx[i];
			});
			dpx.add(0, ind, ca);
			dpx.set<true>(ind, ys[ci + 1], cb);
		}
		{
			long ca = dx, cb = -iys[ci + 1] + ay;
			int ind = dpx.bsearchr(ys[ci + 1], [&](int i) -> bool {
				return ca + dpx[i] <= cb + i;
			});
			dpx.set<false>(ys[ci + 1], ind, cb);
			dpx.add(ind, sz(ycomp), ca);
		}
		{
			long ca = dy, cb = ixs[ci + 1] + ax;
			for (int i = 0; i < xs[ci + 1]; i++) {
				dpy[i] = min(ca + dpy[i], cb - xcomp[i]);
			}
		}
		{
			long ca = dy, cb = -ixs[ci + 1] + ax;
			for (int i = xs[ci + 1]; i < sz(xcomp); i++) {
				dpy[i] = min(ca + dpy[i], cb + xcomp[i]);
			}
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