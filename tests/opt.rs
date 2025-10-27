use enum_bind::Bind;

#[test]
fn opt_all_variants_have_column() {
    #[derive(Bind)]
    #[query(fn a(&self) -> Option<i32>)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        #[bind(a = 2)]
        Beta,
    }
    use Enum::*;

    assert_eq!(Alpha.a(), Some(1));
    assert_eq!(Beta.a(), Some(2));
}

#[test]
fn variant_without_column() {
    #[derive(Bind)]
    #[query(fn a(&self) -> Option<i32>)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        Beta,
    }
    use Enum::*;

    assert_eq!(Alpha.a(), Some(1));
    assert_eq!(Beta.a(), None);
}

#[test]
fn field_as_column() {
    #[derive(Bind)]
    #[query(fn a(&self) -> Option<i32>)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        #[bind()]
        Beta { a: i32 },
    }
    use Enum::*;

    assert_eq!(Alpha.a(), Some(1));
    assert_eq!(Beta { a: 2 }.a(), Some(2));
}

#[test]
fn opt_self_for_column() {
    #[derive(Bind, PartialEq, Debug)]
    #[query(fn by_a(a: usize) -> Option<Self>)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        #[bind(a = 2)]
        Beta,
    }
    use Enum::*;

    assert_eq!(Enum::by_a(1), Some(Alpha));
    assert_eq!(Enum::by_a(2), Some(Beta));
}

#[test]
fn opt_self_for_column_with_capture() {
    #[derive(Bind, PartialEq, Debug)]
    #[query(fn by_a(a: usize) -> Option<Self>)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        Beta { a: usize },
    }
    use Enum::*;

    assert_eq!(Enum::by_a(1), Some(Alpha));
    assert_eq!(Enum::by_a(2), Some(Beta { a: 2 }));
}
