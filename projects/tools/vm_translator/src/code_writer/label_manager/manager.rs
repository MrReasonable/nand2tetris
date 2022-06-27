use super::label_generator::LabelGenerator;

pub struct LabelManager {
    generators: Vec<LabelGenerator>,
}

impl LabelManager {
    pub fn new(filename: &str) -> Self {
        Self {
            generators: vec![LabelGenerator::new(filename)],
        }
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.generators = vec![LabelGenerator::new(filename)];
    }

    pub fn start_function(&mut self, function_name: &str) {
        let last_label = self
            .generators
            .last()
            .map_or(String::new(), |g| g.get_prefix());
        self.generators.push(LabelGenerator::new(
            format!("{}.{}$", last_label, function_name).as_str(),
        ));
    }

    pub fn end_function(&mut self) {
        self.generators.pop();
    }

    pub fn generate_static(&mut self) -> String {
        self.generators
            .first_mut()
            .map_or(String::new(), |generator| generator.generate())
    }

    pub fn generate_label(&mut self, label: &str, unique: bool) -> String {
        self.generators
            .last_mut()
            .map_or(String::new(), |generator| {
                if unique {
                    generator.create_unique(label)
                } else {
                    generator.create(label)
                }
            })
    }
}
