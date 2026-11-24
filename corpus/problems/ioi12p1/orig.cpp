// Not a judge-accepted submission (the accepted ones are odometer programs, not C++): written for the
// corpus. Reads a subtask (1-5) and prints an odometer program for it, each checked by checker.py.
// 1: take a pebble from (0,0) then (0,1) in turn until one cell runs out, and stop there.
// 2: the same, parking the pebbles in row 1 and putting them back once one cell runs out.
// 3: walk the two pebbles towards each other one cell at a time until they share a cell.
// 4, 5: see collect() and minimum(); 5 follows the IOI 2012 solution booklet's construction.
#include <bits/stdc++.h>
using namespace std;

const char* COMPARE = R"(right
L:
pebble A
halt
A:
get
move
pebble B
halt
B:
get
left
left
move
right
right
jump L
)";

const char* PAIR = R"(right
L:
pebble A
jump XLE
A:
get
right
move
put
right
right
move
right
move
pebble B
jump YLT
B:
get
right
move
put
right
right
move
left
move
right
right
jump L
XLE:
@a
move
@b
left
left
move
halt
YLT:
@c
left
left
move
right
right
@d
move
halt
)";

// At (0,c) facing east: put the pebbles parked in (1,c) back into (0,c), ending as it began.
string restore(const string& t) {
    return "right\nmove\nR" + t + ":\npebble G" + t + "\njump D" + t + "\nG" + t +
           ":\nget\nright\nright\nmove\nput\nright\nright\nmove\njump R" + t + "\nD" + t +
           ":\nright\nright\nmove\nright\n";
}

const char* MIDPOINT = R"(right
F:
pebble P1
move
jump F
P1:
get
move
pebble M
put
SR:
move
pebble P2
jump SR
P2:
get
left
left
move
pebble M
put
SL:
move
pebble P3
jump SL
P3:
right
right
jump P1
M:
put
halt
)";

// At (0,0) facing north: go down column 0, then sweep rows 255..1 alternately east and west,
// K cells per loop round (each cell tested after the move onto it), pushing each pebble found one row north; row 0 is swept west carrying
// the pebbles in the program counter, and they are put down at (0,0).
const int K = 16;
string collect() {
    string s = "right\nright\nDown:\nborder Bottom\nmove\njump Down\nBottom:\nleft\n";
    auto sweep = [&](const string& d, const string& toNorth, const string& back) {
        s += "Q" + d + ":\npebble F" + d + "\nL" + d + ":\n";
        for (int i = 0; i < K; i++) s += "move\npebble F" + d + "\n";
        s += "border T" + d + "\njump L" + d + "\n";
        s += "F" + d + ":\nget\n" + toNorth + "\nmove\nput\n" + back + "\n" + back + "\nmove\n" + toNorth +
             "\njump Q" + d + "\n";
    };
    sweep("E", "left", "right");
    s += "TE:\nleft\nmove\nborder Row0\nleft\njump QW\n";
    sweep("W", "right", "left");
    s += "TW:\nright\nmove\nright\njump QE\nRow0:\nleft\n";
    for (int k = 0; k < 15; k++) {
        string n = to_string(k);
        s += "C" + n + ":\npebble G" + n + "\nborder P" + n + "\nmove\njump C" + n + "\nG" + n + ":\nget\n";
    }
    s += "C15:\nborder P15\nmove\njump C15\n";
    for (int k = 15; k > 0; k--) s += "P" + to_string(k) + ":\nput\n";
    return s + "P0:\n";
}

// For v = 0..14, sweep the grid row by row testing each cell for exactly v pebbles (v gets, a
// pebble test, v puts); each sweep turns the grid half round, so the next starts where it ended.
// A hit puts its v pebbles back through a shared chain; with no hit every cell holds 15.
string minimum() {
    string s = "jump Scan0\n";
    auto rep = [](const string& cmd, int v) {
        string r;
        for (int i = 0; i < v; i++) r += cmd + "\n";
        return r;
    };
    for (int v = 0; v < 15; v++) {
        string n = to_string(v), m = to_string(v + 1);
        s += "Next" + n + ":\nright\nborder Scan" + m + "\nmove\nright\nBack" + n + ":\nborder Edge" + n +
             "\nmove\njump Back" + n + "\nEdge" + n + ":\nright\n";
        s += "Scan" + n + ":\nright\nTest" + n + ":\n" + rep("get", v) + "pebble More" + n + "\njump Hit" + n +
             "\nMore" + n + ":\n" + rep("put", v) + "border Next" + n + "\nmove\njump Test" + n + "\n";
    }
    for (int v = 14; v > 0; v--) s += "Hit" + to_string(v) + ":\nput\n";
    return s + "Hit0:\nScan15:\n";
}

int main() {
    int task;
    cin >> task;
    if (task == 1) {
        cout << COMPARE;
    } else if (task == 2) {
        string s = PAIR;
        for (char t : string("abcd")) {
            size_t at = s.find(string("@") + t);
            s.replace(at, 3, restore(string(1, t)));
        }
        cout << s;
    } else if (task == 3) {
        cout << MIDPOINT;
    } else if (task == 4) {
        cout << collect();
    } else {
        cout << minimum();
    }
}
