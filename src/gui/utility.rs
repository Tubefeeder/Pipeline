pub struct Utility {}

#[gtk::template_callbacks(functions)]
impl Utility {
    #[template_callback]
    fn small(#[rest] values: &[gtk::glib::Value]) -> String {
        format!(
            "<small>{}</small>",
            values[0]
                .get::<&str>()
                .expect("Expected string for argument")
        )
    }

    #[template_callback]
    fn not(#[rest] values: &[gtk::glib::Value]) -> bool {
        !values[0]
            .get::<bool>()
            .expect("Expected boolean for argument")
    }

    #[template_callback]
    fn is_empty(#[rest] values: &[gtk::glib::Value]) -> bool {
        let value = values[0]
            .get::<Option<String>>()
            .expect("Expected string for argument");
        value.is_none() || value.unwrap().is_empty()
    }
}
