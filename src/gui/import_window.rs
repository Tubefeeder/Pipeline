use gdk::{glib::Object, subclass::prelude::ObjectSubclassIsExt};

gtk::glib::wrapper! {
    pub struct ImportWindow(ObjectSubclass<imp::ImportWindow>)
        @extends gtk::Dialog, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ImportWindow {
    pub fn new(parent: &crate::gui::window::Window) -> Self {
        let s: Self =
            Object::new(&[("transient-for", &parent)]).expect("Failed to create ImportWindow");
        s.imp().joiner.replace(parent.imp().joiner.borrow().clone());
        s
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk_pixbuf::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::builders::FileChooserNativeBuilder;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::FileChooserAction;
    use gtk::FileFilter;
    use gtk::ResponseType;
    use tf_join::Joiner;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/import_window.ui")]
    pub struct ImportWindow {
        pub(super) joiner: RefCell<Option<Joiner>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImportWindow {
        const NAME: &'static str = "TFImportWindow";
        type Type = super::ImportWindow;
        type ParentType = gtk::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ImportWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for ImportWindow {}
    impl WindowImpl for ImportWindow {}
    impl DialogImpl for ImportWindow {
        fn response(&self, dialog: &Self::Type, response: gtk::ResponseType) {
            match response {
                ResponseType::Other(1) => {
                    log::debug!("Import from NewPipe");
                    let filter = FileFilter::new();
                    filter.add_mime_type("application/json");
                    let chooser = FileChooserNativeBuilder::new()
                        .transient_for(dialog)
                        .filter(&filter)
                        .action(FileChooserAction::Open)
                        .build();
                    let obj = self.instance();
                    chooser.connect_response(
                        clone!(@strong chooser, @strong obj => move |_, action| {
                            if action == ResponseType::Accept {
                                log::trace!("User picked file to import from");
                                let file = chooser.file();
                                if let Some(file) = file {
                                    if let Err(e) = crate::import::import_newpipe(&obj.imp().joiner.borrow().as_ref().expect("Joiner to be set up"), file) {
                                        let dialog = gtk::MessageDialog::builder()
                                            .text(&gettextrs::gettext("Failure to import subscriptions"))
                                            .secondary_text(&format!("{}", e))
                                            .message_type(gtk::MessageType::Error).build();
                                        dialog.show();
                                    }
                                }
                            } else {
                                log::trace!("User did not choose anything to import from");
                            }
                        }),
                    );
                    chooser.show();
                }
                ResponseType::Other(2) => {
                    log::debug!("Import from YouTube");
                    let filter = FileFilter::new();
                    filter.add_mime_type("text/csv");
                    let chooser = FileChooserNativeBuilder::new()
                        .transient_for(dialog)
                        .filter(&filter)
                        .action(FileChooserAction::Open)
                        .build();
                    let obj = self.instance();
                    chooser.connect_response(
                        clone!(@strong chooser, @strong obj => move |_, action| {
                            if action == ResponseType::Accept {
                                log::trace!("User picked file to import from");
                                let file = chooser.file();
                                if let Some(file) = file {
                                    if let Err(e) = crate::import::import_youtube(&obj.imp().joiner.borrow().as_ref().expect("Joiner to be set up"), file) {
                                        let dialog = gtk::MessageDialog::builder()
                                            .text(&gettextrs::gettext("Failure to import subscriptions"))
                                            .secondary_text(&format!("{}", e))
                                            .message_type(gtk::MessageType::Error).build();
                                        dialog.show();
                                    }
                                }
                            } else {
                                log::trace!("User did not choose anything to import from");
                            }
                        }),
                    );
                    chooser.show();
                }
                _ => {}
            }
            dialog.close();
            self.parent_response(dialog, response)
        }
    }
}
