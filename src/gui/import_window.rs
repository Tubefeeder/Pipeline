use gdk_pixbuf::glib::clone;
use gtk::builders::FileChooserNativeBuilder;
use gtk::glib;
use gtk::prelude::*;
use gtk::Builder;
use gtk::FileChooserAction;
use gtk::FileFilter;
use gtk::ResponseType;
use libadwaita::traits::MessageDialogExt;
use libadwaita::MessageDialog;
use tf_join::Joiner;

pub fn import_window(joiner: Joiner, parent: &crate::gui::window::Window) -> MessageDialog {
    let builder = Builder::from_resource("/ui/import_window.ui");
    let dialog: MessageDialog = builder
        .object("dialog")
        .expect("import_window.ui to have at least one object dialog");
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.connect_response(
        None,
        clone!(@strong joiner, @weak parent => move |_dialog, response| {
            handle_response(&joiner, response, &parent);
        }),
    );
    dialog
}

fn handle_response(joiner: &Joiner, response: &str, parent: &crate::gui::window::Window) {
    match response {
        "newpipe" => {
            log::debug!("Import from NewPipe");
            let filter = FileFilter::new();
            filter.add_mime_type("application/json");
            let chooser = FileChooserNativeBuilder::new()
                .title(&gettextrs::gettext("Select NewPipe subscriptions file"))
                .transient_for(parent)
                .modal(true)
                .filter(&filter)
                .action(FileChooserAction::Open)
                .build();
            chooser.connect_response(clone!(@strong chooser, @strong joiner => move |_, action| {
                if action == ResponseType::Accept {
                    log::trace!("User picked file to import from");
                    let file = chooser.file();
                    if let Some(file) = file {
                        if let Err(e) = crate::import::import_newpipe(&joiner, file) {
                            let dialog = MessageDialog::builder()
                                .heading(&gettextrs::gettext("Failure to import subscriptions"))
                                .body(&format!("{}", e))
                                .build();
                            dialog.show();
                        }
                    }
                } else {
                    log::trace!("User did not choose anything to import from");
                }
            }));
            chooser.show();
        }
        "youtube" => {
            log::debug!("Import from YouTube");
            let filter = FileFilter::new();
            filter.add_mime_type("text/csv");
            let chooser = FileChooserNativeBuilder::new()
                .title(&gettextrs::gettext("Select YouTube subscription file"))
                .transient_for(parent)
                .filter(&filter)
                .modal(true)
                .action(FileChooserAction::Open)
                .build();
            chooser.connect_response(clone!(@strong chooser, @strong joiner => move |_, action| {
                if action == ResponseType::Accept {
                    log::trace!("User picked file to import from");
                    let file = chooser.file();
                    if let Some(file) = file {
                        if let Err(e) = crate::import::import_youtube(&joiner, file) {
                            let dialog = MessageDialog::builder()
                                .heading(&gettextrs::gettext("Failure to import subscriptions"))
                                .body(&format!("{}", e))
                                .build();
                            dialog.show();
                        }
                    }
                } else {
                    log::trace!("User did not choose anything to import from");
                }
            }));
            chooser.show();
        }
        _ => {}
    }
}
