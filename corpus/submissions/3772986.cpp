#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

template<typename T, typename _Lazy, typename Operation = plus<T>, bool associativeLazy = false>
struct LazySegTree {
	using Lazy = pair<bool, _Lazy>;

	static constexpr T noT {};
	static constexpr Lazy noLazy {false, _Lazy()};

	const int n;
	const Operation operation;

	vector<T> v, arr;
	vector<Lazy> lazy;

	LazySegTree(const vector<T> &arr,
				const Operation &operation = Operation()): n(sz(arr)), operation(operation), v(4 * n), arr(arr),
														   lazy(4 * n, noLazy) {
		build(1, 0, n - 1);
	}

	void build(int o, int l, int r) {
		if(l == r) {
			v[o] = arr[l];
		}else {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	inline T applyLazy(const T &x, const Lazy &lazy, int l, int r) const {
		if(lazy.first) {
			return lazy.second.apply(x, l, r);
		}
		return x;
	}

	inline void maintain(int o, int l, int r) {
		if(l == r) {
			v[o] = arr[l];
		}else {
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
		v[o] = applyLazy(v[o], lazy[o], l, r);
	}

	inline void mergeLazy(Lazy &first, const _Lazy &second) const {
		if(first.first) {
			first.second = first.second + second;
		}else {
			first.first = true;
			first.second = second;
		}
	}

	inline void mergeClazy(Lazy &clazy, const Lazy &first) const {
		if(first.first) {
			if(clazy.first) {
				clazy.second = first.second + clazy.second;
			}else {
				clazy = first;
			}
		}
	}

	inline T mergePrefix(bool first, const T &pref, const T &next) const {
		if(first) {
			return next;
		}
		return operation(pref, next);
	}

	void set(int o, int l, int r, int ind, const T &x) {
		if(l == r) {
			v[o] = arr[l] = x;
			lazy[o].first = false;
		}else {
			int mid = (l + r) / 2;
			if(!associativeLazy && lazy[o].first) {
				mergeLazy(lazy[o * 2], lazy[o].second);
				mergeLazy(lazy[o * 2 + 1], lazy[o].second);
				lazy[o].first = false;
				if(ind <= mid) {
					set(o * 2, l, mid, ind, x);
					maintain(o * 2 + 1, mid + 1, r);
				}else {
					set(o * 2 + 1, mid + 1, r, ind, x);
					maintain(o * 2, l, mid);
				}
			}else {
				if(ind <= mid) {
					set(o * 2, l, mid, ind, x);
				}else {
					set(o * 2 + 1, mid + 1, r, ind, x);
				}
			}
			maintain(o, l, r);
		}
	}

	inline void set(int ind, const T &t) {
		set(1, 0, n - 1, ind, t);
	}

	T get(int o, int l, int r, int ind, Lazy clazy) const {
		if(l == r) {
			return applyLazy(v[o], clazy, l, r);
		}else {
			int mid = (l + r) / 2;
			mergeClazy(clazy, lazy[o]);
			if(ind <= mid) {
				return get(o * 2, l, mid, ind, clazy);
			}else {
				return get(o * 2 + 1, mid + 1, r, ind, clazy);
			}
		}
	}

	inline T get(int ind) const {
		return get(1, 0, n - 1, ind, noLazy);
	}

	void update(int o, int l, int r, int ql, int qr, const _Lazy &x) {
		if(ql <= l && r <= qr) {
			mergeLazy(lazy[o], x);
		}else {
			int mid = (l + r) / 2;
			if(!associativeLazy && lazy[o].first) {
				mergeLazy(lazy[o * 2], lazy[o].second);
				mergeLazy(lazy[o * 2 + 1], lazy[o].second);
				lazy[o].first = false;
				if(ql <= mid) {
					update(o * 2, l, mid, ql, qr, x);
				}else {
					maintain(o * 2, l, mid);
				}
				if(mid < qr) {
					update(o * 2 + 1, mid + 1, r, ql, qr, x);
				}else {
					maintain(o * 2 + 1, mid + 1, r);
				}
			}else {
				if(ql <= mid) {
					update(o * 2, l, mid, ql, qr, x);
				}
				if(mid < qr) {
					update(o * 2 + 1, mid + 1, r, ql, qr, x);
				}
			}
		}
		maintain(o, l, r);
	}

	inline void update(int l, int r, const _Lazy &x) {
		update(1, 0, n - 1, l, r, x);
	}

	T query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
		if(ql <= l && r <= qr) {
			return applyLazy(v[o], clazy, l, r);
		}else {
			int mid = (l + r) / 2;
			mergeClazy(clazy, lazy[o]);
			if(ql <= mid) {
				if(mid < qr) {
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
	int l, r, v;

	Node operator+(const Node &n) const {
		return {l, n.r, v + n.v + (r != n.l)};
	}
};

struct Lazy {
	int x = -1;

	Lazy operator+(const Lazy &l) const {
		if(l.x == -1) {
			return *this;
		}
		return l;
	}

	Node apply(const Node &n, int l, int r) const {
		return {x, x, 0};
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	vector<Node> arr;
	for(int i = 0; i < n; i++) {
		int x;
		cin >> x;
		arr.push_back({x, x, 0});
	}
	LazySegTree<Node, Lazy> st(arr);
	int rot = 0;
	bool rev = false;
	auto conv = [&](int x) -> int {
		if(rev) {
			x = n - x - 1;
		}
		x = (x - rot + n) % n;
		return x;
	};
	auto norm = [&](int &u, int &v) -> void {
		u--;
		v--;
		bool cmp = u <= v;
		u = conv(u);
		v = conv(v);
		if(cmp != (u <= v)) {
			swap(u, v);
		}
	};
	int q;
	cin >> q;
	while(q--) {
		string s;
		cin >> s;
		if(s == "R") {
			int x;
			cin >> x;
			if(rev) {
				x = -x;
			}
			rot = (rot + x + n) % n;
		}else if(s == "F") {
			rev ^= true;
		}else if(s == "S") {
			int u, v;
			cin >> u >> v;
			norm(u, v);
			if(u != v) {
				Node tmp = st.get(u);
				st.set(u, st.get(v));
				st.set(v, tmp);
			}
		}else if(s == "P") {
			int l, r, x;
			cin >> l >> r >> x;
			norm(l, r);
			if(l <= r) {
				st.update(l, r, {x});
			}else {
				st.update(l, n - 1, {x});
				st.update(0, r, {x});
			}
		}else if(s == "C") {
			Node ans = st.query(0, n - 1);
			cout << max(1, ans.v + (ans.l != ans.r)) << endl;
		}else if(s == "CS") {
			int l, r;
			cin >> l >> r;
			norm(l, r);
			Node ans {};
			if(l <= r) {
				ans = st.query(l, r);
			}else {
				ans = st.query(l, n - 1) + st.query(0, r);
			}
			cout << ans.v + 1 << endl;
		}else {
			assert(false);
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}