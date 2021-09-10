use std::{convert::TryFrom, fs::OpenOptions, path::PathBuf};

use csv::{ReaderBuilder, StringRecord, WriterBuilder};

use tf_core::Observer;
use tf_join::{AnySubscription, AnySubscriptionList, SubscriptionEvent};

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

impl Observer<SubscriptionEvent> for SubscriptionFileManager {
    fn notify(&mut self, message: SubscriptionEvent) {
        match message {
            SubscriptionEvent::Add(sub) => {
                let new_record: StringRecord = Vec::<String>::from(sub).into();

                let file_res = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(&self.path);

                // TODO: Error handling
                if file_res.is_err() {
                    log::debug!("A error opening the file occured");
                    return;
                }

                let file = file_res.unwrap();
                let file_clone = file.try_clone().unwrap();

                let csv_reader = ReaderBuilder::new()
                    .has_headers(false)
                    .flexible(true)
                    .from_reader(file);

                let records = csv_reader.into_records();

                for record_res in records {
                    if let Ok(record) = record_res {
                        if new_record == record {
                            log::debug!("Subscription already in subscription file");
                            return;
                        }
                    } else {
                        log::error!("Error parsing subscription csv");
                    }
                }

                // Insert subscription otherwise.
                let mut csv_writer = WriterBuilder::new()
                    .has_headers(false)
                    .flexible(true)
                    .from_writer(file_clone);
                if let Err(_e) = csv_writer.write_record(&new_record) {
                    log::error!("Error writing to file {:?}", self.path)
                }
                if let Err(_e) = csv_writer.flush() {
                    log::error!("Error writing to file {:?}", self.path)
                }
            }
            SubscriptionEvent::Remove(sub) => {
                let new_record: StringRecord = Vec::<String>::from(sub).into();

                let csv_reader_res = ReaderBuilder::new()
                    .has_headers(false)
                    .flexible(true)
                    .from_path(&self.path);

                if let Err(_e) = csv_reader_res {
                    log::error!("Error writing to file {:?}", self.path);
                    return;
                }

                let csv_reader = csv_reader_res.unwrap();

                let records_read = csv_reader.into_records();

                let records: Vec<StringRecord> = records_read
                    .filter(|s| s.is_ok())
                    .map(|s| s.unwrap())
                    .filter(|s| &new_record != s)
                    .collect();

                // Write new subscription.
                let csv_writer_res = WriterBuilder::new()
                    .has_headers(false)
                    .flexible(true)
                    .from_path(&self.path);

                if let Err(_e) = csv_writer_res {
                    log::error!("Error writing to file {:?}", self.path);
                    return;
                }

                let mut csv_writer = csv_writer_res.unwrap();

                for record in records {
                    if let Err(_e) = csv_writer.write_record(&record) {
                        log::error!("Error writing to file {:?}", self.path)
                    }
                }
                if let Err(_e) = csv_writer.flush() {
                    log::error!("Error writing to file {:?}", self.path)
                }
            }
        }
    }
}
