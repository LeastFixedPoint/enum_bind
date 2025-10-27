use enum_bind::Bind;

#[test]
fn all_variants_have_column() {
    #[derive(Bind)]
    #[query(fn a(&self) -> i32, return = Strict)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        #[bind(a = 2)]
        Beta,
    }
    use Enum::*;

    assert_eq!(Alpha.a(), 1);
    assert_eq!(Beta.a(), 2);
}

#[test]
fn column_from_field() {
    #[derive(Bind)]
    #[query(fn a(&self) -> i32, return = Strict)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        Beta { a: i32 },
    }
    use Enum::*;

    assert_eq!(Alpha.a(), 1);
    assert_eq!(Beta { a: 2 }.a(), 2);
}

#[test]
fn expr_column() {
    #[derive(Bind)]
    #[query(fn a(&self) -> i32, return = Strict)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        #[bind(a = 1 + 1)]
        Beta,
    }
    use Enum::*;

    assert_eq!(Alpha.a(), 1);
    assert_eq!(Beta.a(), 2);
}

#[test]
fn expr_using_argument() {
    #[derive(Bind)]
    #[query(fn a(&self, b: i32) -> Option<i32>, return = Option(a))]
    enum Enum {
        #[bind(a = b + 1)]
        Alpha,
    }
    use Enum::*;

    assert_eq!(Alpha.a(/* b = */ 100), Some(101));
}

#[test]
fn skips_variants_without_column() {
    #[derive(Bind, PartialEq, Debug)]
    #[query(fn by_a(a: i32) -> Option<Self>)]
    #[query(fn b(&self) -> Option<i32>, return = Option(b))]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        #[bind(a = !)]
        Beta,
        #[bind(b = a + 1)]
        Gamma { a: i32 },
    }
    use Enum::*;

    assert_eq!(Enum::by_a(1), Some(Alpha));
    assert_eq!(Enum::by_a(2), Some(Gamma { a: 2 }));

    assert_eq!(Alpha.b(), None);
    assert_eq!(Beta.b(), None);
    assert_eq!(Gamma { a: 10 }.b(), Some(11));
}
