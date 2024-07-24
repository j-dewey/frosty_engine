use frosty_alloc::ObjectHandleMut;

pub(crate) enum QueryForm {
    // take all objects at once
    Continuous,
    // only take one object at a time
    Discrete,
}

pub(crate) struct Query {
    form: QueryForm,
    objs: Vec<ObjectHandleMut<u8>>,
}
