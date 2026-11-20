pub use __cargo_equip::prelude::*;

use lib::prelude::*;
use rand::{Rng, SeedableRng};
use std::ops::{Add, AddAssign};

mod alloc {
    pub use crate::__cargo_equip::prelude::*;

    use super::Node;
    use std::{
        marker::PhantomData,
        mem::MaybeUninit,
        ops::{Deref, DerefMut},
    };

    struct Stack<const N: usize> {
        v: [usize; N],
        ptr: usize,
    }

    impl<const N: usize> Stack<N> {
        const fn new() -> Self {
            Self { v: [0; N], ptr: N }
        }

        fn push(&mut self, x: usize) {
            self.v[self.ptr] = x ^ self.ptr;
            self.ptr += 1;
        }

        fn pop(&mut self) -> usize {
            self.ptr -= 1;
            self.v[self.ptr] ^ self.ptr
        }
    }

    struct Alloc<T, const N: usize> {
        heap: [MaybeUninit<T>; N],
        free_inds: Stack<N>,
    }

    impl<T, const N: usize> Alloc<T, N> {
        const fn new() -> Self {
            Self {
                heap: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
                free_inds: Stack::new(),
            }
        }

        fn alloc(&mut self, t: T) -> &mut T {
            self.heap[self.free_inds.pop()].write(t)
        }

        unsafe fn free(&mut self, ptr: *const T) {
            unsafe {
                self.free_inds
                    .push((ptr as *const MaybeUninit<T>).offset_from(self.heap.as_ptr()) as usize);
            }
        }
    }

    static mut HEAP: Alloc<Node, { 1 << 20 }> = Alloc::new();

    pub(super) struct NodeBox {
        v: &'static mut Node,
        _marker: PhantomData<Node>,
    }

    impl NodeBox {
        pub(super) fn new(t: Node) -> Self {
            Self {
                v: unsafe { HEAP.alloc(t) },
                _marker: PhantomData,
            }
        }
    }

    impl Deref for NodeBox {
        type Target = Node;

        fn deref(&self) -> &Self::Target {
            self.v
        }
    }

    impl DerefMut for NodeBox {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.v
        }
    }

    impl Drop for NodeBox {
        fn drop(&mut self) {
            unsafe {
                std::ptr::drop_in_place(self.v as *mut Node);
                HEAP.free(self.v as *const Node);
            }
        }
    }
}

use alloc::NodeBox;

type P = Option<NodeBox>;

const IINF: i32 = 1e9 as i32;

#[derive(Debug, Copy, Clone)]
struct NodeValue {
    sum: i32,
    pref: i32,
    suff: i32,
    v: i32,
}

impl Default for NodeValue {
    fn default() -> Self {
        Self {
            sum: 0,
            pref: -IINF,
            suff: -IINF,
            v: -IINF,
        }
    }
}

impl Add<NodeValue> for NodeValue {
    type Output = NodeValue;

    fn add(self, rhs: NodeValue) -> Self::Output {
        NodeValue {
            sum: self.sum + rhs.sum,
            pref: self.pref.max(self.sum + rhs.pref),
            suff: rhs.suff.max(self.suff + rhs.sum),
            v: self.v.max(rhs.v).max(self.suff + rhs.pref),
        }
    }
}

impl NodeValue {
    fn new(x: i32) -> Self {
        Self {
            sum: x,
            pref: x,
            suff: x,
            v: x,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Lazy {
    set: Option<i32>,
    rev: bool,
}

impl AddAssign<Lazy> for Lazy {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Lazy) {
        if rhs.set.is_some() {
            *self = rhs;
        } else if rhs.rev {
            self.rev ^= true;
        }
    }
}

impl Lazy {
    fn apply(&self, n: &mut NodeValue, siz: i32) {
        if let Some(set) = self.set {
            n.sum = set * siz;
            let v = set.max(set * siz);
            n.pref = v;
            n.suff = v;
            n.v = v;
        } else if self.rev {
            swap(&mut n.pref, &mut n.suff);
        }
    }

    fn new_set(x: i32) -> Self {
        Self {
            set: Some(x),
            rev: false,
        }
    }

    fn new_rev() -> Self {
        Self {
            set: None,
            rev: true,
        }
    }

    fn is_no_op(&self) -> bool {
        self.set.is_none() && !self.rev
    }
}

struct Node {
    v: NodeValue,
    lazy: Lazy,
    x: i32,
    size: i32,
    pri: i32,
    lc: P,
    rc: P,
}

impl Node {
    fn size(n: Option<&Node>) -> i32 {
        n.map(|n| n.size).unwrap_or_default()
    }

    fn v(n: Option<&Node>) -> NodeValue {
        n.map(|n| n.v).unwrap_or_default()
    }
}

fn pushdown(o: &mut Node) {
    if o.lazy.is_no_op() {
        return;
    }
    let go = |c: Option<&mut Node>| {
        let Some(c) = c else {
            return;
        };
        o.lazy.apply(&mut c.v, c.size);
        c.lazy += o.lazy;
    };
    go(o.lc.as_deref_mut());
    go(o.rc.as_deref_mut());
    if let Some(set) = o.lazy.set {
        o.x = set;
    }
    if o.lazy.rev {
        swap(&mut o.lc, &mut o.rc);
    }
    o.lazy = Default::default();
}

fn maintain(o: &mut Node) {
    o.size = Node::size(o.lc.as_deref()) + 1 + Node::size(o.rc.as_deref());
    o.v = Node::v(o.lc.as_deref()) + NodeValue::new(o.x) + Node::v(o.rc.as_deref());
    o.lazy.apply(&mut o.v, o.size);
}

fn split(o: P, ind: i32) -> (P, P) {
    let Some(mut o) = o else {
        return (None, None);
    };
    pushdown(&mut o);
    let lc_size = Node::size(o.lc.as_deref());
    if ind <= lc_size {
        let (l, r) = split(mem::take(&mut o.lc), ind);
        o.lc = r;
        maintain(&mut o);
        (l, Some(o))
    } else {
        let (l, r) = split(mem::take(&mut o.rc), ind - lc_size - 1);
        o.rc = l;
        maintain(&mut o);
        (Some(o), r)
    }
}

fn merge(l: P, r: P) -> P {
    let Some(mut l) = l else {
        return r;
    };
    let Some(mut r) = r else {
        return Some(l);
    };
    if l.pri < r.pri {
        pushdown(&mut l);
        l.rc = merge(mem::take(&mut l.rc), Some(r));
        maintain(&mut l);
        Some(l)
    } else {
        pushdown(&mut r);
        r.lc = merge(Some(l), mem::take(&mut r.lc));
        maintain(&mut r);
        Some(r)
    }
}

fn split3(o: P, ind: i32, len: i32) -> (P, P, P) {
    let (l, r) = split(o, ind);
    let (mid, r) = split(r, len);
    (l, mid, r)
}

fn merge3(l: P, m: P, r: P) -> P {
    merge(merge(l, m), r)
}

fn build(arr: Vec<i32>, rng: &mut impl Rng) -> P {
    fn build(arr: &[i32], rng: &mut impl Rng) -> P {
        if arr.is_empty() {
            return None;
        }

        let mid = arr.len() / 2;
        let lc = build(&arr[..mid], rng);
        let rc = build(&arr[mid + 1..], rng);

        Some(NodeBox::new(Node {
            v: Node::v(lc.as_deref()) + NodeValue::new(arr[mid]) + Node::v(rc.as_deref()),
            lazy: Default::default(),
            x: arr[mid],
            size: Node::size(lc.as_deref()) + 1 + Node::size(rc.as_deref()),
            pri: rng.gen(),
            lc,
            rc,
        }))
    }

    build(&arr, rng)
}

#[derive(Default)]
struct Solver;

impl StandardSolver for Solver {
    fn solve<C: ReadExtConfig>(self, inp: &mut impl ReadExt<C>, out: &mut impl WriteExt) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1412);
        let (n, q) = inp.read::<(i32, i32)>();

        let mut root = build((0..n).map(|_| inp.read::<i32>()).collect(), &mut rng);

        for _ in 0..q {
            let s = inp.read::<String>();

            if s == b"MAX-SUM" {
                writeln!(out, "{}", root.as_ref().unwrap().v.v).unwrap();
                continue;
            }

            let (ind, len) = inp.read::<(i32, i32)>();

            if s == b"INSERT" {
                let (nl, nr) = split(root, ind);
                root = merge3(
                    nl,
                    build((0..len).map(|_| inp.read::<i32>()).collect(), &mut rng),
                    nr,
                );
                continue;
            }

            let ind = ind - 1;
            let (nl, mut nm, nr) = split3(root, ind, len);

            if s == b"DELETE" {
                root = merge(nl, nr);
                continue;
            }

            match &*s {
                b"MAKE-SAME" => {
                    let nm = nm.as_mut().unwrap();
                    nm.lazy += Lazy::new_set(inp.read::<i32>());
                    maintain(nm);
                }
                b"REVERSE" => {
                    let nm = nm.as_mut().unwrap();
                    nm.lazy += Lazy::new_rev();
                    maintain(nm);
                }
                b"GET-SUM" => {
                    writeln!(out, "{}", Node::v(nm.as_deref()).sum).unwrap();
                }
                _ => panic!("unknown query {}", String::from_utf8_lossy(&s)),
            }

            root = merge3(nl, nm, nr);
        }
    }
}

fn main() {
    solver::main::<Solver, io::config::NoOverflowChecks>(ProblemTestcaseType::SingleTest)
}

// The following code was expanded by `cargo-equip`.

///  # Bundled libraries
/// 
///  - `lib 0.1.0 (path+████████████████████████████████████████████████)` published in **missing** licensed under **missing** as `crate::__cargo_equip::crates::lib`
#[cfg_attr(any(), rustfmt::skip)]
#[allow(unused)]
mod __cargo_equip {
    pub(crate) mod crates {
        pub mod lib {pub mod exts{mod nightly{use std::mem::MaybeUninit;pub mod prelude{pub use super::*;}pub trait MaybeUninitExt<T>{#[allow(clippy::missing_safety_doc)]unsafe fn array_assume_init<const N:usize>(array:[MaybeUninit<T>;N])->[T;N];fn uninit_array<const N:usize>()->[MaybeUninit<T>;N];}impl<T>MaybeUninitExt<T>for MaybeUninit<T>{unsafe fn array_assume_init<const N:usize>(array:[MaybeUninit<T>;N])->[T;N]{let ret=unsafe{(&array as*const _ as*const[T;N]).read()};std::mem::forget(array);ret}fn uninit_array<const N:usize>()->[MaybeUninit<T>;N]{unsafe{MaybeUninit::<[MaybeUninit<T>;N]>::uninit().assume_init()}}}}pub use nightly::*;}pub mod io{use super::exts::*;use std::{io::{BufReader,BufWriter,Read,Write},iter::Peekable,mem::MaybeUninit,str::FromStr,};pub mod prelude{pub use super::{ReadExt,ReadExtConfig,StringLine,WriteExt};}pub mod config{pub trait ReadExtConfig{const OVERFLOW_CHECKS:bool=true;}impl ReadExtConfig for(){}pub struct NoOverflowChecks;impl ReadExtConfig for NoOverflowChecks{const OVERFLOW_CHECKS:bool=false;}}pub use config::ReadExtConfig;pub fn from_read<R:Read>(r:R)->impl ReadExt<()>{from_read_with_config::<R,()>(r)}pub fn from_read_with_config<R:Read,C:ReadExtConfig>(r:R)->impl ReadExt<C>{BufReader::with_capacity(1<<16,r).bytes().map(|c|c.unwrap()).peekable()}pub fn stdin()->impl ReadExt<()>{stdin_with_config::<()>()}pub fn stdin_with_config<C:ReadExtConfig>()->impl ReadExt<C>{let stdin=std::io::stdin().lock();from_read_with_config::<_,C>(stdin)}pub fn stdout()->impl WriteExt{BufWriter::with_capacity(1<<16,std::io::stdout().lock())}pub trait Readable{type Output;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output;}pub trait ReadExt<C:ReadExtConfig>{fn next_byte(&mut self)->Option<u8>;fn peek_byte(&mut self)->Option<u8>;fn skip_while<F:FnMut(&u8)->bool>(&mut self,f:F);fn read<T:Readable>(&mut self)->T::Output;fn read_vec<T:Readable>(&mut self,n:usize)->Vec<T::Output>;fn eof(&mut self)->bool;}pub trait WriteExt:Write{}impl<T:Write>WriteExt for T{}impl<C:ReadExtConfig,I:Iterator<Item=u8>>ReadExt<C>for Peekable<I>{fn next_byte(&mut self)->Option<u8>{self.next()}fn peek_byte(&mut self)->Option<u8>{self.peek().copied()}fn skip_while<F:FnMut(&u8)->bool>(&mut self,mut f:F){while self.next_if(|c|f(c)).is_some(){}}fn read<T:Readable>(&mut self)->T::Output{T::read_from::<C>(self)}fn read_vec<T:Readable>(&mut self,n:usize)->Vec<T::Output>{(0..n).map(|_|<Self as ReadExt<C>>::read::<T>(self)).collect()}fn eof(&mut self)->bool{ReadExt::<C>::skip_while(self,u8::is_ascii_whitespace);<Self as ReadExt<C>>::peek_byte(self).is_none()}}impl Readable for char{type Output=u8;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);r.next_byte().expect("eof")}}impl Readable for String{type Output=Vec<u8>;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);let mut buf=vec![];while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}buf.push(c);}assert!(!buf.is_empty(),"eof");buf}}macro_rules!uint_readable_impl{($ty:ty)=>{impl Readable for$ty{type Output=$ty;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);r.peek_byte().expect("eof");let mut ans=0;while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}assert!(c.is_ascii_digit());let c=(c-b'0')as$ty;if C::OVERFLOW_CHECKS{if ans<=(<$ty>::MAX-c)/10{ans=ans*10+c;}else{panic!(concat!("overflow when reading ",stringify!($ty)))}}else{ans=ans*10+c;}}ans}}};($($ty:ty),*)=>{$(uint_readable_impl!($ty);)*};}macro_rules!sint_readable_impl{($ty:ty)=>{impl Readable for$ty{type Output=$ty;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);if r.peek_byte().expect("eof")==b'-'{r.next_byte();let mut ans=0;while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}assert!(c.is_ascii_digit());let c=(c-b'0')as$ty;if C::OVERFLOW_CHECKS{if(<$ty>::MIN+c)/10<=ans{ans=ans*10-c;}else{panic!(concat!("underflow when reading ",stringify!($ty)))}}else{ans=ans*10-c;}}ans}else{let mut ans=0;while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}assert!(c.is_ascii_digit());let c=(c-b'0')as$ty;if C::OVERFLOW_CHECKS{if ans<=(<$ty>::MAX-c)/10{ans=ans*10+c;}else{panic!(concat!("overflow when reading ",stringify!($ty)))}}else{ans=ans*10+c;}}ans}}}};($($ty:ty),*)=>{$(sint_readable_impl!($ty);)*};}macro_rules!float_readable_impl{($ty:ty)=>{impl Readable for$ty{type Output=$ty;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{let mut buf=[0;128];let mut buf_ind=0;r.skip_while(u8::is_ascii_whitespace);while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}buf[buf_ind]=c;buf_ind+=1;}<$ty>::from_str(unsafe{std::str::from_utf8_unchecked(&buf[..buf_ind])}).expect(concat!("error parsing ",stringify!($ty)))}}};($($ty:ty),*)=>{$(float_readable_impl!($ty);)*};}uint_readable_impl!(u8,u16,u32,u64,u128,usize);sint_readable_impl!(i8,i16,i32,i64,i128,isize);float_readable_impl!(f32,f64);impl<T:Readable,const N:usize>Readable for[T;N]{type Output=[T::Output;N];fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{let mut arr=MaybeUninit::uninit_array::<N>();for i in&mut arr{i.write(r.read::<T>());}unsafe{MaybeUninit::array_assume_init(arr)}}}pub struct StringLine;impl Readable for StringLine{type Output=Vec<u8>;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.peek_byte().expect("eof");let mut s=vec![];while let Some(c)=r.next_byte(){if c==b'\n'{break;}s.push(c);}if let Some(&b'\r')=s.last(){s.pop();}s}}macro_rules!tuple_readable_impl{()=>{};(@impl$($t:ident)*)=>{impl<$($t:Readable),*>Readable for($($t),*,){type Output=($($t::Output),*,);fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{($(r.read::<$t>()),*,)}}};($head:ident$($tail:tt)*)=>{tuple_readable_impl!(@impl$head$($tail)*);tuple_readable_impl!($($tail)*);};}tuple_readable_impl!(T1 T2 T3 T4 T5 T6 T7 T8);}pub mod solver{use crate::__cargo_equip::crates::lib::io::{self,ReadExt,ReadExtConfig,WriteExt};pub mod prelude{pub use super::{ProblemTestcaseType,StandardSolver};}#[derive(Debug,Copy,Clone,Eq,PartialEq)]pub enum ProblemTestcaseType{SingleTest,MultiTest,MultiTestUntilEof,}pub trait StandardSolver:Default{fn solve<C:ReadExtConfig>(self,inp:&mut impl ReadExt<C>,out:&mut impl WriteExt);fn solve_tc<C:ReadExtConfig>(self,#[allow(unused)]tc_id:u32,inp:&mut impl ReadExt<C>,out:&mut impl WriteExt,){self.solve(inp,out);}}pub fn main<S:StandardSolver,C:ReadExtConfig>(tc_type:ProblemTestcaseType){let mut inp=io::stdin_with_config::<C>();let mut out=io::stdout();match tc_type{ProblemTestcaseType::SingleTest=>{S::default().solve(&mut inp,&mut out);}ProblemTestcaseType::MultiTest=>{let tcs=inp.read::<u32>();for tc in 1..=tcs{S::default().solve_tc(tc,&mut inp,&mut out);}assert!(inp.eof(),"multitest didn't read all input");}ProblemTestcaseType::MultiTestUntilEof=>{for tc in 1..{if inp.eof(){break;}S::default().solve_tc(tc,&mut inp,&mut out);}}}}}pub mod prelude{pub use super::{exts::prelude::*,io::{self,prelude::*},solver::{self,prelude::*},};pub use std::mem::{self,swap};}}
    }

    pub(crate) mod macros {
        pub mod lib {}
    }

    pub(crate) mod prelude {pub use crate::__cargo_equip::crates::*;}

    mod preludes {
        pub mod lib {}
    }
}