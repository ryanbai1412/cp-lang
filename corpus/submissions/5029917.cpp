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
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
using il2 = initializer_list<initializer_list<T>>;

struct Shape {
    char label;
    bool arr[5][5];

    constexpr Shape() : label(0), arr {} {}

    constexpr Shape(char label, bool _arr[5][5]) : label(label), arr {} {
        memcpy(arr, _arr, sizeof(arr));
    }

    constexpr Shape(char label, il2<int> v) : label(label), arr {} {
        int i = 0;
        for (auto& a : v) {
            int j = 0;
            bool f = true;
            for (auto& b : a) {
                for (int k = 0; k < b; k++, j++) {
                    arr[i][j] = f;
                }
                f ^= true;
            }
            i++;
        }
    }

    friend ostream& operator<<(ostream& out, const Shape& s) {
        out << s.label;
        for (auto& a : s.arr) {
            out << endl;
            for (auto& b : a) {
                out << ".*"[b];
            }
        }
        return out;
    }

    constexpr Shape rotate() const {
        bool narr[5][5] {};
        for (int i = 0; i < 5; i++) {
            for (int j = 0; j < 5; j++) {
                narr[j][4 - i] = arr[i][j];
            }
        }
        return {label, narr};
    }
    
    constexpr Shape reflect() const {
        bool narr[5][5] {};
        for (int i = 0; i < 5; i++) {
            for (int j = 0; j < 5; j++) {
                narr[4 - i][j] = arr[i][j];
            }
        }
        return {label, narr};
    }

    template <typename T>
    void feach(int dx, int dy, const T& t) {
        for (int i = 0; i < 5; i++) {
            for (int j = 0; j < 5; j++) {
                if (arr[i][j]) {
                    t(dx + i, dy + j);
                }
            }
        }
    }

    constexpr static array<Shape, 8> from(Shape s) {
        array<Shape, 8> ans;
        ans[0] = s;
        for (int i = 0; i < 3; i++) {
            ans[i + 1] = ans[i].rotate();
        }
        ans[4] = s.reflect();
        for (int i = 4; i < 7; i++) {
            ans[i + 1] = ans[i].rotate();
        }
        return ans;
    }

    template <size_t N>
    constexpr static array<array<Shape, 8>, N> from(array<il2<int>, N> v) {
        array<array<Shape, 8>, N> ans;
        int i = 0;
        char label = 'A';
        for (auto& a : v) {
            ans[i++] = Shape::from({label++, a});
        }
        return ans;
    }
};

array<array<Shape, 8>, 12> shapes =
    Shape::from(array<il2<int>, 12> {il2<int> {{2}, {1}},
                                     {{4}},
                                     {{3}, {1}},
                                     {{2}, {2}},
                                     {{1}, {1}, {3}},
                                     {{4}, {0, 1, 1}},
                                     {{3}, {1, 1, 1}},
                                     {{3}, {2}},
                                     {{3}, {0, 2, 2}},
                                     {{0, 1, 1}, {3}, {0, 1, 1}},
                                     {{1}, {2}, {0, 1, 2}},
                                     {{4}, {1}}});

constexpr int n = 10;

char arr[20][20];
bool vis[256];

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y <= x; 
}

set<long> svis;

void dfs(int x, int y) {
    if (y == x + 1) {
        dfs(x + 1, 0);
        return;
    } else if (x == n) {
        for (int i = 0; i < n; i++) {
            cout << arr[i] << endl;
        }
        exit(0);
    } else if (arr[x][y] != '.') {
        dfs(x, y + 1);
        return;
    }
    long key = 0;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j <= i; j++) {
            key = (key << 1) | (arr[i][j] == '.');
        }
    }
    key ^= key << 13;
    key ^= key >> 7;
    key ^= key << 19;
    for (int i = 0; i < 12; i++) {
        key ^= vis[i] << i;
    }
    if (!svis.insert(key).second) {
        return;
    }
    for (auto& a : shapes) {
        for (auto& b : a) {
            if (vis[b.label]) {
                continue;
            }
            for (int dx = -5; dx <= 5; dx++) {
                for (int dy = -5; dy <= 5; dy++) {
                    bool f = true, filled = false;
                    b.feach(x + dx, y + dy, [&](int i, int j) -> void {
                        f &= ibs(i, j) && arr[i][j] == '.';
                        filled |= i == x && j == y;
                    });
                    if (f && filled) {
                        b.feach(x + dx, y + dy, [&](int i, int j) -> void {
                            arr[i][j] = b.label;
                        });
                        vis[b.label] = true;
                        dfs(x, y + 1);
                        vis[b.label] = false;
                        b.feach(x + dx, y + dy, [&](int i, int j) -> void {
                            arr[i][j] = '.';
                        });
                    }
                }
            }
        }
    }
}

void solve() {
    for (auto& a : shapes[11]) {
        dbg(a);
    }
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    for (auto& a : arr) {
        for (auto& b : a) {
            vis[b] = true;
        }
    }
    dfs(0, 0);
    cout << "No solution" << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}