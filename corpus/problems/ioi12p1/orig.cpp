// Not a judge-accepted submission (the accepted ones are odometer programs, not C++): written for the
// corpus. Reads a subtask (1-5) and prints an odometer program for it, each checked by checker.py.
// 1: take a pebble from (0,0) then (0,1) in turn until one cell runs out, and stop there.
// 2: the same, parking the pebbles in row 1 and putting them back once one cell runs out.
// 3: walk the two pebbles towards each other one cell at a time until they share a cell.
// 4: sweep each column southwards, carrying each pebble up to row 0, then carry row 0 west to (0,0).
// 5: for v = 0, 1, ..., snake over the grid testing each cell for exactly v pebbles (v gets, a pebble
//    test, v puts), halting at the first; every cell holds at least v once levels below v failed.
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

const char* COLLECT = R"(right
right
A:
move
S:
pebble G
border D
move
jump S
G:
get
left
left
U1:
border P1
move
jump U1
P1:
put
left
left
move
jump S
D:
left
left
U2:
border T
move
jump U2
T:
right
border B
move
right
jump A
B:
left
left
W:
border H
move
jump W
H:
right
right
R:
move
R2:
pebble Q
border Z
move
jump R2
Q:
get
left
left
W2:
border H2
move
jump W2
H2:
put
right
right
jump R
Z:
halt
)";

string minimum() {
    string s = "right\n";
    auto rep = [](const string& cmd, int v) {
        string r;
        for (int i = 0; i < v; i++) r += cmd + "\n";
        return r;
    };
    for (int v = 0; v < 16; v++) {
        string n = to_string(v);
        auto test = [&](const string& side) {
            return rep("get", v) + "pebble More" + side + n + "\n" + rep("put", v) + "halt\nMore" + side + n +
                   ":\n" + rep("put", v);
        };
        s += "E" + n + ":\n" + test("E") + "border TE" + n + "\nmove\njump E" + n + "\n";
        s += "TE" + n + ":\nright\nborder End" + n + "\nmove\nright\n";
        s += "W" + n + ":\n" + test("W") + "border TW" + n + "\nmove\njump W" + n + "\n";
        s += "TW" + n + ":\nleft\nborder End" + n + "\nmove\nleft\njump E" + n + "\n";
        s += "End" + n + ":\nleft\nleft\nUp" + n + ":\nborder Home" + n + "\nmove\njump Up" + n + "\nHome" + n +
             ":\nright\n";
    }
    return s;
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
        cout << COLLECT;
    } else {
        cout << minimum();
    }
}
