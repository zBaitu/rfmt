trait A = B;
trait A = B + C;
trait B<T> = Result + Iterator<T> + 'static + Sized;
trait C = for < 'a, 'b: 'a > Foo< & 'a Bar>;
