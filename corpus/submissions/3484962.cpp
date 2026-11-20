#include "bits/stdc++.h"
#include "ext/pb_ds/assoc_container.hpp"

using namespace std;
using namespace __gnu_pbds;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, m;
	cin >> n >> m;
	pair<int, int> a[n], b[m];
	tree<pair<int, int>, null_type, less<>, rb_tree_tag, tree_order_statistics_node_update> cur;
	map<int, vector<int>> vals;
	for(int i = 0; i < n; i++) {
		cin >> a[i].first >> a[i].second;
	}
	for(int i = 0; i < m; i++) {
		cin >> b[i].first >> b[i].second;
		vals[b[i].first].push_back(i);
		cur.insert({b[i].first, i});
	}
	sort(a, a + n, [](auto x, auto y) {return x.second < y.second;});
	sort(b, b + m, [](auto x, auto y) {return x.first < y.first;});
	long ans = 0;
	int j = 0;
	for(int i = 0; i < n; i++) {
		while(j < m && b[j].second < a[i].second) {
			cur.erase({b[j].first, vals[b[j].first].back()});
			vals[b[j].first].pop_back();
			j++;
		}
		ans += cur.order_of_key({a[i].first + 1, 1e9});
	}
	cout << ans << endl;
}