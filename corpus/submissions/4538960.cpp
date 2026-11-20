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

namespace __gnu_pbds {
template <typename T>
using ost = tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

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

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	int perm[m], iperm[m];
	iota(perm, perm + m, 0);
	iota(iperm, iperm + m, 0);
	while (q--) {
		int ind;
		cin >> ind;
		ind--;
		swap(perm[ind], perm[ind + 1]);
		for (int i = 0; i < m; i++) {
			iperm[perm[i]] = i;
		}
		Bit cur(m);
		long ans = 0;
		for (auto& a : arr) {
//			ans += sz(cur) - cur.order_of_key(iperm[a] + 1);
//			cur.insert(iperm[a]);
			ans += cur.query(iperm[a] + 1, m - 1);
			cur.update(iperm[a], 1);
		}
		cout << ans << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}