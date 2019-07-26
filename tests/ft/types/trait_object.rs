type a = Trait;
type b = dyn Trait + Send;
type c = dyn Trait + Send + Sync;
type d = dyn Trait + 'static;
type e = dyn Trait + Send + 'static;
type g = dyn 'static + Trait;
type h = dyn (Trait);
