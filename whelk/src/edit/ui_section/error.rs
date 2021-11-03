use super::UiSection;

impl UiSection {
    pub fn show_error(&self) {
        let el = self.root_el();
        el.class_list().add_1("error-span").unwrap();
    }
}
