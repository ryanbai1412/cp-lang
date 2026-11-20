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

constexpr int dknight[8][2] = {{1, 2},   {2, 1},   {2, -1}, {1, -2},
                               {-1, -2}, {-2, -1}, {-2, 1}, {-1, 2}};
constexpr int dking[8][2] = {{0, 1}, {0, -1}, {1, 0},  {-1, 0},
                             {1, 1}, {1, -1}, {-1, 1}, {-1, -1}};

void solve() {
    string s;
    int x1, y1, x2, y2;
    cin >> s >> x1 >> y1 >> x2 >> y2;
    if (x1 == x2 && y1 == y2) {
        cout << 0 << endl;
        return;
    }
    auto s_c = [&](char c) -> bool {
        return s.find(c) != string::npos;
    };
    bool rook = s_c('R') || s_c('Q'), bishop = s_c('B') || s_c('Q'),
         knight = s_c('N'), king = s_c('K'), pawn = s_c('P');
    assert(bishop && !knight && !king && !pawn);
    if (x1 - y1 == x2 - y2 || x1 + y1 == x2 + y2) {
        cout << 1 << endl;
    } else if ((x1 + y1) % 2 == (x2 + y2) % 2) {
        cout << 2 << endl;
    } else {
        cout << -1 << endl;
    }
    return;
    bool one_move = false;
    one_move |= rook && (x1 == x2 || y1 == y2);
    one_move |= bishop && (x1 - y1 == x2 - y2 || x1 + y1 == x2 + y2);
    one_move |= king && (abs(x1 - x2) <= 1 && abs(y1 - y2) <= 1);
    one_move |= pawn && (x1 + 1 == x2);
    if (knight) {
        for (auto& [dx, dy] : dknight) {
            one_move |= x1 + dx == x2 && y1 + dy == y2;
        }
    }
    if (one_move) {
        cout << 1 << endl;
        return;
    }
    if (rook) {
        cout << 2 << endl;
        return;
    }
    if (bishop) {
        if ((x1 + y1) % 2 == (x2 + y2) % 2) {
            cout << 2 << endl;
            return;
        }
        if (!knight && !king && !pawn) {
            cout << -1 << endl;
            return;
        }
        bool two_moves = false;
        auto go = [&](int x, int y) -> void {
            x += x1;
            y += y1;
            two_moves |= x - y == x2 - y2 || x + y == x2 + y2;
        };
        if (knight) {
            for (auto& [dx, dy] : dknight) {
                go(dx, dy);
            }
        }
        if (king) {
            for (auto& [dx, dy] : dking) {
                go(dx, dy);
            }
        }
        if (pawn) {
            go(1, 0);
        }
        if (two_moves) {
            cout << 2 << endl;
        } else {
            cout << 3 << endl;
        }
        return;
    }
    assert(false);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}