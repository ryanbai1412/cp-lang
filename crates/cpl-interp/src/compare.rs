//! Comparing and showing values: equality and order by type, formatting, printing.

use super::*;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    pub(super) fn eq_ty_val(&mut self, ty: &Ty, l: &Value, r: &Value, span: Span) -> Res<bool> {
        match ty {
            Ty::Struct(id) => {
                if let Some(eq_fn) = self.program.structs[*id].eq_fn {
                    return Ok(self
                        .call_direct(eq_fn, vec![l.clone(), r.clone()], span)?
                        .bool());
                }
                let fields = self.program.structs[*id].fields.clone();
                let cached = self.program.structs[*id].cached_fields.clone();
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (i, ((_, ft), (lv, rv))) in fields
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                        .enumerate()
                    {
                        if cached.contains(&i) {
                            continue;
                        }
                        if !self.eq_ty_val(ft, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::Tuple(ts) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (t, (lv, rv)) in ts
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        if !self.eq_ty_val(t, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::Record(fs) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for ((_, t), (lv, rv)) in fs
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        if !self.eq_ty_val(t, lv, rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::List(elem) => {
                if let (Value::List(ls), Value::List(rs)) = (l, r) {
                    if ls.len() != rs.len() {
                        return Ok(false);
                    }
                    for (lv, rv) in ls.iter().zip(rs.iter()) {
                        if !self.eq_ty_val(elem, &lv, &rv, span)? {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                }
                Ok(l == r)
            }
            Ty::Opt(inner) => match (l, r) {
                (Value::Opt(OptVal::None), Value::Opt(OptVal::None)) => Ok(true),
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Int(b))) => Ok(a == b),
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.eq_ty_val(inner, a, b, span)
                }
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.eq_ty_val(inner, &Value::Int(*a), b, span)
                }
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Int(b))) => {
                    self.eq_ty_val(inner, a, &Value::Int(*b), span)
                }
                _ => Ok(false),
            },
            _ => Ok(l == r),
        }
    }

    pub(super) fn cmp_ty_val(
        &mut self,
        ty: &Ty,
        l: &Value,
        r: &Value,
        span: Span,
    ) -> Res<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match ty {
            Ty::Struct(id) => {
                if let Some(hook) = self.program.structs[*id].ord {
                    return self.user_order(hook, l, r, span);
                }
                let fields = self.program.structs[*id].fields.clone();
                let cached = self.program.structs[*id].cached_fields.clone();
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (i, ((_, ft), (lv, rv))) in fields
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                        .enumerate()
                    {
                        if cached.contains(&i) {
                            continue;
                        }
                        let o = self.cmp_ty_val(ft, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(Ordering::Equal);
                }
                Ok(l.cmp(r))
            }
            Ty::Tuple(ts) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for (t, (lv, rv)) in ts
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        let o = self.cmp_ty_val(t, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(Ordering::Equal);
                }
                Ok(l.cmp(r))
            }
            Ty::Record(fs) => {
                if let (Value::Tuple(ls), Value::Tuple(rs)) = (l, r) {
                    let (mut sl, mut sr) = ([Value::Unit], [Value::Unit]);
                    for ((_, t), (lv, rv)) in fs
                        .iter()
                        .zip(ls.as_slice(&mut sl).iter().zip(rs.as_slice(&mut sr).iter()))
                    {
                        let o = self.cmp_ty_val(t, lv, rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(Ordering::Equal);
                }
                Ok(l.cmp(r))
            }
            Ty::List(elem) => {
                if let (Value::List(ls), Value::List(rs)) = (l, r) {
                    for (lv, rv) in ls.iter().zip(rs.iter()) {
                        let o = self.cmp_ty_val(elem, &lv, &rv, span)?;
                        if o != Ordering::Equal {
                            return Ok(o);
                        }
                    }
                    return Ok(ls.len().cmp(&rs.len()));
                }
                Ok(l.cmp(r))
            }
            Ty::Opt(inner) => match (l, r) {
                (Value::Opt(OptVal::None), Value::Opt(OptVal::None)) => Ok(Ordering::Equal),
                (Value::Opt(OptVal::None), _) => Ok(Ordering::Less),
                (_, Value::Opt(OptVal::None)) => Ok(Ordering::Greater),
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Int(b))) => Ok(a.cmp(b)),
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.cmp_ty_val(inner, a, b, span)
                }
                (Value::Opt(OptVal::Int(a)), Value::Opt(OptVal::Boxed(b))) => {
                    self.cmp_ty_val(inner, &Value::Int(*a), b, span)
                }
                (Value::Opt(OptVal::Boxed(a)), Value::Opt(OptVal::Int(b))) => {
                    self.cmp_ty_val(inner, a, &Value::Int(*b), span)
                }
                _ => Ok(l.cmp(r)),
            },
            _ => Ok(l.cmp(r)),
        }
    }

    /// Renders a format string; `vals` holds one value per non-text piece, in order.
    pub(super) fn format_values(
        &mut self,
        pieces: &'p [FmtPiece],
        vals: &[Value],
        span: Span,
    ) -> Res<Value> {
        let mut text = Vec::new();
        let mut vals = vals.iter();
        for piece in pieces {
            match piece {
                FmtPiece::Text(bytes) => text.extend_from_slice(bytes),
                FmtPiece::Value(FmtArg { value, digits }) => {
                    let Some(v) = vals.next() else {
                        return trap(span, cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[]));
                    };
                    self.show_val(&mut text, v, &value.ty, *digits, span)?;
                }
                FmtPiece::Number(_, fmt) => match vals.next() {
                    Some(Value::Int(v)) => text.extend(fmt.int(*v).bytes()),
                    Some(Value::Float(v)) => text.extend(fmt.float(*v).bytes()),
                    _ => return trap(span, cpl_diag::msgs::interp::INTERPRETER_FAILED.at(&[])),
                },
            }
        }
        Ok(Value::Str(Rc::new(text)))
    }

    pub(super) fn print_values(
        &mut self,
        args: &'p [FmtArg],
        vals: &[Value],
        end: Option<&Value>,
        span: Span,
    ) -> Res<()> {
        let start = self.out.len();
        let sparse = args.iter().any(|arg| arg.value.ty.may_show_nothing());
        for (i, (arg, value)) in args.iter().zip(vals).enumerate() {
            let mut buf = Vec::new();
            self.show_val(&mut buf, value, &arg.value.ty, arg.digits, span)?;
            // An empty collection shows nothing and takes no space.
            if buf.is_empty() && arg.value.ty.may_show_nothing() {
                continue;
            }
            if i > 0 && (self.out.len() > start || !sparse) {
                self.out.push(b' ');
            }
            self.out.extend_from_slice(&buf);
        }
        match end {
            Some(end) => {
                let mut buf = Vec::new();
                self.show_val(&mut buf, end, &Ty::Str, 6, span)?;
                self.out.extend_from_slice(&buf);
            }
            None => self.out.push(b'\n'),
        }
        if self.observed() {
            let text = String::from_utf8_lossy(&self.out[start.min(self.out.len())..]).into_owned();
            self.traced(|tracer, view| {
                tracer.on(crate::observe::Event::Printed {
                    view,
                    at: span,
                    text: &text,
                });
            });
        }
        if self.program.interactive {
            self.write_out(true);
        } else {
            self.flush_if_large();
        }
        Ok(())
    }

    pub(super) fn show_val(
        &mut self,
        out: &mut Vec<u8>,
        value: &Value,
        ty: &Ty,
        digits: u32,
        span: Span,
    ) -> Res<()> {
        // Printed once: an empty list keeps its element type, so asking
        // again would find the same hook and never stop.
        if let Some((parts, ty)) = self.shown_parts(value, ty, span)? {
            show(out, &parts, &ty, digits, self.program, &self.domains);
            return Ok(());
        }
        if let Value::MutView(mv) = value {
            let items = Value::list(self.mut_view_to_vec(mv, span)?);
            show(out, &items, ty, digits, self.program, &self.domains);
            return Ok(());
        }
        show(out, value, ty, digits, self.program, &self.domains);
        Ok(())
    }
}

pub(super) fn compare(op: CmpOp, l: &Value, r: &Value) -> bool {
    // Floats compare as IEEE says, so a NaN is neither less nor equal.
    if let (Value::Float(a), Value::Float(b)) = (l, r) {
        return match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
            _ => false,
        };
    }
    let order = l.cmp(r);
    match op {
        CmpOp::Eq => order == Ordering::Equal,
        CmpOp::Ne => order != Ordering::Equal,
        CmpOp::Lt => order == Ordering::Less,
        CmpOp::Le => order != Ordering::Greater,
        CmpOp::Gt => order == Ordering::Greater,
        CmpOp::Ge => order != Ordering::Less,
        _ => false,
    }
}
