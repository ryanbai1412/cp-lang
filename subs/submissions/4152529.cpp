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

template <typename T, typename Lazy_T, typename Operation = plus<T>, bool associativeLazy = false>
struct LazySegTree {
	using Lazy = pair<bool, Lazy_T>;

	static constexpr T noT {};
	static constexpr Lazy noLazy {false, Lazy_T()};

	const int n;
	const Operation operation;

	vector<T> v, arr;
	vector<Lazy> lazy;

	explicit LazySegTree(const vector<T>& arr, const Operation& operation = Operation())
		: n(sz(arr)), operation(operation), v(4 * n), arr(arr), lazy(4 * n, noLazy) {
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
					return operation(query(o * 2, l, mid, ql, qr, clazy), query(o * 2 + 1, mid + 1, r, ql, qr, clazy));
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

struct Node {
	int ans, l, r, l_int, r_int;
	bool l_type, r_type;

	bool is_one() const {
		return r_int == -1;
	}

	void upd_ans(int cl, int cr) {
		ans = min(ans, __builtin_ctz(cl));
		ans = min(ans, __builtin_ctz(cr));
	}

	Node& operator+=(const Node& n) {
		r = n.r;
		ans = min(ans, n.ans);
		if (is_one()) {
			if (n.is_one()) {
				// if both are one
				if (l_type == n.l_type) {
					// if both are the same then just merge them
					l_int = r;
				} else {
					// if both are different then create right interval
					r_int = n.l_int;
					r_type = n.l_type;
				}
			} else {
				// if left is one and right isn't
				if (l_type == n.l_type) {
					// if there's no interval separation then extend this interval
					l_int = n.l_int;
					r_int = n.r_int;
					r_type = n.r_type;
				} else {
					// there's interval separation, lint doesn't change
					upd_ans(n.l, n.l_int);
					r_int = n.r_int;
					r_type = n.r_type;
				}
			}
		} else {
			if (n.is_one()) {
				// if left is not one and right is
				if (r_type == n.l_type) {
					// no interval separation, nothing is changed
				} else {
					// interval separation, lint doesn't change
					upd_ans(n.l, n.l_int);
					r_int = n.l_int;
					r_type = n.l_type;
				}
			} else {
				// neither are one
				// middle segment gets created, update that
				if (r_type == n.l_type) {
					// same type
					upd_ans(r_int, n.l_int);
				} else {
					// different type, two segments
					upd_ans(r_int, n.l);
					upd_ans(n.l, n.l_int);
				}
				// lint doesn't change
				r_int = n.r_int;
				r_type = n.r_type;
			}
		}
		return *this;
	}

	friend Node operator+(Node a, const Node& b) {
		return a += b;
	}

	static Node from_one(int l, int r, bool type) {
		return Node {100, l, r, r, -1, type, false};
	}

	int compute() const {
		if (is_one()) {
			return 100;
		}
		return min({ans, __builtin_ctz(l_int), __builtin_ctz(r_int)});
	}
};

struct Lazy {
	int add;

	Lazy operator+(const Lazy& l) const {
		return {add + l.add};
	}

	Node apply(const Node& n, int, int) const {
		if (add) {
			return Node::from_one(n.l, n.r, true);
		}
		return n;
	}
};

int solve(vector<array<int, 4>> arr) {
	vector<int> compx, compy;
	for (auto& [x1, y1, x2, y2] : arr) {
		compx.push_back(x1);
		compx.push_back(x2);
		compy.push_back(y1);
		compy.push_back(y2);
	}
	sort(begin(compx), end(compx));
	sort(begin(compy), end(compy));
	compx.erase(unique(begin(compx), end(compx)), compx.end());
	compy.erase(unique(begin(compy), end(compy)), compy.end());
	for (auto& [x1, y1, x2, y2] : arr) {
		x1 = int(lower_bound(begin(compx), end(compx), x1) - compx.begin());
		y1 = int(lower_bound(begin(compy), end(compy), y1) - compy.begin());
		x2 = int(lower_bound(begin(compx), end(compx), x2) - compx.begin());
		y2 = int(lower_bound(begin(compy), end(compy), y2) - compy.begin());
	}
	struct Event {
		int l, r, x;
	};
	vector<Event> events[sz(compx) + 1];
	for (auto& [x1, y1, x2, y2] : arr) {
		events[x1].push_back({y1, y2, 1});
		events[x2 + 1].push_back({y1, y2, -1});
	}
	vector<Node> v;
	for (int i = 0; i < sz(compy) - 1; i++) {
		v.push_back(Node::from_one(compy[i], compy[i + 1], false));
	}
	LazySegTree<Node, Lazy, plus<>, true> st(v);
	int ans = 100;
	for (auto& cevt : events) {
		for (auto& [l, r, x] : cevt) {
			st.update(l, r - 1, {x});
		}
		ans = min(ans, st.v[1].compute());
	}
	return ans;
}

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	vector<array<int, 4>> arr(k);
	for (auto& [x1, y1, x2, y2] : arr) {
		cin >> x1 >> y1 >> x2 >> y2;
		x1--;
		y1--;
	}
	int ans = min(__builtin_ctz(n), __builtin_ctz(m));
	ans = min(ans, solve(arr));
	for (auto& [x1, y1, x2, y2] : arr) {
		swap(x1, y1);
		swap(x2, y2);
	}
	ans = min(ans, solve(arr));
	cout << ans << endl;
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