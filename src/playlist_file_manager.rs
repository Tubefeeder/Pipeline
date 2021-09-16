use std::{convert::TryFrom, fs::OpenOptions, path::PathBuf};

use csv::{ReaderBuilder, StringRecord, WriterBuilder};

use tf_join::{AnyVideo, Joiner};
use tf_observer::Observer;
use tf_playlist::{PlaylistEvent, PlaylistManager};

pub(crate) struct PlaylistFileManager {
    playlist_manager: PlaylistManager<String, AnyVideo>,
    playlist_name: String,
    path: PathBuf,
    joiner: Joiner,
}

impl PlaylistFileManager {
    pub fn new(
        path: &PathBuf,
        playlist_manager: PlaylistManager<String, AnyVideo>,
        playlist_name: String,
        joiner: Joiner,
    ) -> Self {
        let mut manager = Self {
            playlist_manager: playlist_manager.clone(),
            playlist_name,
            path: path.clone(),
            joiner,
        };

        manager.fill_videos();
        manager
    }

    fn fill_videos(&mut self) {
        // TODO: Insert into video store
        log::debug!("Filling in the video playlist from {:?}", self.path);
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

                let video_res = AnyVideo::try_from(items.as_slice());

                if video_res.is_ok() {
                    let video = video_res.unwrap();
                    let join_video = self.joiner.upgrade_video(&video);
                    self.playlist_manager
                        .toggle(&self.playlist_name.clone(), &join_video);
                } else {
                    log::error!("Error parsing video with csv {:?}", items);
                }
            } else {
                log::error!("Error parsing video csv");
            }
        }
    }
}

impl Observer<PlaylistEvent<AnyVideo>> for PlaylistFileManager {
    fn notify(&mut self, message: PlaylistEvent<AnyVideo>) {
        match message {
            PlaylistEvent::Add(video) => {
                let new_record: StringRecord = Vec::<String>::from(video).into();

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
                            log::debug!("Video already in playlist file");
                            return;
                        }
                    } else {
                        log::error!("Error parsing playlist csv");
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
            PlaylistEvent::Remove(video) => {
                let new_record: StringRecord = Vec::<String>::from(video).into();

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
