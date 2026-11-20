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

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	int perm[n], iperm[n];
	iota(perm, perm + n, 0);
	iota(iperm, iperm + n, 0);
	while (q--) {
		int ind;
		cin >> ind;
		ind--;
		swap(perm[ind], perm[ind + 1]);
		swap(iperm[perm[ind]], iperm[perm[ind + 1]]);
		ost<int> cur;
		long ans = 0;
		for (auto& a : arr) {
			ans += sz(cur) - cur.order_of_key(iperm[a] + 1);
			cur.insert(iperm[a]);
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