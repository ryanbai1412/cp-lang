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

const int maxn = 1e7;

struct Node {
	char c;
	Node *left, *right;
};

Node *newNode(char c) {
	static Node heap[maxn];
	static int hind = 0;
	heap[hind].c = c;
	return heap + (hind++);
}

Node *head = newNode(0), *tail = newNode(0);

Node *moveLeft(Node *n, int x) {
	for(int i = 0; i < x; i++) {
		n = n->left;
	}
	return n;
}

Node *moveRight(Node *n, int x) {
	for(int i = 0; i < x; i++) {
		n = n->right;
	}
	return n;
}

void insert(Node *n, string s) {
	Node *nr = n->right;
	for(int i = 0; i < sz(s); i++) {
		Node *next = newNode(s[i]);
		n->right = next;
		next->left = n;
		n = next;
	}
	n->right = nr;
	nr->left = n;
}

void del(Node *n, int x) {
	Node *nr = moveRight(n, x + 1);
	n->right = nr;
	nr->left = n;
}

string get(Node *n, int x) {
	string ans;
	for(int i = 0; i < x; i++) {
		ans += (n = n->right)->c;
	}
	return ans;
}

void solve() {
	int q, csize = 0;
	cin >> q;
	head->right = tail;
	tail->left = head;
	Node *cursor = head;
	for(int _ = 0; _ < q; _++) {
		dbg(_);
		string s;
		cin >> s;
		if(s == "Prev") {
			cursor = cursor->left;
		}else if(s == "Next") {
			cursor = cursor->right;
		}else {
			int n;
			cin >> n;
			if(s == "Move") {
				if(n < csize / 2) {
					cursor = moveRight(head, n);
				}else {
					cursor = moveLeft(tail, csize - n + 1);
				}
			}else if(s == "Insert") {
				csize += n;
				string x, buf;
				while(sz(x) < n) {
					getline(cin, buf);
					x += buf;
				}
				insert(cursor, x);
			}else if(s == "Delete") {
				csize -= n;
				// ??? apparently samples suggest this?
				del(cursor->left, n);
			}else if(s == "Get") {
				cout << get(cursor, n) << endl;
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