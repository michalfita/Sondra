#[macro_use(lazy_static)]
extern crate lazy_static;

use std::io::prelude::*;
use walkdir::WalkDir;
use std::time::{Duration, Instant};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufReader, Read};

mod arguments;
mod serializers;
mod photos;

#[derive(PartialEq, Debug)]
struct PhotoFileFilter {
    patterns: Vec<glob::Pattern>,
}

lazy_static! {
    static ref PHOTO_FILE_FILTER: PhotoFileFilter = PhotoFileFilter {
        patterns: vec![
                glob::Pattern::new("*.[Jj][Pp][Gg]").unwrap(),
                glob::Pattern::new("*.[Nn][Ee][Ff]").unwrap(),
            ],
    };
}

impl PhotoFileFilter {
    pub fn get_instance() -> &'static PhotoFileFilter {
        &*PHOTO_FILE_FILTER
    }

    pub fn extension_match(&self, str: &str) -> bool {
        self.patterns.iter().fold(false, |acc, x| acc | x.matches(str))
    }
}

fn dump_json(file_name: &str, photo_collection: &photos::PhotoCollection) {
    use std::path::Path;

    let path = Path::new(file_name);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let serialized = serde_json::to_string(&photo_collection).expect("Serialization failure");

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(serialized.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn main() {
    let opts = arguments::digest();
    let photo_filter = PhotoFileFilter::get_instance();

    //let mut filenames = HashMap::new();
    let mut photo_collection = photos::PhotoCollection::new();

    let started = Instant::now();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner:.yellow} {wide_msg}");
        //.template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})");

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.set_prefix(&format!("[{}/2]", 1));
    pb.set_message(&format!("Processing files in directory '{}'...", opts.directory));

    for entry in WalkDir::new(opts.directory)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .filter(move |e| { photo_filter.extension_match(&e.file_name().to_string_lossy()) } ) {
        // -- This is junk
        // let f_name = String::from(entry.file_name().to_string_lossy());
        // let counter = filenames.entry(f_name.clone()).or_insert(0);
        // *counter += 1;

        // if *counter > 2 {
        //     println!("{}:{}", f_name, *counter);
        // }
        // -- This is the piece from software
        let size = entry.metadata().expect("No metadata for the entry!").len();
        photo_collection.add_file(entry.path().as_os_str(), size);
        pb.inc(1);
    }

    pb.set_prefix(&format!("[{}/2]", 2));
    photo_collection.obtain_hashes(|path| {
        pb.set_message(&format!("Hashing file '{}'...", path.to_str().unwrap()));
        
        let file = File::open(&path).expect("File cannot me opened!");
        let mut buf_reader = BufReader::new(file);
        let mut contents = Vec::new();
        buf_reader.read_to_end(&mut contents).expect("Cannot read the content!");
        
        let mut hasher = blake3::Hasher::new();
        hasher.update(&contents);
        hasher.finalize()
    });

    pb.set_message("Dumping serialization of data...");

    dump_json(&opts.output, &photo_collection);

    pb.finish_with_message(&format!("Done. {} files found, {} potential duplicates identified in {}",
            photo_collection.get_entries_number(),
            photo_collection.get_duplicates_number(),
            HumanDuration(started.elapsed())));
}
