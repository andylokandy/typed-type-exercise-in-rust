use derive_builder::Builder;

#[derive(Debug, Default, Clone, Builder)]
#[builder(derive(Clone))]
#[builder(default)]
#[builder(pattern = "owned")]
pub struct ValueProperty {
    pub not_null: bool,
}

#[derive(Debug, Default, Clone, Builder)]
#[builder(derive(Clone))]
#[builder(default)]
#[builder(pattern = "owned")]
pub struct FunctionProperty {
    pub preserve_not_null: bool,
    pub commutative: bool,
    // pub injectivity: bool,
}
