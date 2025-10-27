use enum_bind::Bind;

#[test]
fn all_argument_binding_cases_create() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn create(x: i32) -> Vec<Self>, return = Vec)]
    enum Enum {
        #[bind(x = 1)]  Alpha,
                       Beta { x: i32 },
        #[bind(x = y)]  Gamma { y: i32 },
        #[bind(x = _0)] Delta(i32),
        #[bind(x = !)]  Epsilon(i32),
        #[bind(x = _)]  Zeta,
    }
    use Enum::*;

    assert_eq!(Enum::create(1), vec![Alpha, Beta { x: 1 }, Gamma { y: 1 }, Delta(1), Zeta]);
    assert_eq!(Enum::create(2), vec![Beta { x: 2 }, Gamma { y: 2 }, Delta(2), Zeta]);
}

#[test]
fn all_argument_binding_cases_select() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn x(self) -> Option<i32>, return = Option)]
    enum Enum {
        #[bind(x = 1)]  Alpha,
                        Beta { x: i32 },
        #[bind(x = y)]  Gamma { y: i32 },
        #[bind(x = _0)] Delta(i32),
        #[bind(x = !)]  Epsilon(i32),
        // #[bind(x = _)]  Zeta, // Not allowed in select queries
    }
    use Enum::*;

    assert_eq!(Alpha.x(), Some(1));
    assert_eq!(Beta { x: 2 }.x(), Some(2));
    assert_eq!(Gamma { y: 3 }.x(), Some(3));
    assert_eq!(Delta(4).x(), Some(4));
    assert_eq!(Epsilon(5).x(), None);
}
