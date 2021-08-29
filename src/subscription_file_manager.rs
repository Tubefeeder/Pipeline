use std::{convert::TryFrom, fs::OpenOptions, path::PathBuf};

use csv::ReaderBuilder;

use tf_join::{AnySubscription, AnySubscriptionList};

pub(crate) struct SubscriptionFileManager {
    subscriptions: AnySubscriptionList,
    path: PathBuf,
}

impl SubscriptionFileManager {
    pub fn new(path: &PathBuf, subscriptions: &AnySubscriptionList) -> Self {
        let manager = Self {
            subscriptions: subscriptions.clone(),
            path: path.clone(),
        };

        manager.fill_subscriptions();
        manager
    }

    fn fill_subscriptions(&self) {
        log::debug!("Filling in the subscriptions list from {:?}", self.path);
        let file_res = OpenOptions::new().read(true).write(false).open(&self.path);

        // TODO: Error handling
        if file_res.is_err() {
            log::debug!("A error opening the file occured");
            return;
        }

        let csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(file_res.unwrap());

        let records = csv_reader.into_records();

        for record_res in records {
            if let Ok(record) = record_res {
                let items: Vec<&str> = record.iter().collect();

                let subscription_res = AnySubscription::try_from(items.as_slice());

                if subscription_res.is_ok() {
                    let subscription = subscription_res.unwrap();
                    log::debug!("Found subscription {}", subscription);
                    self.subscriptions.add(subscription);
                } else {
                    log::error!("Error parsing subscription with csv {:?}", items);
                }
            } else {
                log::error!("Error parsing subscription csv");
            }
        }
    }
}
