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
#define dbg(...)
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

	/**
	 * applies the function f to arr[ind]
	 * f should be of signature (Node_T&) -> void
	 */
	template <typename F>
	void set_func(int ind, const F& f) {
		if (debug) {
			assert(0 <= ind);
			assert(ind < n);
		}
		set_func(1, 0, n - 1, ind, f);
	}

	/**
	 * computes the first index i such that predicate(query(0, i)) returns true
	 * or n, if no such index exists
	 */
	template <typename Predicate>
	[[nodiscard]] int bsearch(const Predicate& predicate) const {
		if (!predicate(v[1])) {
			return n;
		}
		return bsearch<Predicate, true>(1, 0, n - 1, noT, predicate);
	}

	/**
	 * computes the first index i (i >= l) such that predicate(query(l, i))
	 * returns true or n, if no such index exists
	 */
	template <typename Predicate>
	[[nodiscard]] int bsearchl(int l, const Predicate& predicate) const {
		return bsearchl(1, 0, n - 1, l, noT, predicate).first;
	}
};

const long NONE = -1e18;

struct Func {
	long s, v, a;
	
	long operator()(long x) const {
		return s + v * x + a * x * (x - 1) / 2;
	}
	
	bool operator==(const Func &f) const {
		return tie(s, v, a) == tie(f.s, f.v, f.a);
	}
};

struct Node {
	bool bad;
	Func f;
	long x, y;
	
	static Node from(long x, long y) {
		return {false, {0, 0, 0}, x, y};
	}

	static Node from(const Func &f) {
		return {false, f, NONE, NONE};
	}
	
	static Node badn() {
		return {true, {0, 0, 0}, NONE, NONE};
	}
};

Node operator+(const Node &a, const Node &b) {
	if (a.bad || b.bad) {
		return a;
	}
	if (a.x != NONE) {
		if (b.x != NONE) {
			if (tie(a.x, a.y) == tie(b.x, b.y)) {
				return a;
			}
			return Node::badn();
		}
		if (b.f(a.x) == a.y) {
			return a;
		}
		return Node::badn();
	}
	if (a.f == b.f) {
		return a;
	} else if (a.f(0) == b.f(0)) {
		return Node::from(0, a.f(0));
	}
	bool init = a.f(0) > b.f(0);
	long l = 0, r = 1e4;
	while (l < r) {
		long mid = (l + r + 1) / 2;
		bool cur = a.f(mid) > b.f(mid);
		if (init == cur) {
			l = mid;
		} else {
			r = mid - 1;
		}
	}
	dbg(l);
	if (a.f(l) == b.f(r)) {
		return Node::from(l, a.f(l));
	} else if (a.f(l + 1) == b.f(l + 1)) {
		return Node::from(l + 1, a.f(l + 1));
	}
	return Node::badn();
}

void solve() {
	int n, q;
	cin >> n >> q;
	vector<Node> v;
	for (int i = 0; i < n; i++) {
		long s, l, a;
		cin >> s >> l >> a;
		v.push_back(Node::from({s, l, a}));
	}
	SegTree<Node> st(v);
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			int ind;
			long a, b, c;
			cin >> ind >> a >> b >> c;
			st.set(ind, Node::from({a, b, c}));
		} else {
			int l, r;
			cin >> l >> r;
			l--;
			r--;
			if (st.query(l, r).bad) {
				cout << 0 << endl;
			} else {
				cout << 1 << endl;
			}
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}