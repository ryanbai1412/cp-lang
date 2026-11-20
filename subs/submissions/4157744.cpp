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

struct Entry {
	long v;
	int lazy, ind;

	pair<long, int> compute() const {
		return {v + lazy, ind};
	}
};

struct Node {
	Entry opt1, opt2;

	Node& operator+=(const Node& n) {
		if (n.opt1.v < opt1.v) {
			opt1 = n.opt1;
		}
		if (n.opt2.lazy + n.opt2.v < opt2.lazy + opt2.v) {
			opt2 = n.opt2;
		}
		return *this;
	}

	Node friend operator+(Node a, const Node& b) {
		return a += b;
	}

	pair<long, int> compute() const {
		return min(opt1.compute(), opt2.compute());
	}
	
	static Node from(int ind, long v = 0) {
		return {{v, 0, ind}, {v, 0, ind}};
	}
};

struct Lazy {
	int add = -1;

	Lazy& operator+=(const Lazy& l) {
		if (l.add != -1) {
			dbg(add, l.add);
			assert(add <= l.add);
			add = l.add;
		}
		return *this;
	}

	Lazy friend operator+(Lazy a, const Lazy& b) {
		return a += b;
	}

	void apply(Node &n) const { 
		if (add != -1) {
			assert(add >= n.opt2.lazy && add >= n.opt1.lazy);
			n.opt1.lazy = add;
			n.opt2 = n.opt1;
		}
	}
};

int clog2(int x) {
	return __lg(x - 1) + 1;
}

struct ST {
	int n;
	vector<long> arr;
	vector<Node> v;
	vector<Lazy> lazy;

	explicit ST(int _n) : n(1 << clog2(_n)), arr(n), v(n << 1), lazy(n << 1) {
		for (int i = n; i < 2 * n; i++) {
			v[i] = Node::from(i - n);
		}
		for (int i = n - 1; i; i--) {
			v[i] = v[i * 2] + v[i * 2 + 1];
		}
	}

	void maintain(int o, int l, int r) {
		if (l == r) {
			v[o] = Node::from(l, arr[l]);
		} else {
			v[o] = v[o * 2] + v[o * 2 + 1];
		}
		lazy[o].apply(v[o]);
	}

	void set(int ind, long x) {
		arr[ind] = x;
		v[ind + n] = Node::from(ind, x);
		ind += n;
		lazy[ind].apply(v[ind]);
		for (; ind > 1;) {
			v[ind >> 1] = v[ind] + v[ind ^ 1];
			ind >>= 1;
			lazy[ind].apply(v[ind]);
		}
	}

	void update(int o, int l, int r, int ql, int qr, const Lazy& x) {
		if (ql <= l && r <= qr) {
			lazy[o] += x;
		} else {
			int mid = (l + r) / 2;
			lazy[o * 2] += lazy[o];
			lazy[o * 2 + 1] += lazy[o];
			lazy[o] = {-1};
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
		}
		maintain(o, l, r);
	}

	void rupd(int l, int r, int x) {
		dbg("RUPD", l, r, x);
		update(1, 0, n - 1, l, r, {x});
	}

	Node query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
		if (ql <= l && r <= qr) {
			auto cur = v[o];
			clazy.apply(cur);
			return cur;
		} else {
			clazy = lazy[o] + clazy;
			int mid = (l + r) / 2;
			Node ans = Node::from(0, 1e18);
			if (ql <= mid) {
				ans += query(o * 2, l, mid, ql, qr, clazy);
			}
			if (mid < qr) {
				ans += query(o * 2 + 1, mid + 1, r, ql, qr, clazy);
			}
			return ans;
		}
	}

	pair<long, int> query(int l, int r) const {
		return query(1, 0, n - 1, l, r, {-1}).compute();
	}
};

void solve() {
	int m, n;
	cin >> m >> n;
	string names[n];
	int arr[n];
	for (int i = 0; i < n; i++) {
		cin >> names[i] >> arr[i];
	}
	ST st(n + 1);
	pair<long, int> dp[n + 1];
	vector<pair<int, int>> s;
	s.emplace_back(1e9, n);
	for (int i = n - 1; i >= 0; i--) {
		while (s.back().first < arr[i]) {
			s.pop_back();
		}
		st.rupd(i + 1, s.back().second, arr[i]);
		s.emplace_back(arr[i], i);
		dp[i] = st.query(i + 1, min(n, i + m));
		st.set(i, dp[i].first);
		dbg(i, dp[i].first, st.query(6, 6).first);
	}
	for (int i = 0; i < n; i++) {
		cerr << dp[i].first << " \n"[i == n - 1];
	}
	cout << "Total Time: " << dp[0].first << endl;
	int i = 0;
	while (i < n) {
		int e = dp[i].second;
		for (; i < e; i++) {
			cout << names[i] << " \n"[i == e - 1];
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