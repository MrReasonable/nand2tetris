pub(crate) struct LabelGenerator {
    prefix: String,
    id: u16,
}

impl LabelGenerator {
    pub(super) fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned().to_uppercase(),
            id: 0,
        }
    }

    pub(super) fn generate(&mut self) -> String {
        self.id += 1;
        self.get_last()
    }

    pub(super) fn get_last(&self) -> String {
        format!("{}_{}", self.prefix, self.id)
    }
}
