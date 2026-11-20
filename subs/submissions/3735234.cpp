#include "bits/extc++.h"
#include "ext/rope"

using namespace std;
using namespace __gnu_cxx;

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

const int maxn = 4e6;

char buf[maxn];

void solve() {
	int q;
	cin >> q;
	rope<char> rope;
	int cursor = 0;
	for(int _ = 0; _ < q; _++) {
		dbg(_);
		string s;
		cin >> s;
		if(s == "Prev") {
			cursor--;
		}else if(s == "Next") {
			cursor++;
		}else {
			int n;
			cin >> n;
			if(s == "Move") {
				cursor = n;
			}else if(s == "Insert") {
				string x, buf;
				while(sz(x) < n) {
					getline(cin, buf);
					x += buf;
				}
				assert(sz(x) == n);
				rope.insert(cursor, x.c_str());
			}else if(s == "Delete") {
				rope.erase(cursor, n);
			}else if(s == "Get") {
				for(int i = cursor; i < cursor + n; i++) {
					cout << rope[i];
				}
				cout << endl;
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