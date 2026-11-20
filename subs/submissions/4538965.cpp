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

struct Bit {
	int n;
	vector<int> v;
	
	Bit(int n) : n(n + 1), v(n + 1, 0) {}

	void update(int ind, int x) {
		ind++;
		while (ind <= n) {
			v[ind] += x;
			ind += ind & -ind;
		}
	}

	int psum(int ind) const {
		int ans = 0;
		while (ind > 0) {
			ans += v[ind];
			ind -= ind & -ind;
		}
		return ans;
	}
	
	int query(int l, int r) {
		return psum(r + 1) - psum(l);
	}
};

const int maxn = 1e5 + 5, bsize = 400;

int psum[bsize][maxn];

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	vector<int> inds[m];
	for (int i = 0; i < n; i++) {
		inds[arr[i]].push_back(i);
	}
	int ord[m];
	iota(ord, ord + m, 0);
	sort(ord, ord + m, [&](int a, int b) -> bool {
		return sz(inds[a]) > sz(inds[b]);
	});
	int oind[m];
	for (int i = 0; i < m; i++) {
		oind[ord[i]] = i;
	}
	for (int i = 0; i < min(m, bsize); i++) {
		psum[i][0] = 0;
		for (int j = 0; j < n; j++) {
			psum[i][j + 1] = psum[i][j] + (arr[j] == ord[i]);
		}
	}
	// sz a > sz b
	auto solvea = [&](int a, int b) -> long {
		static map<pair<int, int>, long> qcache;
		auto [it, inserted] = qcache.insert({{a, b}, 0});
		if (!inserted) {
			return it->second;
		}
		long& ans = it->second;
		if (oind[a] < bsize) {
			for (auto& x : inds[b]) {
				ans += psum[oind[a]][x];
			}
			return ans;
		}
		int j = 0;
		for (int i = 0; i < sz(inds[a]); i++) {
			while (j < sz(inds[b]) && inds[b][j] <= inds[a][i]) {
				j++;
			}
			ans += sz(inds[b]) - j;
		}
		return ans;
	};
	auto solveb = [&](int a, int b) -> long {
		long tot = long(sz(inds[a])) * sz(inds[b]);
		if (sz(inds[a]) < sz(inds[b])) {
			return tot - solvea(b, a);
		}
		return solvea(a, b);
	};
	long ans = 0;
	{
		Bit cur(m + 1);
		for (auto& a : arr) {
			//			ans += sz(cur) - cur.order_of_key(iperm[a] + 1);
			//			cur.insert(iperm[a]);
			ans += cur.query(a + 1, m - 1);
			cur.update(a, 1);
		}
	}
	int perm[m];
	iota(perm, perm + m, 0);
	while (q--) {
		int ind;
		cin >> ind;
		ind--;
		int u = perm[ind], v = perm[ind + 1];
		long tot = long(sz(inds[u])) * sz(inds[v]);
		long av = solveb(u, v), bv = tot - av;
		dbg(u, v, av);
		ans += av - bv;
		cout << ans << endl;
		swap(perm[ind], perm[ind + 1]);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}