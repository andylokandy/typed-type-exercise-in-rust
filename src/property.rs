#[derive(Debug, Default, Clone, Copy)]
pub struct ValueProperty {
    pub not_null: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FunctionProperty {
    pub preserve_not_null: bool,
    pub commutative: bool,
    // pub injectivity: bool,
}

impl ValueProperty {
    pub fn not_null(mut self, not_null: bool) -> Self {
        self.not_null = not_null;
        self
    }
}

impl FunctionProperty {
    pub fn preserve_not_null(mut self, preserve_not_null: bool) -> Self {
        self.preserve_not_null = preserve_not_null;
        self
    }

    pub fn commutative(mut self, commutative: bool) -> Self {
        self.commutative = commutative;
        self
    }
}
