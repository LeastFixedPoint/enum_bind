#[allow(unused)]

use tabular_enum::Relational;

#[derive(Relational, PartialEq, Debug)]
#[query_one(fn foo(&self) -> usize)]
#[query_opt(fn bar(&self) -> Option<&'static str>)]
#[query_all(fn bar(&self) -> Selection<&'static str>)]
#[query_opt(fn by_foo_bar(foo: i32, bar: &'static str) -> Option<Self>)]
#[query_opt(fn bar(&self, foo: i32) -> Option<&'static str>)]
// #[lookup(fn by_foo_bar(foo: i32, bar: &'static str) -> Option<Self>)]
// #[lookup(fn by_foo(foo: i32) -> Option<Self>)]
// #[lookup(fn by_bar(bar: &'static str) -> Option<Self>)]
// #[getter(fn foo(&self) -> i32)]
// #[getopt(fn bar(&self) -> Option<&'static str>)]
enum TestEnumLookup {
	#[bind(foo = 1)]
    Absent,

	#[bind(foo = 2, bar = "present")]
	#[bind(foo = 2, bar = "alternative")]
    Present,

	#[bind(foo = 3, bar = _)]
    BindAll { bar: &'static str },

	#[bind(foo = 3, bar = "bound1")]
	#[bind(foo = 4, bar = "bound2")]
    BindSome { bar: &'static str },

	#[bind(foo = 6, bar = qux @ _)] 
    Rename { qux: &'static str },

	#[bind(foo = 7, bar = _)]
    Any,
}

#[test]
pub fn test_enum_lookup() {
    // assert_eq!(TestEnumLookup::by_foo(0), None);
    // assert_eq!(TestEnumLookup::by_foo(1), Some(TestEnumLookup::Absent));
    // assert_eq!(TestEnumLookup::by_foo(2), Some(TestEnumLookup::Present));
    // assert_eq!(TestEnumLookup::by_foo(3), Some(TestEnumLookup::Any));

    assert_eq!(TestEnumLookup::by_bar("present"), Some(TestEnumLookup::Present));
    assert_eq!(TestEnumLookup::by_bar("bar"), Some(TestEnumLookup::Match { bar: "bar" }));
    assert_eq!(TestEnumLookup::by_bar("qux"), Some(TestEnumLookup::Rename { qux: "qux" }));
    assert_eq!(TestEnumLookup::by_bar("other"), Some(TestEnumLookup::Any));

    // assert_eq!(TestEnumLookup::by_foo_bar_one("any"), None);
    // assert_eq!(TestEnumLookup::by_foo_bar(2, "present"), Some(TestEnumLookup::Present));
    // assert_eq!(TestEnumLookup::by_foo_bar(2, "other"), None);
    // assert_eq!(TestEnumLookup::by_foo_bar(3, "any"), Some(TestEnumLookup::Any));

    assert_eq!(TestEnumLookup::Absent.bar(), None);
    assert_eq!(TestEnumLookup::Present.bar(), Some("present"));
    assert_eq!(TestEnumLookup::Match { bar: "bar" }.bar(), Some("bar"));
    assert_eq!(TestEnumLookup::Match { bar: "other" }.bar(), None);
    assert_eq!(TestEnumLookup::Rename { qux: "qux" }.bar(), Some("qux"));
    assert_eq!(TestEnumLookup::Rename { qux: "other" }.bar(), None);
    assert_eq!(TestEnumLookup::Any.bar(), None);
}
