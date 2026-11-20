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

const pair<int, int> sq[] = {{0, 0},
							 {0, 1},
							 {1, 1},
							 {1, 0}};
const pair<int, int> octa[] = {{-1, 0},
							   {-1, 1},
							   {0,  2},
							   {1,  2},
							   {2,  1},
							   {2,  0},
							   {1,  -1},
							   {0,  -1}};

void solve() {
	int n;
	cin >> n;
	map<pair<int, int>, vector<pair<int, int>>> nxt;
	for(int i = 0; i < n; i++) {
		int x, y;
		cin >> x >> y;
		if((x + y) % 4 == 0) {
			vector<pair<int, int>> corder;
			for(auto &[dx, dy]: sq) {
				corder.emplace_back(x + dx, y + dy);
			}
			for(int j = 0; j < sz(corder); j++) {
				nxt[corder[j]].push_back(corder[(j + 1) % sz(corder)]);
			}
		}else {
			vector<pair<int, int>> corder;
			for(auto &[dx, dy]: octa) {
				corder.emplace_back(x + dx, y + dy);
			}
			for(int j = 0; j < sz(corder); j++) {
				nxt[corder[j]].push_back(corder[(j + 1) % sz(corder)]);
			}
		}
	}
	set<pair<int, int>> invalid;
	for(auto &[k, v]: nxt) {
		if(sz(v) == 3) {
			invalid.insert(k);
		}
	}
	for(auto &[k, v]: nxt) {
		vector<pair<int, int>> nv;
		for(auto &[x, y]: v) {
			if(!invalid.count({x, y})) {
				nv.emplace_back(x, y);
			}
		}
		v = nv;
	}
	pair<int, int> start {INT_MAX, INT_MAX};
	for(auto &[k, v]: nxt) {
		if(sz(v) == 1) {
			start = min(start, k);
		}
	}
	assert(start.first != INT_MAX);
	auto[x, y] = start;
	vector<pair<int, int>> ans;
	do {
		ans.emplace_back(x, y);
		assert(!invalid.count({x, y}));
		tie(x, y) = nxt[{x, y}][0];
	}while(pair<int, int> {x, y} != start);
	cout << sz(ans) << endl;
	for(auto &[x, y]: ans) {
		cout << x << " " << y << endl;
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