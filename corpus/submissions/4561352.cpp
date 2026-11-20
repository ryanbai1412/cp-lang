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
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

namespace __gnu_pbds {
template <typename T>
using ost = tree<T, null_type, less<>, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	vector<int> ind[n];
	for (int i = 0; i < n; i++) {
		ind[arr[i]].push_back(i);
	}
	int av[n];
	priority_queue<int> pq;
	for (int i = n - 1; i >= 0; i--) {
		for (auto& a : ind[i]) {
			pq.push(a);
		}
		if (!sz(pq)) {
			cout << -1 << endl;
			return;
		}
		av[i] = pq.top();
		dbg(i, av[i], sz(pq));
		pq.pop();
	}
	ost<pair<int, int>> s;
	long ans = 0;
	for (int i = 0; i < n; i++) {
		ans += sz(s) - s.order_of_key({av[i] + 1, -1});
		s.insert({av[i], i});
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}