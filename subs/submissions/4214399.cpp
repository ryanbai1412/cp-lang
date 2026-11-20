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

constexpr long bpow(long base, long exp, long mod) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

constexpr long inv(long x, long mod) {
	return bpow(x, mod - 2, mod);
}

template <long mod>
struct Hasher {
	static int ctoi(char c) {
		return c - 'a';
	}

	static constexpr long base = 31, ibase = inv(base, mod);
	int n;
	string s;
	vector<long> psum, ipow;

	explicit Hasher(const string& s) : n(sz(s)), s(s), psum(n + 1), ipow(n + 1) {
		long cmul = 1;
		ipow[0] = 1;
		for (int i = 0; i < n; i++) {
			psum[i + 1] = (psum[i] + cmul * (ctoi(s[i]) + 1)) % mod;
			ipow[i + 1] = (ipow[i] * ibase) % mod;
			cmul = (cmul * base) % mod;
		}
	}

	char operator[](int ind) const {
		return s[ind];
	}

	int query(int l, int r) const {
		return int((psum[r] - psum[l] + mod) * ipow[l] % mod);
	}
};

template <long mod>
struct Slice {
	const Hasher<mod>* hasher;
	int ind;

	Slice(const Hasher<mod>* hasher, int ind) : hasher(hasher), ind(ind) {}

	int size() const {
		return hasher->n - ind;
	}

	int query(int len) const {
		return hasher->query(ind, ind + len);
	}

	char operator[](int i) const {
		return (*hasher)[ind + i];
	}
	
	string to_string() const {
		return hasher->s.substr(ind);
	}

	bool operator<(const Slice& s) const {
		int l = 0, r = min(size(), sz(s));
		if (query(r) == s.query(r)) {
			return size() < sz(s);
		}
		while (l < r) {
			int mid = (l + r) / 2;
			if (query(mid) != s.query(mid)) {
				r = mid;
			} else {
				l = mid + 1;
			}
		}
		return (*this)[l - 1] < s[l - 1];
	}
};

struct DS {
	vector<int> arr;

	explicit DS(const vector<int>& arr) : arr(arr) {}

	int query(int l, int r, int k) const {
		vector<int> cur(arr.begin() + l, arr.begin() + r + 1);
		sort(begin(cur), end(cur));
		return cur[k];
	}
};

const long mod = 1e9 + 7;

void solve() {
	string s;
	cin >> s;
	set<char> cnt;
	for (auto& a : s) {
		cnt.insert(a);
	}
	if (sz(cnt) == 1) {
		char c = *cnt.begin();
		int q;
		cin >> q;
		while (q--) {
			string t;
			int k;
			cin >> t >> k;
			bool ae = true;
			for (auto& a : t) {
				ae &= a == c;
			}
			if (!ae) {
				cout << -1 << endl;
				continue;
			}
			k--;
			int l = 0, r = sz(s) - sz(t);
			if (k < r - l + 1) {
				cout << l + k + 1 << endl;
			} else {
				cout << -1 << endl;
			}
		}
		return;
	}
	int n = sz(s);
	Hasher<mod> hasher(s);
	vector<Slice<mod>> sa;
	sa.reserve(n);
	for (int i = 0; i < n; i++) {
		sa.emplace_back(&hasher, i);
	}
	sort(begin(sa), end(sa));
	vector<int> vals;
	for (auto& [_, x] : sa) {
		dbg(x);
		vals.push_back(x);
	}
	DS ds(vals);
	int q;
	cin >> q;
	while (q--) {
		string t;
		int k;
		cin >> t >> k;
		k--;
		Hasher<mod> chsh(t);
		Slice<mod> slice(&chsh, 0);
		long xhsh = chsh.query(0, sz(t));
		auto check = [&](int i) -> bool {
			return i < n && sz(sa[i]) >= sz(t) && sa[i].query(sz(t)) == xhsh;
		};
		int rl = int(lower_bound(begin(sa), end(sa), slice) - sa.begin());
		if (!check(rl)) {
			cout << -1 << endl;
			continue;
		}
		int l = rl;
		int r = n - 1;
		while (l < r) {
			int mid = (l + r + 1) / 2;
			if (check(mid)) {
				l = mid;
			} else {
				r = mid - 1;
			}
		}
		l = rl;
		dbg(l, r);
		if (k < r - l + 1) {
			cout << ds.query(l, r, k) + 1 << endl;
		} else {
			cout << -1 << endl;
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