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

const int maxn = 3e4;

struct Node {
	int id, ind;
} nodes[maxn];

struct DS {
	int id, il, ir;
	deque<int> x;

	DS(): id(0), il(0), ir(0), x() {}

	void push_front(int node) {
		x.push_front(node);
		auto &[id, ind] = nodes[node];
		id = this->id;
		ind = --il;
	}

	void push_back(int node) {
		x.push_back(node);
		auto &[id, ind] = nodes[node];
		id = this->id;
		ind = ir++;
	}

	int size() {
		return ir - il;
	}
} ds[maxn];

void solve() {
	for(int i = 0; i < maxn; i++) {
		ds[i].id = i;
		ds[i].push_back(i);
	}
	int q;
	cin >> q;
	while(q--) {
		char t;
		int u, v;
		cin >> t >> u >> v;
		u--;
		v--;
		if(t == 'M') {
			u = nodes[u].id;
			v = nodes[v].id;
			if(sz(ds[u]) < sz(ds[v])) {
				for(auto &a: ds[u].x) {
					ds[v].push_back(a);
				}
			}else {
				reverse(begin(ds[v].x), end(ds[v].x));
				for(auto &a: ds[v].x) {
					ds[u].push_front(a);
				}
			}
		}else {
			if(nodes[u].id == nodes[v].id) {
				cout << abs(nodes[u].ind - nodes[v].ind) - 1 << endl;
			}else {
				cout << -1 << endl;
			}
		}
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