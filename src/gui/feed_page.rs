use std::cmp::min;

use gdk::{
    gio::{SimpleAction, SimpleActionGroup},
    glib::clone,
    prelude::{ActionMapExt, ListModelExt},
    subclass::prelude::ObjectSubclassIsExt,
};
use gtk::traits::WidgetExt;
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

use super::feed_item_object::VideoObject;

gtk::glib::wrapper! {
    pub struct FeedPage(ObjectSubclass<imp::FeedPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

const LOAD_COUNT: usize = 10;

impl FeedPage {
    fn add_actions(&self) {
        let action_more = SimpleAction::new("more", None);

        action_more.connect_activate(clone!(@strong self as s => move |_, _| {
            let imp = s.imp();
            let items = &imp.items.borrow();
            let model = &imp.model.borrow();
            let loaded_count = &imp.loaded_count.get();

            let to_load = min(LOAD_COUNT, items.len() - loaded_count);

            model.splice(model.n_items(), 0, &items[*loaded_count..(loaded_count + to_load)]);
            imp.loaded_count.set(loaded_count + to_load);
        }));

        let actions = SimpleActionGroup::new();
        self.insert_action_group("feed", Some(&actions));
        actions.add_action(&action_more);
    }

    pub fn set_items(&self, new_items: Vec<VideoObject>) {
        let imp = self.imp();
        let items = &imp.items;
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        let _ = items.replace(new_items);
        model.borrow().remove_all();
        loaded_count.set(0);

        let _ = self.activate_action("feed.more", None);
    }

    pub fn prepend(&self, new_item: VideoObject) {
        let imp = self.imp();
        let items = &imp.items;
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        let _ = items.borrow_mut().insert(0, new_item.clone());
        model.borrow_mut().insert(0, &new_item);
        loaded_count.set(loaded_count.get() + 1);
    }

    pub fn remove(&self, new_item: VideoObject) {
        let imp = self.imp();
        let mut items = imp.items.borrow_mut();
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        if let Some(idx) = items.iter().position(|i| i.video() == new_item.video()) {
            if idx < loaded_count.get() {
                model.borrow().remove(idx as u32);
                loaded_count.set(loaded_count.get() - 1);
            }

            items.remove(idx);
        }
    }

    pub fn set_playlist_manager(&self, playlist_manager: PlaylistManager<String, AnyVideo>) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().setup();
    }
}

pub mod imp {
    use std::cell::{Cell, RefCell};

    use gdk::gio::ListStore;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::SignalListItemFactory;
    use gtk::Widget;

    use gtk::CompositeTemplate;
    use tf_join::AnyVideo;
    use tf_playlist::PlaylistManager;

    use crate::gui::feed_item_object::VideoObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/feed_page.ui")]
    pub struct FeedPage {
        #[template_child]
        pub(super) feed_list: TemplateChild<gtk::ListView>,
        #[template_child]
        load_more: TemplateChild<gtk::Button>,

        pub(super) items: RefCell<Vec<VideoObject>>,
        pub(super) model: RefCell<ListStore>,
        pub(super) loaded_count: Cell<usize>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
    }

    impl FeedPage {
        pub(super) fn setup(&self) {
            let model = gtk::gio::ListStore::new(VideoObject::static_type());
            let selection_model = gtk::NoSelection::new(Some(&model));
            self.feed_list.get().set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            let playlist_manager = self
                .playlist_manager
                .borrow()
                .clone()
                .expect("PlaylistManager should be set up");
            factory.connect_setup(move |_, list_item| {
                let feed_item = crate::gui::feed_item::FeedItem::new(playlist_manager.clone());
                list_item.set_child(Some(&feed_item));

                list_item
                    .property_expression("item")
                    .bind(&feed_item, "video", Widget::NONE);
            });
            self.feed_list.set_factory(Some(&factory));
            self.feed_list.set_single_click_activate(true);

            self.feed_list.connect_activate(move |list_view, position| {
                let model = list_view.model().expect("The model has to exist.");
                let video_object = model
                    .item(position)
                    .expect("The item has to exist.")
                    .downcast::<VideoObject>()
                    .expect("The item has to be an `VideoObject`.");

                video_object.play();
            });
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedPage {
        const NAME: &'static str = "TFFeedPage";
        type Type = super::FeedPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.add_actions();
        }
    }

    impl WidgetImpl for FeedPage {}
    impl BoxImpl for FeedPage {}
}
