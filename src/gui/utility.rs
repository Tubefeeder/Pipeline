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
}
