#pragma GCC optimize("Ofast")

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

struct chash {
	uint64_t operator()(const pair<int, int> &x) const {
		uint64_t ans = (uint64_t(x.first) << 8) ^ x.second;
		ans ^= ans << 17;
		ans ^= ans >> 13;
		ans ^= ans << 19;
		return ans;
	}
};

const int maxn = 1 << 20, tmaxn = maxn << 1, qmaxn = 1 << 19;

template <typename T, int N>
struct avec {
	int ptr = 0;
	T arr[N];

	avec() {}

	void push_back() {
		ptr++;
	}

	void push_back(const T& t) {
		arr[ptr++] = t;
	}

	T& operator[](int ind) {
		return arr[ind];
	}

	T& back() {
		return arr[ptr - 1];
	}

	void pop_back() {
		ptr--;
	}
};

struct DS1 {
	int v[maxn + 1] {};
	avec<avec<int, 20>, qmaxn> c;

	void update(int ind, int x) {
		c.push_back();
		c.back().ptr = 0;
		c.back().push_back(ind);
		while (ind <= maxn) {
			c.back().push_back(v[ind]);
			v[ind] = max(v[ind], x);
			ind += ind & -ind;
		}
	}

	int query(int ind) const {
		int ans = 0;
		while (ind > 0) {
			ans = max(ans, v[ind]);
			ind -= ind & -ind;
		}
		if (!ans) {
			return -1e9;
		}
		return ans;
	}

	void rollback() {
		auto& x = c.back();
		int ind = x[0], ptr = 1;
		while (ind <= maxn) {
			v[ind] = x[ptr++];
			ind += ind & -ind;
		}
		c.pop_back();
	}
};

struct DS2 {
	DS1 ds;
	int ans = 1e9;
	vector<int> ansc;

	void insert(int l, int r) {
		ansc.push_back(ans);
		ans = min(ans, r - ds.query(l));
		ds.update(r, l);
	}

	int query() const {
		return ans;
	}

	void rollback() {
		ds.rollback();
		ans = ansc.back();
		ansc.pop_back();
	}
};

struct DS3 {
	DS2 ds1, ds2;

	void insert(int l, int r) {
		ds1.insert(l, r);
		ds2.insert(maxn - r - 1, maxn - l - 1);
	}

	int query() const {
		return min(ds1.query(), ds2.query());
	}

	void rollback() {
		ds1.rollback();
		ds2.rollback();
	}
};

struct cmp1 {
	bool operator()(const pair<int, int>& a, const pair<int, int>& b) const {
		if (a.second == b.second) {
			return a.first > b.first;
		}
		return a.second < b.second;
	}
};

struct cmp2 {
	bool operator()(const pair<int, int>& a, const pair<int, int>& b) const {
		if (a.first == b.first) {
			return a.second < b.second;
		}
		return a.first > b.first;
	}
};

int q;
vector<pair<int, int>> st[qmaxn << 1];
int qans[qmaxn];

void update(int o, int l, int r, int ql, int qr, const pair<int, int>& x) {
	if (ql <= l && r <= qr) {
		st[o].push_back(x);
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

void update(int l, int r, const pair<int, int>& x) {
	update(1, 0, q - 1, l, r, x);
}

DS3 ds;
void solve(int o, int l, int r) {
	for (auto& [cl, cr] : st[o]) {
		dbg("+", cl, cr);
		ds.insert(cl, cr);
	}
	if (l == r) {
		dbg("Q", l);
		qans[l] = ds.query();
	} else {
		int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
		solve(lc, l, mid);
		solve(rc, mid + 1, r);
	}
	for (int i = 0; i < sz(st[o]); i++) {
		dbg("-");
		ds.rollback();
	}
}

void solve() {
	cin >> q;
	tuple<char, int, int> arr[q];
	for (auto& [c, l, r] : arr) {
		cin >> c >> l >> r;
	}
	__gnu_pbds::gp_hash_table<pair<int, int>, vector<int>, chash> inds;
	for (int i = 0; i < q; i++) {
		auto& [c, l, r] = arr[i];
		if (c == 'A') {
			inds[{l, r}].push_back(i);
		} else {
			auto &x = inds[{l, r}];
			dbg(x.back(), i - 1);
			update(x.back(), i - 1, {l, r});
			x.pop_back();
		}
	}
	for (auto& [k, v] : inds) {
		for (auto& a : v) {
			update(a, q - 1, k);
		}
	}
	solve(1, 0, q - 1);
	multiset<pair<int, int>, cmp1> s1;
	multiset<pair<int, int>, cmp2> s2;
	multiset<pair<int, int>> opt;
	for (int i = 0; i < q; i++) {
		auto& [c, l, r] = arr[i];
		if (c == 'A') {
			s1.emplace(l, r);
			s2.emplace(l, r);
			opt.emplace(r - l, r - l);
		} else {
			s1.erase(s1.find({l, r}));
			s2.erase(s2.find({l, r}));
			opt.erase(opt.find({r - l, r - l}));
		}
		pair<int, int> ans = *opt.begin();
		auto [l1, r1] = *s1.begin();
		auto [l2, r2] = *s2.begin();
		if (r1 >= l2) {
			ans = min(ans, pair<int, int> {r1 - l2, r2 - l1});
		}
		if (qans[i] < int(1e8)) {
			ans = min(ans, pair<int, int> {0, qans[i]});
		}
		cout << ans.second << endl;
	}
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	solve();
}