use std::sync::{Arc, Mutex};

use gtk::{glib::Object, traits::WidgetExt};
use tf_join::SubscriptionEvent;
use tf_observer::{Observable, Observer};

use crate::csv_file_manager::CsvFileManager;

fn setup_joiner() -> tf_join::Joiner {
    let mut user_data_dir = gtk::glib::user_data_dir();
    user_data_dir.push("tubefeeder");

    let mut subscriptions_file_path = user_data_dir.clone();
    subscriptions_file_path.push("subscriptions.csv");

    let joiner = tf_join::Joiner::new();
    let mut subscription_list = joiner.subscription_list();

    let _subscription_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
        &subscriptions_file_path,
        &mut |sub| subscription_list.add(sub),
    ))
        as Box<dyn Observer<SubscriptionEvent> + Send>));

    subscription_list.attach(Arc::downgrade(&_subscription_file_manager));

    joiner
}

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends libadwaita::ApplicationWindow, gtk::ApplicationWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        Object::new(&[("application", app)]).expect("Failed to create Window")
    }

    pub fn reload(&self) {
        let _ = self.activate_action("win.reload", None);
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::gio::SimpleAction;
    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::PRIORITY_DEFAULT;
    use glib::subclass::InitializingObject;
    use glib::ParamFlags;
    use glib::ParamSpec;
    use glib::ParamSpecBoolean;
    use gtk::glib;
    use gtk::glib::BindingFlags;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::Inhibit;
    use once_cell::sync::Lazy;

    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::AdwApplicationWindowImpl;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use tf_core::ErrorStore;
    use tf_core::Generator;
    use tf_join::AnyVideo;
    use tf_join::Joiner;
    use tf_observer::Observer;
    use tf_playlist::PlaylistEvent;
    use tf_playlist::PlaylistManager;

    use crate::csv_file_manager::CsvFileManager;
    use crate::gui::feed_item_object::VideoObject;
    use crate::gui::feed_page::FeedPage;
    use crate::gui::watch_later::WatchLaterPage;

    use super::setup_joiner;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/window.ui")]
    pub struct Window {
        #[template_child]
        application_stack: TemplateChild<libadwaita::ViewStack>,
        #[template_child]
        btn_reload: TemplateChild<gtk::Button>,
        #[template_child]
        loading_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub(super) feed_page: TemplateChild<FeedPage>,
        #[template_child]
        pub(super) watchlater_page: TemplateChild<WatchLaterPage>,

        reloading: Cell<bool>,
        joiner: RefCell<Option<Joiner>>,
        playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
        _watchlater_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>>>,
    }

    impl Window {
        fn setup_watch_later(&self) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");

            let mut watchlater_file_path = glib::user_data_dir();
            watchlater_file_path.push("tubefeeder");
            watchlater_file_path.push("playlist_watch_later.csv");

            let mut playlist_manager = PlaylistManager::new();
            let mut playlist_manager_clone = playlist_manager.clone();

            let _watchlater_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &watchlater_file_path,
                &mut move |v| {
                    let join_video = joiner.upgrade_video(&v);
                    playlist_manager_clone.toggle(&"WATCHLATER".to_string(), &join_video);
                },
            ))
                as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

            playlist_manager.attach_at(
                Arc::downgrade(&_watchlater_file_manager),
                &"WATCHLATER".to_string(),
            );

            self.playlist_manager
                .replace(Some(playlist_manager.clone()));
            self._watchlater_file_manager
                .replace(Some(_watchlater_file_manager));
            self.feed_page
                .get()
                .set_playlist_manager(playlist_manager.clone());
            self.watchlater_page
                .get()
                .set_playlist_manager(playlist_manager);
        }

        pub fn add_actions(&self, obj: &super::Window) {
            let joiner = setup_joiner();

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
            let reload = SimpleAction::new("reload", None);
            reload.connect_activate(clone!(@strong obj as s, @strong joiner => move |_, _| {
                log::debug!("Reloading");
                s.set_property("reloading", &true);

                let sender = sender.clone();
                let joiner = joiner.clone();
                tokio::spawn(async move {
                    let errors = ErrorStore::new();
                    let videos = joiner.generate(&errors).await;
                    let _ = sender.send(videos);
                });
            }));
            receiver.attach(
            None,
            clone!(@strong obj as s => @default-return Continue(false), move |videos| {
                let video_objects = videos.into_iter().map(VideoObject::new).collect::<Vec<_>>();
                log::debug!("Loaded {} videos", video_objects.len());
                s.imp().feed_page.get().set_items(video_objects);
                s.set_property("reloading", &false);
                Continue(true)
            }),
        );
            obj.add_action(&reload);
            obj.reload();

            self.joiner.replace(Some(joiner));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "TFWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.add_actions(obj);
            self.setup_watch_later();

            obj.bind_property("reloading", &self.btn_reload.get(), "visible")
                .flags(BindingFlags::SYNC_CREATE | BindingFlags::INVERT_BOOLEAN)
                .build();
            obj.bind_property("reloading", &self.loading_spinner.get(), "visible")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecBoolean::new(
                    "reloading",
                    "reloading",
                    "reloading",
                    false,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "reloading" => {
                    let _ = self.reloading.replace(
                        value
                            .get()
                            .expect("The property 'reloading' of TFWindow has to be boolean"),
                    );
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "reloading" => self.reloading.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self, _obj: &Self::Type) -> Inhibit {
            let mut user_cache_dir = glib::user_cache_dir();
            user_cache_dir.push("tubefeeder");

            if user_cache_dir.exists() {
                std::fs::remove_dir_all(user_cache_dir).unwrap_or(());
            }

            Inhibit(false)
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
