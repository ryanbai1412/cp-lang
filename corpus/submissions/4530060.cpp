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

const long mod = 1e9 + 7;

constexpr long bpow(long base, long exp) {
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

constexpr long inv(long x) {
	return bpow(x, mod - 2);
}

template <long base>
struct Hasher {
	static constexpr long ibase = inv(base);

	int n;
	vector<long> psum, ipow;

	Hasher(const string& s) : n(sz(s)), psum(n + 1), ipow(n) {
		long cur = 1, icur = 1;
		for (int i = 0; i < n; i++) {
			psum[i + 1] = (psum[i] + (s[i] - '0' + 1) * cur) % mod;
			ipow[i] = icur;
			icur = (icur * ibase) % mod;
			cur = (cur * base) % mod;
		}
	}

	long query(int l, int r) {
		return (psum[r + 1] - psum[l] + mod) * ipow[l] % mod;
	}
};

struct DSU {
	vector<int> p;

	DSU(int n) : p(n, -1) {}

	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}

	void merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return;
		}
		if (p[u] < p[v]) {
			swap(u, v);
		}
		p[v] += p[u];
		p[u] = v;
	}
};

string reversed(string s) {
	reverse(begin(s), end(s));
	return s;
}

namespace s1 {

int solve(const string &s) {
	int n = sz(s);
	Hasher<3> ha(s), hb(reversed(s));
	set<int> ans;
	for (int i = 0; i < n; i++) {
		for (int j = i; j < n; j++) {
			if (ha.query(i, j) == hb.query(n - j - 1, n - i - 1)) {
				ans.insert(int(ha.query(i, j)));
			}
		}
	}
	return sz(ans);
}

void solve(int n) {
	string s;
	cin >> s;
	string arr[n];
	for (int i = 0; i < n; i++) {
		arr[i] = s[i];
	}
	DSU dsu(n);
	while (--n) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		u = dsu.find(u);
		v = dsu.find(v);
		dsu.merge(u, v);
		int out = dsu.find(u);
		arr[out] = arr[u] + arr[v];
		cout << solve(arr[out]) << endl;
	}
}

}  // namespace s1

void solve() {
	int n;
	cin >> n;
	if (n <= 100) {
		s1::solve(n);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}