use std::{
    convert::TryFrom,
    fs::OpenOptions,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use csv::ReaderBuilder;

use tf_filter::FilterGroup;
use tf_join::{AnyVideo, AnyVideoFilter};

pub(crate) struct FilterFileManager {
    filters: Arc<Mutex<FilterGroup<AnyVideo>>>,
    path: PathBuf,
}

impl FilterFileManager {
    pub fn new(path: &PathBuf, filters: Arc<Mutex<FilterGroup<AnyVideo>>>) -> Self {
        let manager = Self {
            filters: filters.clone(),
            path: path.clone(),
        };

        manager.fill_filters();
        manager
    }

    fn fill_filters(&self) {
        log::debug!("Filling in the filters list from {:?}", self.path);
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

                let filter_res = AnyVideoFilter::try_from(items.as_slice());

                if filter_res.is_ok() {
                    let filter = filter_res.unwrap();
                    log::debug!("Found filter {:?}", filter);
                    self.filters.lock().unwrap().add(filter);
                } else {
                    log::error!("Error parsing subscription with csv {:?}", items);
                }
            } else {
                log::error!("Error parsing subscription csv");
            }
        }
    }
}
