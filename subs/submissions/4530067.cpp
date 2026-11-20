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

	int query(int l, int r) {
		return int((psum[r + 1] - psum[l] + mod) * ipow[l] % mod);
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

vector<int> s3solve(const string &s) {
	int n = sz(s);
	Hasher<3> ha(s), hb(reversed(s));
	int mlen[n] {};
	{
		auto ipalin = [&](int i, int len) -> bool {
			int l = i - len + 1, r = i + len - 1;
			return 0 <= l && r < n && ha.query(l, r) == hb.query(n - r - 1, n - l - 1);
		};
		vector<int> events[n + 1];
		for (int i = 0; i < n; i++) {
			int l = 0, r = n;
			while (l < r) {
				int mid = (l + r + 1) / 2;
				if (ipalin(i, mid)) {
					l = mid;
				} else {
					r = mid - 1;
				}
			}
			events[i].push_back(i + 1);
			events[i + l].push_back(-(i + 1));
		}
		set<int> cur;
		for (int i = 0; i < n; i++) {
			for (auto& a : events[i]) {
				int ind = abs(a) - 1;
				if (a < 0) {
					cur.erase(ind);
				} else {
					cur.insert(ind);
				}
			}
			int x = *cur.begin();
			mlen[i] = max(mlen[i], (i - x) * 2 + 1);
		}
	}
	{
		auto ipalin = [&](int i, int len) -> bool {
			int l = i - len, r = i + len - 1;
			return 0 <= l && r < n && ha.query(l, r) == hb.query(n - r - 1, n - l - 1);
		};
		vector<int> events[n + 1];
		for (int i = 0; i < n; i++) {
			int l = 0, r = n;
			while (l < r) {
				int mid = (l + r + 1) / 2;
				if (ipalin(i, mid)) {
					l = mid;
				} else {
					r = mid - 1;
				}
			}
			events[i].push_back(i + 1);
			events[i + l].push_back(-(i + 1));
		}
		set<int> cur;
		for (int i = 0; i < n; i++) {
			for (auto& a : events[i]) {
				int ind = abs(a) - 1;
				if (a < 0) {
					cur.erase(ind);
				} else {
					cur.insert(ind);
				}
			}
			if (sz(cur)) {
				int x = *cur.begin();
				mlen[i] = max(mlen[i], (i - x + 1) * 2);
			}
		}
	}
	set<int> ans;
	vector<int> cans(n);
	for (int i = 0; i < n; i++) {
		ans.insert(ha.query(i - mlen[i] + 1, i));
		cans[i] = sz(ans);
	}
	return cans;
}

namespace s1 {

void solve(int n) {
	string s;
	cin >> s;
	string arr[n];
	for (int i = 0; i < n; i++) {
		arr[i] = s[i];
	}
	DSU dsu(n);
	int queries[n - 1][2];
	vector<string> qa;
	for (auto& [u, v] : queries) {
		cin >> u >> v;
		u--;
		v--;
	}
	for (auto& [u, v] : queries) {
		u = dsu.find(u);
		v = dsu.find(v);
		dsu.merge(u, v);
		qa.push_back(arr[dsu.find(u)] = arr[u] + arr[v]);
	}
	s = arr[dsu.find(0)];
	vector<int> ans[n];
	for (int i = 0; i < n; i++) {
		ans[i] = s3solve(s.substr(i));
	}
	Hasher<3> h(s);
	map<int, int> ind[n + 1];
	for (int i = 1; i <= n; i++) {
		for (int j = 0; i + j <= n; j++) {
			ind[i][h.query(j, j + i - 1)] = j;
		}
	}
	for (auto& a : qa) {
		int l = ind[sz(a)][Hasher<3>(a).query(0, sz(a) - 1)];
		dbg(a, l);
		cout << ans[l][sz(a) - 1] << endl;
	}
}

}  // namespace s1

namespace s2 {

void solve(int n) {
	string s;
	cin >> s;
	auto ans = s3solve(s);
	for (int i = 1; i < n; i++) {
		cout << ans[i] << endl;
	}
}

}  // namespace s2

void solve() {
	int n;
	cin >> n;
	if (n <= 100) {
		s1::solve(n);
	} else {
		s2::solve(n);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}