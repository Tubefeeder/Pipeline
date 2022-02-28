use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::Object;
use tf_join::AnySubscriptionList;

gtk::glib::wrapper! {
    pub struct SubscriptionItem(ObjectSubclass<imp::SubscriptionItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SubscriptionItem {
    pub fn new(subscription_list: AnySubscriptionList) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create SubscriptionItem");
        s.imp().subscription_list.replace(Some(subscription_list));
        s
    }
}

pub mod imp {
    use std::cell::RefCell;

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
    use tf_join::AnySubscriptionList;

    use crate::gui::subscription::subscription_item_object::SubscriptionObject;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/subscription_item.ui")]
    pub struct SubscriptionItem {
        #[template_child]
        label_name: TemplateChild<gtk::Label>,
        #[template_child]
        label_platform: TemplateChild<gtk::Label>,
        #[template_child]
        remove: TemplateChild<gtk::Button>,

        subscription: RefCell<Option<SubscriptionObject>>,
        pub(super) subscription_list: RefCell<Option<AnySubscriptionList>>,
    }

    impl SubscriptionItem {
        fn bind_remove(&self) {
            let subscription = &self.subscription;
            let subscription_list = &self.subscription_list;
            self.remove.connect_clicked(
                clone!(@strong subscription, @strong subscription_list => move |_| {
                    let subscription = subscription.borrow().as_ref().map(|s| s.subscription()).flatten();
                    if let Some(subscription) = subscription {
                        let mut subscription_list = subscription_list.borrow_mut();
                        subscription_list.as_mut().unwrap().remove(subscription);
                    }
                }),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionItem {
        const NAME: &'static str = "TFSubscriptionItem";
        type Type = super::SubscriptionItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl SubscriptionItem {}

    impl ObjectImpl for SubscriptionItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "subscription",
                    "subscription",
                    "subscription",
                    SubscriptionObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "subscription" => {
                    let value: Option<SubscriptionObject> =
                        value.get().expect("Property video of incorrect type");
                    self.subscription.replace(value);
                    self.bind_remove();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "subscription" => self.subscription.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SubscriptionItem {}
    impl BoxImpl for SubscriptionItem {}
}
