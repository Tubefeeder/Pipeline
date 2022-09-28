use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends libadwaita::PreferencesWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferencesWindow {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create PreferencesWindow")
    }
}

pub mod imp {
    use gdk::gio::Settings;
    use gdk::gio::SettingsBindFlags;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use libadwaita::subclass::prelude::PreferencesWindowImpl;
    use libadwaita::traits::ActionRowExt;
    use libadwaita::traits::PreferencesGroupExt;
    use libadwaita::EntryRow;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        entry_player: TemplateChild<EntryRow>,
        #[template_child]
        entry_downloader: TemplateChild<EntryRow>,

        #[template_child]
        entry_piped_api: TemplateChild<EntryRow>,

        #[template_child]
        group_programs: TemplateChild<libadwaita::PreferencesGroup>,

        settings: Settings,
    }

    #[gtk::template_callbacks]
    impl PreferencesWindow {
        fn init_flatpak(&self) {
            self.group_programs.set_description(Some(&gettextrs::gettext("Note that on Flatpak, there are some more steps required when using a player external to the Flatpak. For more information, please consult the wiki.")));
        }

        fn init_string_setting(
            &self,
            env: &'static str,
            settings: &'static str,
            entry: EntryRow,
        ) {
            let val_env = std::env::var_os(env);
            let val_settings = self.settings.string(settings);
            entry.set_text(&val_settings);
            if val_env.is_some() && &val_env.unwrap() != val_settings.as_str() {
                entry.set_editable(false);
                // TODO: Not really nice to access the parents.
                entry
                    .parent()
                    .expect("Settings entry to have parent")
                    .parent()
                    .expect("Settings entry to have parent")
                    .parent()
                    .expect("Settings entry to have parent")
                    .dynamic_cast::<libadwaita::ActionRow>()
                    .expect("Settings entry to have parent of thpe ActionRow")
                    .set_subtitle(&gettextrs::gettext(
                        "Overwritten by environmental variable.",
                    ));
            }
            self.settings
                .bind(settings, &entry, "text")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            entry.connect_changed(move |entry| std::env::set_var(env, entry.text()));
        }

        fn init_settings(&self) {
            self.init_string_setting("PLAYER", "player", self.entry_player.get());
            self.init_string_setting("DOWNLOADER", "downloader", self.entry_downloader.get());
            self.init_string_setting("PIPED_API_URL", "piped-url", self.entry_piped_api.get());
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "TFPreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = libadwaita::PreferencesWindow;

        fn new() -> Self {
            Self {
                settings: Settings::new(crate::config::APP_ID),
                group_programs: TemplateChild::default(),
                entry_player: TemplateChild::default(),
                entry_downloader: TemplateChild::default(),
                entry_piped_api: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.init_settings();
            if crate::config::FLATPAK {
                self.init_flatpak();
            }
        }
    }
    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
}
