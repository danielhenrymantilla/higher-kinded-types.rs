use ::core::{any::TypeId, cell::Cell};

pub
trait Apply<'lt> : 'static {
    type Applied;
}

impl Apply<'_> for String {
    type Applied = Self;
}

impl Apply<'_> for i32 {
    type Applied = Self;
}

impl<'lt, T : Apply<'lt>> Apply<'lt> for &'static T {
    type Applied = &'lt T::Applied;
}

impl<'lt, T : Apply<'lt>> Apply<'lt> for Cell<&'static T> {
    type Applied = Cell<&'lt T::Applied>;
}

/// # Safety
pub
trait LtInfected<'lt> : 'lt {
    type StaticSelf : Apply<'lt, Applied = Self>;

    fn static_type_id() -> TypeId {
        TypeId::of::<Self::StaticSelf>()
    }
}

impl LtInfected<'_> for String {
    type StaticSelf = Self;
}

impl LtInfected<'_> for i32 {
    type StaticSelf = Self;
}

impl<'lt, T : LtInfected<'lt>> LtInfected<'lt> for &'lt T {
    type StaticSelf = &'static T::StaticSelf;
}

impl<'lt, T : LtInfected<'lt>> LtInfected<'lt> for Cell<&'lt T> {
    type StaticSelf = Cell<&'static T::StaticSelf>;
}

mod seal {
    use super::*;

    pub trait Sealed<'lt> {}
    impl<'lt, T : LtInfected<'lt>> Sealed<'lt> for T {}
}

pub
trait MyAny<'lt> : seal::Sealed<'lt> {
    fn dyn_static_type_id(&self) -> TypeId;
}

impl<'lt, T : LtInfected<'lt>> MyAny<'lt> for T {
    fn dyn_static_type_id(&self) -> TypeId {
        T::static_type_id()
    }
}

impl<'lt> dyn 'lt + MyAny<'lt> {
    pub
    fn is<T : LtInfected<'lt>>(&self) -> bool {
        self.dyn_static_type_id() == T::static_type_id()
    }

    pub
    fn downcast_ref<'r, T : LtInfected<'lt>>(
        self: &'r (dyn 'lt + MyAny<'lt>),
    ) -> Option<&'r T>
    {
        self.is::<T>().then(|| unsafe {
            &*(self as *const Self as *const T)
        })
    }

    pub
    fn downcast_owned<T : LtInfected<'lt>>(
        self: Box<dyn 'lt + MyAny<'lt>>,
    ) -> Option<T>
    {
        self.is::<T>().then(|| unsafe {
            *Box::from_raw(Box::into_raw(self) as *mut Self as *mut T)
        })
    }
}

fn coerce<'lt, T : LtInfected<'lt>>(
    it: T,
) -> Box<dyn 'lt + MyAny<'lt>>
{
    // Look: no unsafe!
    Box::new(it) as _
}

/// Helper to avoid coherence issues
pub
struct Static<T : 'static>(
    pub T,
);
impl<T : 'static> Apply<'_> for Static<T> {
    type Applied = Self;
}
impl<T : 'static> LtInfected<'_> for Static<T> {
    type StaticSelf = Self;
}

fn main() {
    let mut any; // single var to prove they all have the same `dyn`-erased type.

    let x: i32 = 42;
    any = coerce(x);
    assert_eq!(any.is::<i32>(), true);
    assert_eq!(any.is::<String>(), false);
    assert_eq!(any.is::<&i32>(), false);
    assert_eq!(any.is::<Cell<&i32>>(), false);
    assert_eq!(any.downcast_owned::<i32>(), Some(42));

    let s: String = "42".into();
    any = coerce(s);
    assert_eq!(any.is::<i32>(), false);
    assert_eq!(any.is::<String>(), true);
    assert_eq!(any.is::<&i32>(), false);
    assert_eq!(any.is::<Cell<&i32>>(), false);
    assert_eq!(any.downcast_owned::<String>().as_deref(), Some("42"));

    let local = 42;
    let r: &'_ i32 = &local;
    any = coerce(r);
    assert_eq!(any.is::<i32>(), false);
    assert_eq!(any.is::<String>(), false);
    assert_eq!(any.is::<&i32>(), true);
    assert_eq!(any.is::<Cell<&i32>>(), false);
    assert_eq!(any.downcast_owned::<&i32>(), Some(&42));

    let c: Cell<&'_ i32> = Cell::new(&local);
    any = coerce(c);
    assert_eq!(any.is::<i32>(), false);
    assert_eq!(any.is::<String>(), false);
    assert_eq!(any.is::<&i32>(), false);
    assert_eq!(any.is::<Cell<&i32>>(), true);
    assert_eq!(any.downcast_owned::<Cell<&i32>>().map(|c| c.get()), Some(&42));

    // this one works thanks to covariance: we have a `&'r &'local i32`, which
    // subtypes `&'r &'r i32`, which is compatible with `Box<dyn 'r + MyAny<'r>>`
    let nested_r: &&i32 = &r;
    any = coerce(nested_r);
    assert!(matches!(any.downcast_owned::<&&i32>(), Some(42)));

    // Thanks to the `Static` coherence-but-also-distinct-TypeId newtype wrapper,
    // these `&'static` references are "tagged" within their `TypeId` so as to
    // make them distinguishable from the `r: &'local i32 <: &'r i32` above.
    // This is why we are abvec
    let mut static_r: &'static i32 = &42;
    any = coerce(Static(static_r));
    // notice how, despite the `'r`-infected `Any`, we are still capable of
    // extracting a fully `'static` type out of it!
    static_r = any.downcast_owned::<Static<&i32>>().unwrap().0;
    assert_eq!(static_r, &42);

    println!("✅");
}
