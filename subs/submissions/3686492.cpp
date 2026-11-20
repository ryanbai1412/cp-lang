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
	vector<int> inds[2 * n];
	for(int i = 0; i < 2 * n; i++) {
		int x;
		cin >> x;
		x--;
		inds[x].push_back(i + 1);
	}
	vector<pair<int, int>> ans;
	map<int, vector<int>> acnt;
	for(int i = 0; i < 2 * n; i++) {
		acnt[sz(inds[i])].push_back(i);
	}
	for(int i = 0; i < n; i++) {
		auto a = --acnt.end();
		if(sz(a->second) >= 2) {
			int u = a->second.back();
			a->second.pop_back();
			int v = a->second.back();
			a->second.pop_back();
			ans.emplace_back(u, v);
			acnt[a->first - 1].push_back(u);
			acnt[a->first - 1].push_back(v);
			if(a->second.empty()) {
				acnt.erase(a);
			}
		}else {
			auto b = prev(a);
			if(b->first == 0) {
				ans.emplace_back(a->second[0], a->second[0]);
				acnt[a->first - 2].push_back(a->second[0]);
				acnt.erase(a);
			}else {
				ans.emplace_back(a->second[0], b->second.back());
				acnt[a->first - 1].push_back(a->second[0]);
				acnt[b->first - 1].push_back(b->second.back());
				acnt[b->first].pop_back();
				int cur = a->first;
				if(acnt[b->first].empty()) {
					acnt.erase(b);
				}
				acnt.erase(acnt.find(cur));
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