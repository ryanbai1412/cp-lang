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

namespace __gnu_pbds {

template <typename T>
using ost = tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;

}

using __gnu_pbds::ost;

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
			ans += n - j;
		}
		return ans;
	};
	auto solveb = [&](int a, int b) -> long {
		if (sz(inds[a]) < sz(inds[b])) {
			return -solvea(b, a);
		}
		return solvea(a, b);
	};
	long ans = 0;
	{
		ost<int> s;
		for (auto& a : arr) {
			ans += int(sz(s) - s.order_of_key(a + 1));
			s.insert(a);
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