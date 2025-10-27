use enum_bind::Bind;

#[test]
fn extract_key() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn a(&self) -> Option<i32>)]
    enum Enum {
        #[bind(a = 1)]
        Alpha,
        Beta { a: i32 },
        Gamma { b: &'static str },
    }
    use Enum::*;

    assert_eq!(Alpha.a(), Some(1));
    assert_eq!(Beta { a: 2 }.a(), Some(2));
    assert_eq!(Gamma { b: "test" }.a(), None);
}

#[test]
fn bidi_capture() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn a(&self) -> Option<i32>, return = Strict(a))]
    #[query(fn by_a(a: Option<i32>) -> Option<Self>)]
    enum Enum {
        #[bind(a = Some(b))]
        Alpha { b: i32 },
        Beta { a: Option<i32> },
    }
    use Enum::*;

    assert_eq!(Alpha { b: 100 }.a(), Some(100));
    assert_eq!(Beta { a: Some(2) }.a(), Some(2));
    assert_eq!(Beta { a: None }.a(), None);

    assert_eq!(Enum::by_a(Some(1)), Some(Alpha { b: 1 }));
    assert_eq!(Enum::by_a(None), Some(Beta { a: None }));
}

#[test]
fn map_fields() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn name(&self) -> Option<&'static str>)]
    enum Declaration {
        Function { name: &'static str },

        #[bind(name = struct_name)]
        Struct { struct_name: &'static str },

        #[bind(name = const_name)]
        Variable { const_name: &'static str },

        #[bind(name = "marker")]
        Marker,
    }

    use Declaration::*;

    assert_eq!(Function { name: "foo" }.name(), Some("foo"));
    assert_eq!(Struct { struct_name: "Bar" }.name(), Some("Bar"));
    assert_eq!(Variable { const_name: "BAZ" }.name(), Some("BAZ"));
    assert_eq!(Marker.name(), Some("marker"));
}

#[test]
fn dispatch_and_map_input_to_different_fields() {
    #[derive(Bind, Debug, PartialEq)]
    #[query(fn create(kind: &'static str, name: &'static str) -> Option<Self>)]
    #[query(fn name(&self) -> &'static str, return = Strict(name))]
    enum Declaration {
        #[bind(kind = "fn")]
        Function { name: &'static str },

        #[bind(kind = "struct", name = struct_name)]
        Struct { struct_name: &'static str },

        #[bind(kind = "const", name = const_name)]
        Variable { const_name: &'static str },

        #[bind(kind = "marker", name = "marker")]
        Marker,
    }

    use Declaration::*;

    assert_eq!(Declaration::create("fn", "foo"), Some(Function { name: "foo" }));
    assert_eq!(Declaration::create("struct", "Bar"), Some(Struct { struct_name: "Bar" }));
    assert_eq!(Declaration::create("const", "BAZ"), Some(Variable { const_name: "BAZ" }));
    assert_eq!(Declaration::create("enum", "E"), None);

    assert_eq!(Function { name: "foo" }.name(), "foo");
    assert_eq!(Struct { struct_name: "Bar" }.name(), "Bar");
    assert_eq!(Variable { const_name: "BAZ" }.name(), "BAZ");
    assert_eq!(Marker.name(), "marker");
}
