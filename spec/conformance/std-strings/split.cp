// expect: ok, output "YES"
use std.strings.{split, split_once, split_whitespace}
let e: List<Str> = []
print(yes("a,b,,c".split(',') == ["a", "b", "", "c"] and "a,".split(',') == ["a", ""] and "".split(',') == [""] and "key=val=x".split_once('=') == Some(("key", "val=x")) and "abc".split_once('=') == None and " a  b\tc ".split_whitespace() == ["a", "b", "c"] and "   ".split_whitespace() == e))
