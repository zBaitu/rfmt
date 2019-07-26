type a = impl Trait;
type b = impl Trait + Send;
type c = impl Trait + Send + Sync;
type d = impl Trait + 'static;
type e = impl Trait + Send + 'static;
type g = impl 'static + Trait;
type h = impl (Trait);
