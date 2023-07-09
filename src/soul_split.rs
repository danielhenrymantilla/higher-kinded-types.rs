#![allow(unsafe_code)]

/// Much flavor.
pub trait Vessel : 'static + crate::ForLt {
    type PosessedBy<'soul>;
}
impl<T : 'static + crate::ForLt> Vessel for T {
    type PosessedBy<'soul> = T::Of<'soul>;
}

pub type Soul<'r> = helper::Soul<'r>;
#[doc(hidden)] pub
use helper::*;
mod helper {
    #![allow(warnings)]

    #[derive(Clone, Copy, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub
    enum Soul<'soul> {
        #[default]
        Soul,

        #[doc(hidden)] /** Not part of the public API */
        ඞ {
            _phantom_variant: ::never_say_never::Never,
            _invariant: ::core::marker::PhantomData<fn(&'soul ()) -> &'soul ()>,
        },
    }

    mod ඞ {}

    pub use self::Soul::*;
}

pub
struct SoulAndBody<'soul, Body : Vessel>(
    pub Body::PosessedBy<'soul>,
);

/// Same a `mem::transmute()`, but restricted to transmuting a single lifetime,
/// for a finer-grained and thus less error-prone function, as well as for MSRV
/// reasons (older Rust did non understand that lifetimes cannot affect layout).
unsafe
fn transmute_soul<'src, 'dst, Body : Vessel>(
    it: Body::PosessedBy<'src>,
) -> Body::PosessedBy<'dst>
where
    // allow the lifetimes to be turbofished for further opt-in safety
    // (and make rustfmt have a seizure).
    'src :,
    'dst :,
{
    // SAFETY: a lifetime difference, alone, cannot cause a change of layout.
    ::core::mem::transmute_copy(&::core::mem::ManuallyDrop::new(it))
}

impl<'soul, Body : Vessel> SoulAndBody<'soul, Body> {
    /// Conceptually.
    fn _erase_soul_conceptually(
        self: SoulAndBody<'soul, Body>,
    ) -> impl 'soul + FnOnce(for<'any> fn(SoulAndBody<'any, Body>))
    {
        |yield_| yield_(self)
    }

    fn erase_soul(
        self: SoulAndBody<'soul, Body>,
    ) -> VacantVessel<Body>
    {
        VacantVessel(::maybe_dangling::ManuallyDrop::new(unsafe {
            transmute_soul::<'soul, 'static, Body>(self.0)
        }))
    }

    /// Split a [`SoulAndBody`] into its two core
    /// [`VacantVessel<Body>`] and [`Soul`] constituents.
    ///
    /// ![Dr. Strange soul split](
    /// https://user-images.githubusercontent.com/9920355/251223491-15d2c0d4-5c7d-4020-9640-8f202734555e.png)
    pub
    fn exorcise_soul(
        self: SoulAndBody<'soul, Body>,
    ) -> (VacantVessel<Body>, Soul<'soul>)
    {
        (self.erase_soul(), Soul::<'soul>)
    }
}

pub
struct VacantVessel<Body : Vessel> /* = */ (
    ::maybe_dangling::ManuallyDrop<Body::PosessedBy<'static>>,
);

macro_rules! usability_safety {() => (
    concat!(
        " # Safety\n",
        "\n",
        "`'soul : 'call` needs to hold (with `'soul` being that of the\n",
        " `Soul` returned by [`SoulAndBody::exorcise_soul()`])",
    )
)}

impl<Body : Vessel> VacantVessel<Body> {
    /// # Safety
    ///
    /// The infused <code>[Soul]<\'soul></code> has to be the one returned by
    /// [`SoulAndBody::exorcise_soul()`].
    pub
    unsafe
    fn reinfuse_soul<'soul>(
        self: VacantVessel<Body>,
        _soul: Soul<'soul>,
    ) -> SoulAndBody<'soul, Body>
    {
        SoulAndBody(unsafe {
            transmute_soul::<'static, 'soul, Body>(::maybe_dangling::ManuallyDrop::into_inner(self.0))
        })
    }

    #[doc = usability_safety!()]
    pub
    unsafe
    fn drop<'call>(self: VacantVessel<Body>)
    {
        { self }.with_mut(|r| <*mut Body::PosessedBy<'_>>::drop_in_place(r))
    }

    #[doc = usability_safety!()]
    pub
    unsafe
    fn with<'call, R>(
        self: VacantVessel<Body>,
        yield_: impl for<'soul> FnOnce(Body::PosessedBy<'soul>) -> R,
    ) -> R
    {
        yield_(::maybe_dangling::ManuallyDrop::into_inner(self.0))
    }

    #[doc = usability_safety!()]
    pub
    unsafe
    fn with_ref<'call, R>(
        self: &VacantVessel<Body>,
        yield_: impl for<'soul> FnOnce(&Body::PosessedBy<'soul>) -> R,
    ) -> R
    {
        // Note: we could be tempted to make `&'r self` yield a
        // `&'r Body::PosessedBy<'soul>`, but this is where anonymous lifetimes
        // are tricky. As a rule of thumb: **you should never lower-bound a
        // purposedly-anonymous lifetime**.
        //
        // Indeed, what if this lower bound, such as `'r` in this hypothesized
        // API, were `'static`, *the maximal lifetime*?
        //
        // Then our purposedly-anonymous/ineffable lifetime would end up
        // cornered between a rock and a hard place:
        //
        //  1. `'static ⊇ '_ ⊇ 'r`
        //  2. `'static ⊇ '_ ⊇ 'static`
        //  3. `'_ = 'static`
        //  4. `'_` is no longer anonymous/ineffable!
        //
        // A corollary of this API restriction is that:
        // `fn as_ref<'r>(&'r self) -> VacantVessel<ForLt!(&'r Body::PosessedBy<'_>)>`
        // would be just as unsound (since `.as_ref().with()` would boil down to
        // the aforementioned problematic API).
        yield_(&self.0)
    }

    #[doc = usability_safety!()]
    pub
    unsafe
    fn with_mut<'call, R>(
        self: &mut VacantVessel<Body>,
        yield_: impl for<'soul> FnOnce(&mut Body::PosessedBy<'soul>) -> R,
    ) -> R
    {
        yield_(&mut self.0)
    }
}

#[test]
fn raw_non_static_any() {
    extern crate std;
    use std::prelude::v1::*;

    use crate::ForLt;
    use {
        ::core::any::Any,
    };

    let local = String::from("…");
    let example: &str = &local;
    let (body, soul) = SoulAndBody::<ForLt!(&str)>(example).exorcise_soul();
    let any_and_soul = (Box::new(body) as Box<dyn Any>, soul);

    let shtatic: &'static str = "static";
    let (body, soul) = SoulAndBody::<ForLt!(&'static str)>(shtatic).exorcise_soul();
    let any_and_soul_2 = (Box::new(body) as Box<dyn Any>, soul);

    // move them around (_e.g._, with other erased types)
    let [any_and_soul, any_and_soul_2] = [any_and_soul, any_and_soul_2];

    // and, later on:
    unsafe {
        let (any, soul) = any_and_soul; // <- imagine a proper privacy API => `Any<'lt>`!
        // not static!
        let _example: &str = any.downcast::<VacantVessel<ForLt!(&str)>>().unwrap().reinfuse_soul(soul).0;

        let (any, soul) = any_and_soul_2;
        // static!
        let _shtatic: &'static str = any.downcast::<VacantVessel<ForLt!(&'static str)>>().unwrap().reinfuse_soul(soul).0;
    }
}
