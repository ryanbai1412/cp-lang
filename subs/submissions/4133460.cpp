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

template <typename T,
		  typename Lazy_T,
		  typename Operation = plus<T>,
		  bool associativeLazy = false>
struct LazySegTree {
	using Lazy = pair<bool, Lazy_T>;
	
	static constexpr T noT {};
	static constexpr Lazy noLazy {false, Lazy_T()};
	
	const int n;
	const Operation operation;

	vector<T> v, arr;
	vector<Lazy> lazy;

	explicit LazySegTree(const vector<T>& arr, const Operation& operation = Operation())
		: n(sz(arr)),
		  operation(operation),
		  v(4 * n),
		  arr(arr),
		  lazy(4 * n, noLazy) {
		build(1, 0, n - 1);
	}

	void build(int o, int l, int r) {
		if (l == r) {
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	inline T applyLazy(const T& x, const Lazy& clazy, int l, int r) const {
		if (clazy.first) {
			return clazy.second.apply(x, l, r);
		}
		return x;
	}

	inline void maintain(int o, int l, int r) {
		if (l == r) {
			v[o] = arr[l];
		} else {
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
		v[o] = applyLazy(v[o], lazy[o], l, r);
	}

	inline void mergeLazy(Lazy& first, const Lazy_T& second) const {
		if (first.first) {
			first.second = first.second + second;
		} else {
			first.first = true;
			first.second = second;
		}
	}

	inline void mergeClazy(Lazy& clazy, const Lazy& first) const {
		if (first.first) {
			if (clazy.first) {
				clazy.second = first.second + clazy.second;
			} else {
				clazy = first;
			}
		}
	}

	inline T mergePrefix(bool first, const T& pref, const T& next) const {
		if (first) {
			return next;
		}
		return operation(pref, next);
	}

	void set(int o, int l, int r, int ind, const T& x) {
		if (l == r) {
			v[o] = arr[l] = x;
			lazy[o].first = false;
		} else {
			int mid = (l + r) / 2;
			if (!associativeLazy && lazy[o].first) {
				mergeLazy(lazy[o * 2], lazy[o].second);
				mergeLazy(lazy[o * 2 + 1], lazy[o].second);
				lazy[o].first = false;
				if (ind <= mid) {
					set(o * 2, l, mid, ind, x);
					maintain(o * 2 + 1, mid + 1, r);
				} else {
					set(o * 2 + 1, mid + 1, r, ind, x);
					maintain(o * 2, l, mid);
				}
			} else {
				if (ind <= mid) {
					set(o * 2, l, mid, ind, x);
				} else {
					set(o * 2 + 1, mid + 1, r, ind, x);
				}
			}
			maintain(o, l, r);
		}
	}

	inline void set(int ind, const T& t) {
		set(1, 0, n - 1, ind, t);
	}

	T get(int o, int l, int r, int ind, Lazy clazy) const {
		if (l == r) {
			return applyLazy(v[o], clazy, l, r);
		} else {
			int mid = (l + r) / 2;
			mergeClazy(clazy, lazy[o]);
			if (ind <= mid) {
				return get(o * 2, l, mid, ind, clazy);
			} else {
				return get(o * 2 + 1, mid + 1, r, ind, clazy);
			}
		}
	}

	inline T get(int ind) const {
		return get(1, 0, n - 1, ind, noLazy);
	}

	void update(int o, int l, int r, int ql, int qr, const Lazy_T& x) {
		if (ql <= l && r <= qr) {
			mergeLazy(lazy[o], x);
		} else {
			int mid = (l + r) / 2;
			if (!associativeLazy && lazy[o].first) {
				mergeLazy(lazy[o * 2], lazy[o].second);
				mergeLazy(lazy[o * 2 + 1], lazy[o].second);
				lazy[o].first = false;
				if (ql <= mid) {
					update(o * 2, l, mid, ql, qr, x);
				} else {
					maintain(o * 2, l, mid);
				}
				if (mid < qr) {
					update(o * 2 + 1, mid + 1, r, ql, qr, x);
				} else {
					maintain(o * 2 + 1, mid + 1, r);
				}
			} else {
				if (ql <= mid) {
					update(o * 2, l, mid, ql, qr, x);
				}
				if (mid < qr) {
					update(o * 2 + 1, mid + 1, r, ql, qr, x);
				}
			}
		}
		maintain(o, l, r);
	}

	inline void update(int l, int r, const Lazy_T& x) {
		update(1, 0, n - 1, l, r, x);
	}

	T query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
		if (ql <= l && r <= qr) {
			return applyLazy(v[o], clazy, l, r);
		} else {
			int mid = (l + r) / 2;
			mergeClazy(clazy, lazy[o]);
			if (ql <= mid) {
				if (mid < qr) {
					return operation(
						query(o * 2, l, mid, ql, qr, clazy),
						query(o * 2 + 1, mid + 1, r, ql, qr, clazy));
				}
				return query(o * 2, l, mid, ql, qr, clazy);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr, clazy);
		}
	}

	inline T query(int l, int r) const {
		return query(1, 0, n - 1, l, r, noLazy);
	}
};

struct Op {
	long operator()(long a, long b) const {
		return max(a, b);
	}
};

struct Lazy {
	long add;
	
	Lazy operator+(const Lazy &l) const {
		return {add + l.add};
	}
	
	long apply(long x, int, int) const {
		return x + add;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	vector<pair<int, long>> arr[n];
	for (int i = 0; i < m; i++) {
		int l, r, x;
		cin >> l >> r >> x;
		l--;
		r--;
		arr[l].emplace_back(r, x);
	}
	LazySegTree<long, Lazy, Op> st((vector<long> (n + 1)));
	for (int i = n - 1; i >= 0; i--) {
		for (auto& [e, w] : arr[i]) {
			st.update(i + 1, e + 1, {w});
		}
		long cur = max(long(0), st.query(i + 1, n));
		st.set(i, cur);
		dbg(i, cur);
	}
	cout << st.query(0, n - 1) << endl;
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