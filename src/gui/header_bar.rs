use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct HeaderBar(ObjectSubclass<imp::HeaderBar>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl HeaderBar {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create HeaderBar")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::SimpleAction;
    use gdk::gio::SimpleActionGroup;
    use gdk::glib::clone;
    use gdk::glib::Object;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::builders::AboutDialogBuilder;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::Widget;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/header_bar.ui")]
    pub struct HeaderBar {
        #[template_child]
        child_box: TemplateChild<gtk::Box>,

        title: RefCell<Option<String>>,
        child: RefCell<Option<Object>>,
    }

    impl HeaderBar {
        fn setup_actions(&self, obj: &super::HeaderBar) {
            let action_about = SimpleAction::new("about", None);
            action_about.connect_activate(|_, _| {
                let about_dialog = AboutDialogBuilder::new()
                    .authors(
                        env!("CARGO_PKG_AUTHORS")
                            .split(";")
                            .map(|s| s.to_string())
                            .collect(),
                    )
                    .comments(env!("CARGO_PKG_DESCRIPTION"))
                    .copyright(
                        include_str!("../../NOTICE")
                            .to_string()
                            .lines()
                            .next()
                            .unwrap_or_default(),
                    )
                    .license_type(gtk::License::Gpl30)
                    .logo_icon_name("icon")
                    .program_name("Tubefeeder")
                    .version(env!("CARGO_PKG_VERSION"))
                    .website(env!("CARGO_PKG_HOMEPAGE"))
                    .build();
                about_dialog.show();
            });

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("win", Some(&actions));
            actions.add_action(&action_about);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeaderBar {
        const NAME: &'static str = "TFHeaderBar";
        type Type = super::HeaderBar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HeaderBar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_actions(obj);
            obj.connect_notify_local(
                Some("child"),
                clone!(@strong self.child_box as b => move |obj, _| {
                    let widget = obj.property::<Option<Object>>("child");
                    if let Some(widget) = widget {
                        b.append(&widget.dynamic_cast::<Widget>().expect("Child has to be a widget"));
                    }
                }),
            );
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("title", "title", "title", None, ParamFlags::READWRITE),
                    ParamSpecObject::new(
                        "child",
                        "child",
                        "child",
                        Widget::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "title" => {
                    let value: Option<String> =
                        value.get().expect("Property title of incorrect type");
                    self.title.replace(value);
                }
                "child" => {
                    let value: Option<Object> =
                        value.get().expect("Property child of incorrect type");
                    self.child.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "child" => self.child.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for HeaderBar {}
    impl BoxImpl for HeaderBar {}
}
