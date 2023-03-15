/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk_pixbuf::prelude::Cast;
use gtk::traits::WidgetExt;
use tf_join::{AnySubscriptionList, AnyVideo};
use tf_playlist::PlaylistManager;

gtk::glib::wrapper! {
    pub struct SubscriptionPage(ObjectSubclass<imp::SubscriptionPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SubscriptionPage {
    pub fn set_subscription_list(
        &self,
        subscription_list: AnySubscriptionList,
        playlist_manager: PlaylistManager<String, AnyVideo>,
    ) {
        self.imp()
            .any_subscription_list
            .replace(Some(subscription_list.clone()));
        self.imp()
            .subscription_list
            .get()
            .set_subscription_list(subscription_list);
        self.imp()
            .subscription_video_list
            .get()
            .set_playlist_manager(playlist_manager);
    }

    fn window(&self) -> crate::gui::window::Window {
        self.root()
            .expect("SubscriptionPage to have root")
            .downcast::<crate::gui::window::Window>()
            .expect("Root to be window")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::ListStore;
    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::Object;
    use gdk::glib::ParamSpec;
    use gdk::glib::PRIORITY_DEFAULT;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::ConstantExpression;
    use gtk::PropertyExpression;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_core::Generator;
    use tf_join::AnySubscriptionList;
    use tf_join::Platform;
    use tf_lbry::LbrySubscription;
    use tf_pt::PTSubscription;
    use tf_yt::YTSubscription;

    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::feed_list::FeedList;
    use crate::gui::subscription::platform::PlatformObject;
    use crate::gui::subscription::subscription_item_object::SubscriptionObject;
    use crate::gui::subscription::subscription_list::SubscriptionList;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/subscription_page.ui")]
    pub struct SubscriptionPage {
        #[template_child]
        pub(super) subscription_list: TemplateChild<SubscriptionList>,

        #[template_child]
        pub(super) btn_toggle_add_subscription: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) dropdown_platform: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub(super) entry_url: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) entry_name_id: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) dialog_add: TemplateChild<libadwaita::MessageDialog>,

        #[template_child]
        pub(super) subscription_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) subscription_video_list: TemplateChild<FeedList>,

        pub(super) any_subscription_list: RefCell<Option<AnySubscriptionList>>,
    }

    impl SubscriptionPage {
        fn setup_toggle_add_subscription(&self, obj: &super::SubscriptionPage) {
            self.btn_toggle_add_subscription.connect_clicked(clone!(@strong obj as s,
                                                                    @strong self.dialog_add as dialog,
                                                                    @strong self.entry_url as in_url,
                                                                    @strong self.entry_name_id as in_name_id,
                                                                    @strong self.dropdown_platform as dropdown_platform,
                                                                    => move |_| {
                dropdown_platform.set_selected(0);
                in_url.set_text("");
                in_name_id.set_text("");

                // Theoretically only needs to be done once, but when setting up the page does
                // not yet have a root.
                let window = s.window();
                dialog.set_transient_for(Some(&window));
                dialog.present();
            }));
        }

        fn setup_platform_dropdown(&self) {
            self.dropdown_platform
                .set_expression(Some(&PropertyExpression::new(
                    PlatformObject::static_type(),
                    None::<ConstantExpression>,
                    "name",
                )));

            let model = ListStore::new(PlatformObject::static_type());
            model.splice(
                0,
                0,
                &[
                    PlatformObject::new(Platform::Youtube),
                    PlatformObject::new(Platform::Lbry),
                    PlatformObject::new(Platform::Peertube),
                ],
            );
            self.dropdown_platform.set_model(Some(&model));
        }
    }

    #[gtk::template_callbacks]
    impl SubscriptionPage {
        #[template_callback]
        fn handle_add_subscription(&self, response: Option<&str>) {
            if response != Some("add") {
                return;
            }

            let in_platform = &self.dropdown_platform;
            let in_url = &self.entry_url;
            let in_name_id = &self.entry_name_id;

            let platform = in_platform
                .selected_item()
                .expect("Something has to be selected.")
                .downcast::<PlatformObject>()
                .expect("Dropdown items should be of type PlatformObject.")
                .platform()
                .expect("The platform has to be set up.");
            let url = in_url.text();
            let name_id = in_name_id.text();

            in_url.set_text("");
            in_name_id.set_text("");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
            let sender = sender.clone();
            tokio::spawn(async move {
                let subscription = match platform {
                    Platform::Youtube => YTSubscription::try_from_search(&name_id)
                        .await
                        .map(|s| s.into()),
                    Platform::Peertube => Some(PTSubscription::new(&url, &name_id).into()),
                    Platform::Lbry => Some(LbrySubscription::new(&name_id).into()),
                    // -- Add case here
                };
                if let Some(subscription) = subscription {
                    sender
                        .send(subscription)
                        .expect("Failed to send message about subscription");
                } else {
                    // TODO: Better Error Handling
                    log::error!("Failed to get subscription with supplied data");
                }
            });

            receiver.attach(
                None,
                clone!(@strong self.any_subscription_list as list =>
                       move |sub| {
                           list.borrow().as_ref().expect("SubscriptionList should be set up").add(sub);
                           Continue(true)
                       }
                )
            );
        }

        #[template_callback]
        fn handle_go_to_videos_page(&self, subscription: SubscriptionObject) {
            log::debug!(
                "Going to videos of subscription {}",
                subscription
                    .subscription()
                    .expect("SubscriptionObject to have value")
            );
            self.subscription_stack.set_visible_child_name("page-vid");
            let joiner = tf_join::Joiner::new();
            joiner.subscription_list().add(
                subscription
                    .subscription()
                    .expect("SubscriptionObject to have value"),
            );

            let error_store = tf_core::ErrorStore::new();

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
            tokio::spawn(async move {
                let videos = joiner.generate(&error_store).await;
                let _ = sender.send(videos);
            });
            let obj = self.instance();
            receiver.attach(
                None,
                clone!(@strong obj as s => @default-return Continue(false), move |videos| {
                    let video_objects = videos.into_iter().map(VideoObject::new).collect::<Vec<_>>();
                    s.imp().subscription_video_list.get().set_items(video_objects);
                    Continue(true)
                }),
            );
        }

        #[template_callback]
        fn handle_go_to_subscriptions_page(&self) {
            log::debug!("Going back to the subscriptions page",);
            self.subscription_stack.set_visible_child_name("page-sub");
        }

        #[template_callback(function)]
        fn url_visible(#[rest] values: &[gtk::glib::Value]) -> bool {
            let platform: Option<PlatformObject> = values[0]
                .get::<Option<Object>>()
                .expect("Parameter must be a Object")
                .map(|o| o.downcast().expect("Parameter must be PlatformObject"));
            platform.as_ref().map(PlatformObject::platform).flatten() == Some(Platform::Peertube)
        }

        #[template_callback(function)]
        fn name_visible(#[rest] _values: &[gtk::glib::Value]) -> bool {
            true
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionPage {
        const NAME: &'static str = "TFSubscriptionPage";
        type Type = super::SubscriptionPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SubscriptionPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_toggle_add_subscription(obj);
            self.setup_platform_dropdown();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            _value: &glib::Value,
            _pspec: &glib::ParamSpec,
        ) {
            unimplemented!()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!()
        }
    }

    impl WidgetImpl for SubscriptionPage {}
    impl BoxImpl for SubscriptionPage {}
}
