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

template <typename Node_T, typename Operation = plus<Node_T>, bool debug = true>
class SegTree {
  private:
	static constexpr Node_T noT {};

  public:
	const int n;
	const Operation operation;

	vector<Node_T> v, arr;

	explicit SegTree(const vector<Node_T>& _arr,
					 const Operation& _operation = Operation())
		: n(sz(_arr)), operation(_operation), v(4 * n), arr(_arr) {
		if (n) {
			build(1, 0, n - 1);
		}
	}

  private:
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

	template <typename F>
	void set_func(int o, int l, int r, int ind, const F& f) {
		if (l == r) {
			f(arr[l]);
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			if (ind <= mid) {
				set_func(o * 2, l, mid, ind, f);
			} else {
				set_func(o * 2 + 1, mid + 1, r, ind, f);
			}
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	[[nodiscard]] Node_T query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2;
			if (ql <= mid) {
				if (mid < qr) {
					return operation(query(o * 2, l, mid, ql, qr),
									 query(o * 2 + 1, mid + 1, r, ql, qr));
				}
				return query(o * 2, l, mid, ql, qr);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr);
		}
	}

	template <typename Predicate, bool first>
	[[nodiscard]] int bsearch(int o,
							  int l,
							  int r,
							  const Node_T& pref,
							  const Predicate& predicate) const {
		if (l == r) {
			return l;
		} else {
			int mid = (l + r) / 2;
			Node_T npref;
			if (first) {
				npref = operation(pref, v[o * 2]);
			} else {
				npref = v[o * 2];
			}
			if (predicate(npref)) {
				return bsearch<Predicate, first>(o * 2, l, mid, pref,
												 predicate);
			}
			return bsearch<Predicate, true>(o * 2 + 1, mid + 1, r, npref,
											predicate);
		}
	}

	template <typename Predicate>
	[[nodiscard]] pair<int, Node_T> bsearchl(int o,
											 int l,
											 int r,
											 int ql,
											 const Node_T& pref,
											 const Predicate& predicate) const {
		if (ql <= l) {
			Node_T npref;
			if (ql < l) {
				npref = operation(pref, v[o]);
			} else {
				npref = v[o];
			}
			if (!predicate(npref)) {
				return {r + 1, npref};
			}
			if (ql < l) {
				return {bsearch<Predicate, true>(o, l, r, pref, predicate),
						npref};
			} else {
				return {bsearch<Predicate, false>(o, l, r, pref, predicate),
						npref};
			}
		} else {
			int mid = (l + r) / 2;
			if (ql <= mid) {
				auto [ans, npref] =
					bsearchl(o * 2, l, mid, ql, pref, predicate);
				if (ans <= mid) {
					return {ans, operation(npref, v[o * 2 + 1])};
				}
				return bsearchl(o * 2 + 1, mid + 1, r, ql, npref, predicate);
			}
			return bsearchl(o * 2 + 1, mid + 1, r, ql, pref, predicate);
		}
	}

  public:
	[[nodiscard]] Node_T query(int l, int r) const {
		if (debug) {
			assert(0 <= l);
			assert(l < n);
			assert(0 <= r);
			assert(r < n);
			assert(l <= r);
		}
		return query(1, 0, n - 1, l, r);
	}

	void set(int ind, const Node_T& x) {
		if (debug) {
			assert(0 <= ind);
			assert(ind < n);
		}
		set_func(1, 0, n - 1, ind, [&](Node_T& t) { t = x; });
	}
};

void solve() {
	int n, q;
	cin >> n >> q;
	vector<long> av(n), bv(n);
	for (int i = 0; i < n; i++) {
		int x;
		cin >> x;
		if (i & 1) {
			av[i] = x;
		} else {
			bv[i] = x;
		}
	}
	SegTree<long> a(av), b(bv);
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			int ind, x;
			cin >> ind >> x;
			ind--;
			if (ind & 1) {
				a.set(ind, x);
			} else {
				b.set(ind, x);
			}
		} else {
			int l, r;
			cin >> l >> r;
			l--;
			r--;
			if (l & 1) {
				cout << a.query(l, r) << endl;
			} else {
				cout << b.query(l, r) << endl;
			}
		}
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