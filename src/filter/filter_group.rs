use crate::errors::Error;
use crate::filter::EntryFilter;
use crate::youtube_feed::Entry;

use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use file_minidb::{
    column::Column, serializer::Serializable, table::Table, types::ColumnType, values::Value,
};

#[derive(Clone, Debug)]
pub struct EntryFilterGroup {
    filters: Vec<EntryFilter>,
}

impl EntryFilterGroup {
    /// Create a new, empty filter group.
    pub fn new() -> EntryFilterGroup {
        EntryFilterGroup { filters: vec![] }
    }

    /// Add a filter to the group.
    pub fn add(&mut self, filter: EntryFilter) {
        self.filters.push(filter)
    }

    /// Removes a filter of the filter group.
    pub fn remove(&mut self, filter: EntryFilter) {
        self.filters = self
            .filters
            .clone()
            .into_iter()
            .filter(|f| f.clone() != filter)
            .collect();
    }

    /// Check if any filter matches.
    pub fn matches_any(&self, entry: &Entry) -> bool {
        self.filters
            .par_iter()
            .find_any(|f| f.matches(entry))
            .is_some()
    }

    /// Get the filters.
    pub fn get_filters(&self) -> Vec<EntryFilter> {
        self.filters.clone()
    }

    /// Parses the filter group from the file at the given path.
    /// The file must not exist, but it is created and a empty filter group will be returned.
    /// An error will be returned if the file could not be parsed.
    pub fn get_from_path(path: &PathBuf) -> Result<Self, Error> {
        let filter_file_res = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone());

        if let Ok(mut filter_file) = filter_file_res {
            return EntryFilterGroup::get_from_file(path, &mut filter_file);
        } else {
            return Err(Error::general_filter("opening", &path.to_string_lossy()));
        }
    }

    /// Parses the filter group from the given file.
    /// The file must not exist, but it is created and a empty filter group will be returned.
    /// An error will be returned if the file could not be parsed.
    fn get_from_file(path: &PathBuf, filter_file: &mut File) -> Result<Self, Error> {
        let mut group = EntryFilterGroup::new();

        let mut contents = String::new();
        if filter_file.read_to_string(&mut contents).is_ok() {
            if contents.is_empty() {
                let column_title = Column::new("title_filter", ColumnType::String);
                let column_channel = Column::new("channel_filter", ColumnType::String);
                let table = Table::new(vec![column_title, column_channel]).unwrap();
                let res = write!(filter_file, "{}", table.serialize());

                if res.is_err() {
                    return Err(Error::general_filter("writing", &path.to_string_lossy()));
                }
            } else {
                let table_res = Table::deserialize(contents);

                if let Err(_e) = table_res {
                    return Err(Error::parsing_filter(&path.to_string_lossy()));
                }

                let table = table_res.unwrap();

                let entries = table.get_entries();

                for entry in entries {
                    let values: Vec<Value> = entry.get_values();
                    let title_filter: Value = values[0].clone();
                    let channel_filter: Value = values[1].clone();

                    let new_filter = EntryFilter::new(
                        &(String::try_from(title_filter).unwrap()),
                        &(String::try_from(channel_filter).unwrap()),
                    );

                    if let Err(e) = new_filter {
                        return Err(e);
                    }

                    group.add(new_filter.unwrap());
                }
            }
            return Ok(group);
        } else {
            return Err(Error::general_filter("reading", &path.to_string_lossy()));
        }
    }

    /// Writes the filters into the given file at the given path.
    /// The file must not exist, but it is created if it does not exist.
    pub fn write_to_path(&self, path: &PathBuf) -> Result<(), Error> {
        let filter_file_res = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.clone());

        if let Ok(mut filter_file) = filter_file_res {
            self.write_to_file(path, &mut filter_file)
        } else {
            Err(Error::general_filter("opening", &path.to_string_lossy()))
        }
    }

    fn write_to_file(&self, path: &PathBuf, filter_file: &mut File) -> Result<(), Error> {
        let column_title = Column::new("title_filter", ColumnType::String);
        let column_channel = Column::new("channel_filter", ColumnType::String);
        let mut table = Table::new(vec![column_title, column_channel]).unwrap();

        for filter in &self.filters {
            table
                .insert(vec![
                    filter.get_title_filter_string().into(),
                    filter.get_channel_filter_string().into(),
                ])
                .expect("Could not append to table");
        }

        let write_res = write!(filter_file, "{}", table.serialize());

        if write_res.is_err() {
            Err(Error::general_filter("writing", &path.to_string_lossy()))
        } else {
            Ok(())
        }
    }
}
