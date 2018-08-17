struct Class<'a> {
    name: &'a str,
    extends: Option<&'a Class<'a>>,
    methods: Vec<&'a Method<'a>>,
}