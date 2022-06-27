use std::collections::HashMap;

pub(crate) struct LabelGenerator {
    prefix: String,
    id: u16,
    label_idx: HashMap<String, u8>,
    last: String,
}

impl LabelGenerator {
    pub(super) fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned().to_uppercase(),
            id: 0,
            label_idx: HashMap::new(),
            last: String::new(),
        }
    }

    pub(super) fn generate(&mut self) -> String {
        self.id += 1;
        self.last = self.create(format!("{}", &self.id).as_str());
        self.get_last()
    }

    pub(super) fn get_last(&self) -> String {
        self.last.clone()
    }

    pub(super) fn create(&self, name: &str) -> String {
        format!("{}.{}", self.prefix, name)
    }

    pub(super) fn create_unique(&mut self, name: &str) -> String {
        let label = self.create(name);
        let id = self.label_idx.entry(label.clone()).or_default();
        *id += 1;
        self.last = format!("{}.{}", label, id);
        self.get_last()
    }

    pub(super) fn get_prefix(&self) -> String {
        self.prefix.clone()
    }
}
