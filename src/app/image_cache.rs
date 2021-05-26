use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossbeam::channel::TryRecvError;
use crossbeam::channel::{Receiver, Sender};
use eframe::egui::{Color32, Vec2};
use image::imageops::FilterType;
use image::{GenericImageView, ImageError};

use crate::app::image_file::{ImageFile, ImageFileError};

type PixelsMessage = Result<(ImageLookup, ImageData), ImageParseError>;
type ImageMessage = (ImageLookup, PathBuf);

pub struct ImageCache {
    size: Arc<Mutex<Vec2>>,
    cache: BTreeMap<ImageLookup, ImageData>,
    pixel_receiver: Receiver<PixelsMessage>,
    image_sender: Sender<ImageMessage>,
    queued: BTreeSet<ImageLookup>,
}

pub struct ImageData {
    size: (u32, u32),
    pub(crate) data: Vec<Color32>,
}

impl ImageData {
    pub fn size_usize(&self) -> (usize, usize) {
        (self.size.0 as usize, self.size.1 as usize)
    }

    pub fn size_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.size.0 as f32,
            y: self.size.1 as f32,
        }
    }
}

#[derive(Debug)]
enum ImageParseError {
    ImageError(ImageError),
    ImageFileError(ImageFileError),
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct ImageLookup {
    pub(crate) index: usize,
}

impl ImageCache {
    pub fn new(size: Vec2) -> ImageCache {
        let (im_tx, im_rx) = crossbeam::channel::bounded::<ImageMessage>(num_cpus::get());
        let (px_tx, px_rx) = crossbeam::channel::bounded::<PixelsMessage>(num_cpus::get());
        let arc = Arc::new(Mutex::new(size));
        for i in 1..=num_cpus::get() {
            let im_rx_clone = im_rx.clone();
            let px_tx_clone = px_tx.clone();
            let arc_clone = arc.clone();
            thread::spawn(move || {
                let thread_num = i;
                println!("started worker thread {}", i);
                loop {
                    match im_rx_clone.try_recv() {
                        Ok((lookup, file)) => {
                            println!("thread {} is trying to deal with {:?}", thread_num, file);
                            match ImageFile::new(file) {
                                Ok(img) => match img.as_image() {
                                    Ok(img) => {
                                        let Vec2 { x: w, y: h } =
                                            { *arc_clone.lock().expect("lock was poisoned") };
                                        let resized =
                                            img.resize(w as u32, h as u32, FilterType::Nearest);
                                        let pixels = resized
                                            .pixels()
                                            .map(|(.., p)| {
                                                Color32::from_rgba_premultiplied(
                                                    p.0[0], p.0[1], p.0[2], p.0[3],
                                                )
                                            })
                                            .collect::<Vec<_>>();
                                        let data = ImageData {
                                            size: resized.dimensions(),
                                            data: pixels,
                                        };
                                        println!(
                                            "thread {} dealt with it successfully",
                                            thread_num
                                        );
                                        let send_result = px_tx_clone.send(Ok((lookup, data)));
                                        if let Err(err) = send_result {
                                            println!("failed to send {:?}", err)
                                        }
                                    }
                                    Err(err) => {
                                        px_tx_clone
                                            .send(Err(ImageParseError::ImageError(err)))
                                            .unwrap_or_else(|err| {
                                                panic!(
                                                    "failed to send error {} from thread {}",
                                                    err, thread_num
                                                )
                                            });
                                    }
                                },
                                Err(err) => {
                                    px_tx_clone
                                        .send(Err(ImageParseError::ImageFileError(err)))
                                        .unwrap_or_else(|err| {
                                            panic!(
                                                "failed to send error {} from thread {}",
                                                err, thread_num
                                            )
                                        });
                                }
                            }
                        }
                        Err(TryRecvError::Disconnected) => break,
                        Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(100)),
                    }
                }
                println!("killing worker thread {}", i);
            });
        }
        ImageCache {
            size: arc,
            cache: BTreeMap::new(),
            pixel_receiver: px_rx,
            image_sender: im_tx,
            queued: BTreeSet::new(),
        }
    }

    pub fn get(&mut self, lookup: ImageLookup, files: &[ImageFile]) -> Option<&ImageData> {
        self.update();
        for i in 0..=(num_cpus::get() / 2) {
            let guess_at_next = ImageLookup {
                index: lookup.index.saturating_add(i),
            };
            if !self.cache.contains_key(&guess_at_next) && self.queued.insert(guess_at_next) {
                self.request(guess_at_next, files);
            }
        }
        if self.queued.contains(&lookup) {
            None
        } else {
            self.cache.get(&lookup)
        }
    }

    pub fn set_size(&mut self, new_size: Vec2) -> bool {
        let Self { size, cache, .. } = self;
        let mut current_size = size.lock().unwrap();
        let changed = *current_size != new_size;
        if changed {
            *current_size = new_size;
            cache.clear();
        }
        changed
    }

    pub fn update(&mut self) {
        while let Ok(process_result) = self.pixel_receiver.try_recv() {
            match process_result {
                Ok((lookup, pixels)) => {
                    self.cache.insert(lookup, pixels);
                    println!("cached {}", lookup.index);
                    self.queued.retain(|q| *q != lookup)
                }
                Err(err) => {
                    println!("error parsing image {:?}", err)
                }
            }
        }
    }

    fn request(&self, request: ImageLookup, files: &[ImageFile]) {
        println!("sending request for {:?}", request);
        match files.get(request.index) {
            None => {
                println!("invalid request occurred with lookup {:?}", request)
            }
            Some(file) => {
                if let Err(err) = self.image_sender.try_send((request, file.as_path())) {
                    println!("failed to send due to {:?}", err)
                }
            }
        }
    }
}
