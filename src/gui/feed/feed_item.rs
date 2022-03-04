use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::Object;
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

gtk::glib::wrapper! {
    pub struct FeedItem(ObjectSubclass<imp::FeedItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FeedItem {
    pub fn new(playlist_manager: PlaylistManager<String, AnyVideo>) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create FeedItem");
        s.imp().playlist_manager.replace(Some(playlist_manager));
        s
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::SimpleAction;
    use gdk::gio::SimpleActionGroup;
    use gdk::glib::clone;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use glib::ParamFlags;
    use glib::ParamSpec;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_core::Video;
    use tf_join::AnyVideo;
    use tf_playlist::PlaylistManager;

    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::thumbnail::Thumbnail;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/feed_item.ui")]
    pub struct FeedItem {
        #[template_child]
        label_title: TemplateChild<gtk::Label>,
        #[template_child]
        label_author: TemplateChild<gtk::Label>,
        #[template_child]
        label_platform: TemplateChild<gtk::Label>,
        #[template_child]
        label_date: TemplateChild<gtk::Label>,

        #[template_child]
        playing: TemplateChild<gtk::Image>,
        #[template_child]
        thumbnail: TemplateChild<Thumbnail>,

        #[template_child]
        watch_later: TemplateChild<gtk::Button>,

        video: RefCell<Option<VideoObject>>,
        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
    }

    impl FeedItem {
        fn setup_actions(&self, obj: &super::FeedItem) {
            let action_download = SimpleAction::new("download", None);
            action_download.connect_activate(clone!(@strong self.video as video => move |_, _| {
                video.borrow().as_ref().expect("Video should be set up").download();
            }));
            let action_clipboard = SimpleAction::new("clipboard", None);
            action_clipboard.connect_activate(clone!(@strong self.video as video, @strong obj => move |_, _| {
                let clipboard = obj.display().clipboard();
                // Replace // with / because of simple bug I am too lazy to fix in the youtube-extractor.
                clipboard.set_text(&video.borrow().as_ref().expect("Video should be set up").video().expect("Video should be set up").url().replace("//watch", "/watch"));
            }));

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("item", Some(&actions));
            actions.add_action(&action_download);
            actions.add_action(&action_clipboard);
        }
        fn bind_watch_later(&self) {
            let video = &self.video;
            let playlist_manager = &self.playlist_manager;
            self.watch_later.connect_clicked(
                clone!(@strong video, @strong playlist_manager => move |_| {
                    let video = video.borrow().as_ref().map(|v| v.video()).flatten();
                    if let Some(video) = video {
                        let mut playlist_manager = playlist_manager.borrow_mut();
                        playlist_manager.as_mut().unwrap().toggle(&"WATCHLATER".to_owned(), &video);
                    }
                }),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedItem {
        const NAME: &'static str = "TFFeedItem";
        type Type = super::FeedItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "video",
                    "video",
                    "video",
                    VideoObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "video" => {
                    let value: Option<VideoObject> =
                        value.get().expect("Property video of incorrect type");
                    self.video.replace(value);
                    self.bind_watch_later();
                    self.setup_actions(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "video" => self.video.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FeedItem {}
    impl BoxImpl for FeedItem {}
}
