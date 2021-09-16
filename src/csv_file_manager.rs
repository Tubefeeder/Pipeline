/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder-extractor.
 *
 * Tubefeeder-extractor is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder-extractor is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder-extractor.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    convert::{TryFrom, TryInto},
    fs::OpenOptions,
    marker::PhantomData,
    path::PathBuf,
};

use csv::{ReaderBuilder, StringRecord, WriterBuilder};

use tf_filter::FilterEvent;
use tf_join::{AnySubscription, SubscriptionEvent};
use tf_observer::Observer;
use tf_playlist::PlaylistEvent;

pub(crate) struct CsvFileManager<T> {
    path: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T> CsvFileManager<T>
where
    T: TryFrom<Vec<String>>,
{
    pub fn new<F>(path: &PathBuf, add_func: &mut F) -> Self
    where
        F: FnMut(T) -> (),
    {
        let mut manager = Self {
            path: path.clone(),
            _phantom: PhantomData,
        };

        manager.fill(add_func);
        manager
    }

    fn fill<F>(&mut self, add_func: &mut F)
    where
        F: FnMut(T) -> (),
    {
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
                let items: Vec<String> = record.iter().map(|s| s.to_string()).collect();

                let res = T::try_from(items.clone());

                if let Ok(r) = res {
                    add_func(r);
                } else {
                    log::error!("Error parsing csv {:?}", items);
                }
            } else {
                log::error!("Error parsing csv");
            }
        }
    }
}

pub enum CsvEvent<T> {
    Add(T),
    Remove(T),
}

impl<T> TryFrom<PlaylistEvent<T>> for CsvEvent<T> {
    type Error = ();
    fn try_from(e: PlaylistEvent<T>) -> Result<Self, ()> {
        match e {
            PlaylistEvent::Add(i) => Ok(CsvEvent::Add(i)),
            PlaylistEvent::Remove(i) => Ok(CsvEvent::Remove(i)),
        }
    }
}

impl TryFrom<SubscriptionEvent> for CsvEvent<AnySubscription> {
    type Error = ();
    fn try_from(e: SubscriptionEvent) -> Result<Self, ()> {
        match e {
            SubscriptionEvent::Add(i) => Ok(CsvEvent::Add(i)),
            SubscriptionEvent::Remove(i) => Ok(CsvEvent::Remove(i)),
        }
    }
}

impl<T> TryFrom<FilterEvent<T>> for CsvEvent<T> {
    type Error = ();
    fn try_from(e: FilterEvent<T>) -> Result<Self, ()> {
        match e {
            FilterEvent::Add(i) => Ok(CsvEvent::Add(i)),
            FilterEvent::Remove(i) => Ok(CsvEvent::Remove(i)),
        }
    }
}

impl<E, T> Observer<E> for CsvFileManager<T>
where
    E: TryInto<CsvEvent<T>>,
    T: Into<Vec<String>>,
{
    fn notify(&mut self, message: E) {
        match message.try_into() {
            Ok(CsvEvent::Add(item)) => {
                let vec_str: Vec<String> = item.into();
                let new_record: StringRecord = vec_str.into();

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
                            log::debug!("Entry already file");
                            return;
                        }
                    } else {
                        log::error!("Error parsing csv {:?}", self.path);
                    }
                }

                // Insert playlist otherwise.
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
            Ok(CsvEvent::Remove(item)) => {
                let vec_str: Vec<String> = item.into();
                let new_record: StringRecord = vec_str.into();

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
            _ => {}
        }
    }
}
