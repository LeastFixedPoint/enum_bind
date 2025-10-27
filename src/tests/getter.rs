use enum_bind::Relational;

#[derive(Relational)]
#[query_one(pub fn x(&self) -> i32)]
enum TestEnumGetter {
    #[bind(x = 1)]
    Alpha,
    #[bind(x = 1 + 1)]
    Beta,
    #[bind(x = TestEnumGetter::Beta.x() + 1)]
    Gamma,
    #[bind(x = *a)]
    Delta { a: i32 },
    #[bind(x = *_0)]
    Epsilon(i32),
}

#[test]
pub fn test_enum_getter() {
    assert_eq!(TestEnumGetter::Alpha.x(), 1);
    assert_eq!(TestEnumGetter::Beta.x(), 2);
    assert_eq!(TestEnumGetter::Gamma.x(), 3);
    assert_eq!(TestEnumGetter::Delta { a: 4 }.x(), 4);
    assert_eq!(TestEnumGetter::Epsilon(5).x(), 5);
}

#[derive(Relational)]
#[query_one(x: pub fn get_x(&self) -> i32)]
enum TestEnumGetterWithRename {
    #[bind(x = 1)]
    Alpha,
    #[bind(x = 2)]
    Beta,
}

#[test]
pub fn test_enum_getter_with_rename() {
    assert_eq!(TestEnumGetterWithRename::Alpha.get_x(), 1);
    assert_eq!(TestEnumGetterWithRename::Beta.get_x(), 2);
}
