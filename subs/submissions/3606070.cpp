#include "bits/extc++.h"

using namespace std;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

void solve() {
	int k;
	cin >> k;
	vector<int> ans;
	int j = 5000;
	auto count = [&](int x) {
		return x * (x - 1) / 2;
	};
	while(k) {
		for(; count(j) > k; j--) ;
		k -= count(j);
		ans.push_back(j);
	}
	int prev = 0;
	vector<pair<int, int>> edges;
	for(auto &a: ans) {
		int cl = prev + 1, cr = prev + a;
		if(prev) {
			edges.emplace_back(prev, cl);
		}
		edges.emplace_back(cl, cr);
		for(int i = cl; i < cr; i++) {
			edges.emplace_back(i, i + 1);
		}
		prev = cr;
	}
	cout << prev << " " << sz(edges) << endl;
	for(auto &[u, v]: edges) {
		cout << u << " " << v << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}