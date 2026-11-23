// expect: ok, output "a YES YES ab"
// `Char` is a domain of 128 chars: `Char.from_idx(i)` is the char numbered `i`, `None` outside `0..Char.K`, in a generic body too.
fn letters<A>(idx: List<Int>) -> List<A> = [A.try_at(i)! for i in idx]
print(Char.try_at(97)!, yes(Char.try_at(128) == None), yes(Char.try_at(-1) == None), Str(letters<Char>([97, 98])))
