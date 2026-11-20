#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

void solve() {
	int n;
	cin >> n;
	map<int, vector<int>> inds;
	for(int i = 0; i < 2 * n; i++) {
		int x;
		cin >> x;
		x--;
		inds[x].push_back(i + 1);
	}
	vector<pair<int, int>> ans;
	priority_queue<pair<int, int>> pq;
	for(auto &[k, v]: inds) {
		pq.emplace(sz(v), k);
	}
	for(int i = 0; i < n; i++) {
		if(sz(pq) == 1) {
			auto [cnt, x] = pq.top();
			pq.pop();
			ans.emplace_back(x, x);
			if(cnt - 2) {
				pq.emplace(cnt - 2, x);
			}
		}else {
			auto [cnt1, x1] = pq.top();
			pq.pop();
			auto [cnt2, x2] = pq.top();
			pq.pop();
			ans.emplace_back(x1, x2);
			if(cnt1 - 1) {
				pq.emplace(cnt1 - 1, x1);
			}
			if(cnt2 - 1) {
				pq.emplace(cnt2 - 1, x2);
			}
		}
	}
	int vans = 0;
	for(auto &[u, v]: ans) {
		vans += u != v;
	}
	cout << vans << endl;
	for(auto &[u, v]: ans) {
		int a = inds[u].back();
		inds[u].pop_back();
		int b = inds[v].back();
		inds[v].pop_back();
		cout << a << " " << b << endl;
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