fn test_to_json() {
    extern crate a;
    use super::ToJson;
    mod b;

    let array2 = Array(vec!(I64(1), I64(2)));
    let array3 = Array(vec!(I64(1), I64(2), I64(3)));
    let object = {
        let mut tree_map = BTreeMap::new();
        tree_map.insert("a".to_string(), U64(1));
        tree_map.insert("b".to_string(), U64(2));
        Object(tree_map)
    };
}
