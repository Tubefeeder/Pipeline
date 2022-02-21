use gdk::subclass::prelude::ObjectSubclassIsExt;
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

gtk::glib::wrapper! {
    pub struct WatchLaterPage(ObjectSubclass<imp::WatchLaterPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl WatchLaterPage {
    pub fn set_playlist_manager(&self, playlist_manager: PlaylistManager<String, AnyVideo>) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().setup();
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::Sender;
    use gdk::glib::PRIORITY_DEFAULT;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::CompositeTemplate;
    use tf_join::AnyVideo;
    use tf_observer::Observer;
    use tf_playlist::PlaylistEvent;
    use tf_playlist::PlaylistManager;

    use crate::gui::feed_item_object::VideoObject;
    use crate::gui::feed_page::FeedPage;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/watch_later.ui")]
    pub struct WatchLaterPage {
        #[template_child]
        pub(super) feed_page: TemplateChild<FeedPage>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,

        _playlist_observer:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>>>,
    }

    impl WatchLaterPage {
        pub(super) fn setup(&self) {
            let mut playlist_manager = self
                .playlist_manager
                .borrow()
                .clone()
                .expect("Playlist Manager has to exist");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

            let observer = Arc::new(Mutex::new(Box::new(PlaylistPageObserver {
                sender: sender.clone(),
            })
                as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

            let mut existing: Vec<VideoObject> = playlist_manager
                .items(&"WATCHLATER".to_string())
                .iter()
                .map(|v| VideoObject::new(v.clone()))
                .collect();
            existing.reverse();

            playlist_manager.attach_at(Arc::downgrade(&observer), &"WATCHLATER".to_string());
            self._playlist_observer.replace(Some(observer));

            let feed_page = &self.feed_page.clone();
            feed_page.set_playlist_manager(playlist_manager);
            feed_page.set_items(existing);

            receiver.attach(
                None,
                clone!(@strong feed_page => move |playlist_event| {
                    match playlist_event {
                        PlaylistEvent::Add(v) => {
                            let video = VideoObject::new(v);
                            feed_page.prepend(video);
                        }
                        PlaylistEvent::Remove(v) => {
                            let video = VideoObject::new(v);
                            feed_page.remove(video);
                        }
                    }
                    Continue(true)
                }),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WatchLaterPage {
        const NAME: &'static str = "TFWatchLaterPage";
        type Type = super::WatchLaterPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WatchLaterPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for WatchLaterPage {}
    impl BoxImpl for WatchLaterPage {}

    pub struct PlaylistPageObserver {
        sender: Sender<PlaylistEvent<AnyVideo>>,
    }

    impl Observer<PlaylistEvent<AnyVideo>> for PlaylistPageObserver {
        fn notify(&mut self, message: PlaylistEvent<AnyVideo>) {
            let _ = self.sender.send(message);
        }
    }
}
