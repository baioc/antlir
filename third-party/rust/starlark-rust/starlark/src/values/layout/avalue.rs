/*
 * Copyright 2019 The Starlark in Rust Authors.
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{
    codemap::Span,
    environment::Globals,
    eval::{Arguments, Evaluator},
    values::{
        docs::DocItem, layout::arena::AValuePtr, none::NoneType, string::StarlarkStr, ComplexValue,
        Freezer, FrozenValue, Heap, SimpleValue, StarlarkValue, Tracer, Value,
    },
};
use gazebo::{any::AnyLifetime, cast, coerce::Coerce};
use std::{
    any::TypeId,
    cmp::Ordering,
    fmt::{self, Debug},
    mem,
    ptr::metadata,
};

pub(crate) static VALUE_NONE: &AValuePtr = {
    const PAYLOAD: Wrapper<Basic, NoneType> = Wrapper(Basic, NoneType);
    const DYN: &dyn AValue<'static> = &PAYLOAD;
    static DATA: (AValuePtr, Wrapper<Basic, NoneType>) = (AValuePtr(metadata(DYN)), PAYLOAD);
    &DATA.0
};

pub(crate) static VALUE_FALSE: &AValuePtr = {
    const PAYLOAD: Wrapper<Basic, bool> = Wrapper(Basic, false);
    const DYN: &dyn AValue<'static> = &PAYLOAD;
    static DATA: (AValuePtr, Wrapper<Basic, bool>) = (AValuePtr(metadata(DYN)), PAYLOAD);
    &DATA.0
};

pub(crate) static VALUE_TRUE: &AValuePtr = {
    const PAYLOAD: Wrapper<Basic, bool> = Wrapper(Basic, true);
    const DYN: &dyn AValue<'static> = &PAYLOAD;
    static DATA: (AValuePtr, Wrapper<Basic, bool>) = (AValuePtr(metadata(DYN)), PAYLOAD);
    &DATA.0
};

pub(crate) const VALUE_STR_A_VALUE_PTR: AValuePtr = {
    #[allow(clippy::declare_interior_mutable_const)]
    const VTABLE: Wrapper<Direct, StarlarkStr> = Wrapper(Direct, unsafe { StarlarkStr::new(0) });
    AValuePtr(metadata(
        &VTABLE as *const Wrapper<Direct, StarlarkStr> as *const dyn AValue<'static>,
    ))
};

/// A trait that covers [`StarlarkValue`].
/// If you need a real [`StarlarkValue`] see [`AsStarlarkValue`](crate::values::AsStarlarkValue).
pub trait AValue<'v>: StarlarkValue<'v> {
    #[doc(hidden)]
    // How much memory I take up on the heap.
    // Included to allow unsized types to live on the heap.
    fn memory_size(&self) -> usize;

    #[doc(hidden)]
    fn heap_freeze(&self, me: &AValuePtr, freezer: &Freezer) -> anyhow::Result<FrozenValue>;

    #[doc(hidden)]
    fn heap_copy(&self, me: &AValuePtr, tracer: &Tracer<'v>) -> Value<'v>;

    fn unpack_str(&self) -> Option<&str> {
        self.unpack_starlark_str().map(|x| x.unpack())
    }

    fn unpack_starlark_str(&self) -> Option<&StarlarkStr>;
}

impl<'v> dyn AValue<'v> {
    /// Downcast a reference to type `T`, or return [`None`](None) if it is not the
    /// right type.
    // We'd love to reuse the type from as_dyn_any, but that doesn't seem to have the right vtable-ness
    pub fn downcast_ref<T: AnyLifetime<'v>>(&self) -> Option<&T> {
        if self.static_type_of() == T::static_type_id() {
            // SAFETY: just checked whether we are pointing to the correct type.
            unsafe { Some(&*(self as *const Self as *const T)) }
        } else {
            None
        }
    }
}

pub(crate) fn starlark_str(len: usize) -> impl AValue<'static> + Send + Sync {
    Wrapper(Direct, unsafe { StarlarkStr::new(len) })
}

pub(crate) fn basic_ref<'v, T: StarlarkValue<'v>>(x: &T) -> &dyn AValue<'v> {
    // These are the same representation, so safe to convert
    let x: &Wrapper<Basic, T> = unsafe { cast::ptr(x) };
    x
}

pub(crate) fn simple(x: impl SimpleValue) -> impl AValue<'static> + Send + Sync {
    Wrapper(Simple, x)
}

pub(crate) fn complex<'v>(x: impl ComplexValue<'v>) -> impl AValue<'v> {
    Wrapper(Complex, x)
}

// A type where the second element is in control of what instances are in scope
struct Direct;

// A type that implements StarlarkValue but nothing else, so will never be stored
// in the heap (e.g. bool, None)
struct Basic;

// A type that implements SimpleValue.
struct Simple;

// A type that implements ComplexValue.
struct Complex;

// We want to define several types (Simple, Complex) that wrap a StarlarkValue,
// reimplement it, and do some things custom. The easiest way to avoid repeating
// the StarlarkValue trait each time is to make them all share a single wrapper,
// where Mode is one of Simple/Complex.
#[repr(C)]
struct Wrapper<Mode, T>(Mode, T);

// Safe because Simple/Complex are ZST
unsafe impl<T> Coerce<T> for Wrapper<Simple, T> {}
unsafe impl<T> Coerce<T> for Wrapper<Complex, T> {}

/// The overwrite operation in the heap requires that the LSB not be set.
/// For FrozenValue this is the case, but for Value the LSB is always set.
/// Fortunately, the consumer of the overwritten value reapplies the
/// FrozenValue/Value tags, so we can freely discard it here.
fn clear_lsb(x: usize) -> usize {
    x & !1
}

impl<'v, T: StarlarkValue<'v>> AValue<'v> for Wrapper<Basic, T> {
    fn memory_size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn heap_freeze(&self, _me: &AValuePtr, _freezer: &Freezer) -> anyhow::Result<FrozenValue> {
        unreachable!("Basic types don't appear in the heap")
    }
    fn heap_copy(&self, _me: &AValuePtr, _tracer: &Tracer<'v>) -> Value<'v> {
        unreachable!("Basic types don't appear in the heap")
    }

    fn unpack_starlark_str(&self) -> Option<&StarlarkStr> {
        None
    }
}

impl<'v> AValue<'v> for Wrapper<Direct, StarlarkStr> {
    fn memory_size(&self) -> usize {
        mem::size_of::<StarlarkStr>() + self.1.len()
    }

    fn heap_freeze(&self, me: &AValuePtr, freezer: &Freezer) -> anyhow::Result<FrozenValue> {
        let s = self.1.unpack();
        let fv = freezer.alloc(s);
        unsafe {
            me.overwrite::<Self>(fv.0.ptr_value() & !1)
        };
        Ok(fv)
    }

    fn heap_copy(&self, me: &AValuePtr, tracer: &Tracer<'v>) -> Value<'v> {
        let s = self.1.unpack();
        let v = tracer.alloc_str(s);
        unsafe {
            me.overwrite::<Self>(v.0.ptr_value() & !1)
        };
        v
    }

    fn unpack_starlark_str(&self) -> Option<&StarlarkStr> {
        Some(&self.1)
    }
}

impl<'v, T: SimpleValue> AValue<'v> for Wrapper<Simple, T>
where
    'v: 'static,
{
    fn memory_size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn heap_freeze(&self, me: &AValuePtr, freezer: &Freezer) -> anyhow::Result<FrozenValue> {
        let (fv, r) = freezer.reserve::<Self>();
        let x = unsafe { me.overwrite::<Self>(clear_lsb(fv.0.ptr_value())) };
        r.fill(x);
        Ok(fv)
    }

    fn heap_copy(&self, me: &AValuePtr, tracer: &Tracer<'v>) -> Value<'v> {
        let (v, r) = tracer.reserve::<Self>();
        let x = unsafe { me.overwrite::<Self>(clear_lsb(v.0.ptr_value())) };
        r.fill(x);
        v
    }

    fn unpack_starlark_str(&self) -> Option<&StarlarkStr> {
        None
    }
}

impl<'v, T: ComplexValue<'v>> AValue<'v> for Wrapper<Complex, T> {
    fn memory_size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn heap_freeze(&self, me: &AValuePtr, freezer: &Freezer) -> anyhow::Result<FrozenValue> {
        let (fv, r) = freezer.reserve::<Wrapper<Simple, T::Frozen>>();
        let x = unsafe { me.overwrite::<Self>(clear_lsb(fv.0.ptr_value())) };
        let res = x.1.freeze(freezer)?;
        r.fill(simple(res));
        Ok(fv)
    }

    fn heap_copy(&self, me: &AValuePtr, tracer: &Tracer<'v>) -> Value<'v> {
        let (v, r) = tracer.reserve::<Self>();
        let mut x = unsafe { me.overwrite::<Self>(clear_lsb(v.0.ptr_value())) };
        // We have to put the forwarding node in _before_ we trace in case there are cycles
        x.1.trace(tracer);
        r.fill(x);
        v
    }

    fn unpack_starlark_str(&self) -> Option<&StarlarkStr> {
        None
    }
}

impl<Mode, T: Debug> Debug for Wrapper<Mode, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(f)
    }
}

// This is somewhat dubious. We can't define Wrapper<T> to be AnyLifetime, since these things
// don't compose properly. We do layout Wrapper such that it is really a T underneath,
// so we can pretend it really is just a T.
unsafe impl<'v, Mode: 'static, T: AnyLifetime<'v>> AnyLifetime<'v> for Wrapper<Mode, T> {
    fn static_type_id() -> TypeId
    where
        Self: Sized,
    {
        T::static_type_id()
    }

    fn static_type_of(&self) -> std::any::TypeId {
        Self::static_type_id()
    }
}

#[derive(Debug, AnyLifetime)]
pub(crate) struct BlackHole(pub(crate) usize);

impl<'v> AValue<'v> for BlackHole {
    fn memory_size(&self) -> usize {
        self.0
    }

    fn heap_freeze(&self, _me: &AValuePtr, _freezer: &Freezer) -> anyhow::Result<FrozenValue> {
        unreachable!()
    }
    fn heap_copy(&self, _me: &AValuePtr, _tracer: &Tracer<'v>) -> Value<'v> {
        unreachable!()
    }
    fn unpack_starlark_str(&self) -> Option<&StarlarkStr> {
        unreachable!()
    }
}

impl<'v> StarlarkValue<'v> for BlackHole {
    starlark_type!("BlackHole");
}

impl<'v, Mode: 'static, T: StarlarkValue<'v>> StarlarkValue<'v> for Wrapper<Mode, T> {
    fn get_type(&self) -> &'static str {
        self.1.get_type()
    }
    fn get_type_value(&self) -> FrozenValue {
        self.1.get_type_value()
    }
    fn matches_type(&self, ty: &str) -> bool {
        self.1.matches_type(ty)
    }
    fn get_methods(&self) -> Option<&'static Globals> {
        self.1.get_methods()
    }
    fn collect_repr(&self, collector: &mut String) {
        self.1.collect_repr(collector)
    }
    fn to_json(&self) -> anyhow::Result<String> {
        self.1.to_json()
    }
    fn to_bool(&self) -> bool {
        self.1.to_bool()
    }
    fn to_int(&self) -> anyhow::Result<i32> {
        self.1.to_int()
    }
    fn get_hash(&self) -> anyhow::Result<u64> {
        self.1.get_hash()
    }
    fn equals(&self, other: Value<'v>) -> anyhow::Result<bool> {
        self.1.equals(other)
    }
    fn compare(&self, other: Value<'v>) -> anyhow::Result<Ordering> {
        self.1.compare(other)
    }
    fn invoke(
        &self,
        me: Value<'v>,
        location: Option<Span>,
        args: Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_>,
    ) -> anyhow::Result<Value<'v>> {
        self.1.invoke(me, location, args, eval)
    }
    fn at(&self, index: Value<'v>, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.at(index, heap)
    }
    fn slice(
        &self,
        start: Option<Value<'v>>,
        stop: Option<Value<'v>>,
        stride: Option<Value<'v>>,
        heap: &'v Heap,
    ) -> anyhow::Result<Value<'v>> {
        self.1.slice(start, stop, stride, heap)
    }
    fn iterate<'a>(
        &'a self,
        heap: &'v Heap,
    ) -> anyhow::Result<Box<dyn Iterator<Item = Value<'v>> + 'a>>
    where
        'v: 'a,
    {
        self.1.iterate(heap)
    }
    fn with_iterator(
        &self,
        heap: &'v Heap,
        f: &mut dyn FnMut(&mut dyn Iterator<Item = Value<'v>>) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.1.with_iterator(heap, f)
    }
    fn length(&self) -> anyhow::Result<i32> {
        self.1.length()
    }
    fn get_attr(&self, attribute: &str, heap: &'v Heap) -> Option<Value<'v>> {
        self.1.get_attr(attribute, heap)
    }
    fn has_attr(&self, attribute: &str) -> bool {
        self.1.has_attr(attribute)
    }
    fn dir_attr(&self) -> Vec<String> {
        self.1.dir_attr()
    }
    fn is_in(&self, other: Value<'v>) -> anyhow::Result<bool> {
        self.1.is_in(other)
    }
    fn plus(&self, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.plus(heap)
    }
    fn minus(&self, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.minus(heap)
    }
    fn radd(&self, lhs: Value<'v>, heap: &'v Heap) -> Option<anyhow::Result<Value<'v>>> {
        self.1.radd(lhs, heap)
    }
    fn add(&self, rhs: Value<'v>, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.add(rhs, heap)
    }
    fn sub(&self, other: Value<'v>, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.sub(other, heap)
    }
    fn mul(&self, other: Value<'v>, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.mul(other, heap)
    }
    fn percent(&self, other: Value<'v>, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.percent(other, heap)
    }
    fn floor_div(&self, other: Value<'v>, heap: &'v Heap) -> anyhow::Result<Value<'v>> {
        self.1.floor_div(other, heap)
    }
    fn bit_and(&self, other: Value<'v>) -> anyhow::Result<Value<'v>> {
        self.1.bit_and(other)
    }
    fn bit_or(&self, other: Value<'v>) -> anyhow::Result<Value<'v>> {
        self.1.bit_or(other)
    }
    fn bit_xor(&self, other: Value<'v>) -> anyhow::Result<Value<'v>> {
        self.1.bit_xor(other)
    }
    fn left_shift(&self, other: Value<'v>) -> anyhow::Result<Value<'v>> {
        self.1.left_shift(other)
    }
    fn right_shift(&self, other: Value<'v>) -> anyhow::Result<Value<'v>> {
        self.1.right_shift(other)
    }
    fn export_as(&self, variable_name: &str, eval: &mut Evaluator<'v, '_>) {
        self.1.export_as(variable_name, eval)
    }
    fn set_at(&self, index: Value<'v>, new_value: Value<'v>) -> anyhow::Result<()> {
        self.1.set_at(index, new_value)
    }
    fn set_attr(&self, attribute: &str, new_value: Value<'v>) -> anyhow::Result<()> {
        self.1.set_attr(attribute, new_value)
    }
    fn documentation(&self) -> Option<DocItem> {
        self.1.documentation()
    }
}
