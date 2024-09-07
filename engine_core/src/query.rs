use frosty_alloc::ObjectHandleMut;

pub(crate) enum QueryForm {
    // take all objects at once
    Continuous,
    // only take one object at a time
    Discrete(u8),
}

pub(crate) struct Query {
    form: QueryForm,
    objs: Vec<ObjectHandleMut<u8>>,
}

impl Query {
    pub fn new(form: QueryForm, objs: Vec<ObjectHandleMut<u8>>) -> Self {
        Self { form, objs }
    }

    pub fn next(&mut self) -> &mut u8 {
        todo!()
    }
}
