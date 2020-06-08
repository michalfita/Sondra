use std::rc::Rc;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::ffi::OsStr;
use serde::{Serialize, Deserialize};
use blake3::Hash;
use crate::serializers::SerializableHash;

#[derive(Serialize, Deserialize)]
pub struct PhotoFile {
    path: PathBuf,
    size: u64, // std::fs::Metadata::len() returns u64
    hash: Option<SerializableHash>,
}

#[derive(Serialize, Deserialize)]
pub struct PhotoElement {
    stem: Rc<String>, // photo name w/o extension
    jpg: Option<PhotoFile>,
    raw: Option<PhotoFile>,
}

#[derive(Serialize, Deserialize)]
pub struct PhotoDuplicates {
    jpg: Vec<PhotoFile>,
    raw: Vec<PhotoFile>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PhotoCollection {
    file_map: BTreeMap<Rc<String>, PhotoElement>,
    duplicates: BTreeMap<Rc<String>, Vec<PhotoElement>>,
}

impl PhotoFile {
    pub fn new(path: &OsStr, size: u64) -> Self {
        PhotoFile {
            path: PathBuf::from(path),
            size,
            hash: None
        }
    }

    fn add_hash(&mut self, hash: SerializableHash) {
        self.hash = Some(hash);
    }
}

impl PhotoElement {
    pub fn new(stem: Rc<String>) -> Self {
        Self {
            stem,
            jpg: None,
            raw: None,
        }
    }

    pub fn add_jpg(&mut self, p: PhotoFile) {
        self.jpg = Some(p);
    }

    pub fn add_raw(&mut self, p: PhotoFile) { // TODO: whether it overwrites?
        self.raw = Some(p);
    }

    /// Call closure `F` to generate hash for a `PhotoFile` if present
    fn obtain_hashes<F: Fn(&PathBuf) -> blake3::Hash>(&mut self, get_hash: F)  {
        match &mut self.jpg {
            Some(jpg) => { 
                let hash_value = get_hash(&jpg.path).into();
                jpg.add_hash(hash_value)
            },
            None => {}
        };
        match &mut self.raw {
            Some(raw) => { 
                let hash_value = get_hash(&raw.path).into();
                raw.add_hash(hash_value)
            },
            None => {}
        };
    }
}

impl PhotoCollection {
    /// Constructs a new `PhotoCollection`
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Adds a new entry into the `PhotoCollection`
    pub fn add_file(&mut self, path: &OsStr, size: u64) {
        let pf = PhotoFile::new(path, size);
        let stem_string_rc = Rc::new(pf.path.file_stem().expect("Filename stem extraction failure").to_str().unwrap_or("").to_string());

        let element = if self.file_map.contains_key(&stem_string_rc) {
            let duplicates = self.duplicates.entry(stem_string_rc.clone()).or_insert(Vec::new());
            duplicates.push(PhotoElement::new(stem_string_rc));
            duplicates.last_mut().unwrap()
        } else {
            self.file_map.entry(stem_string_rc.clone())
                         .or_insert_with(|| PhotoElement::new(stem_string_rc))
        };
        
        match pf.path.extension().expect("No extension in filename").to_string_lossy().to_lowercase().as_ref() {
            "jpg" => element.add_jpg(pf),
            "nef" => element.add_raw(pf),
            _ => panic!("Unsupported extension")
        }
    }

    /// Calls closure `F` to generate hashes for each `PhotoElement` held in `PhotoCollection`
    pub fn obtain_hashes<F: Fn(&PathBuf) -> blake3::Hash>(&mut self, get_hash: F) {
        self.file_map.iter_mut().for_each(|(_, photo_elem)| {
            photo_elem.obtain_hashes(&get_hash);
        });
        self.duplicates.iter_mut().for_each(|(_, dupl_vec)| {
            dupl_vec.iter_mut().for_each(|photo_elem| {
                photo_elem.obtain_hashes(&get_hash);
            });
        });
    }

    /// Returns the number of entries in the `PhotoCollection`
    pub fn get_entries_number(&self) -> usize {
        self.file_map.len()
    }

    /// Returns the number of duplicates in the `PhotoCollection`
    pub fn get_duplicates_number(&self) -> usize {
        self.duplicates.len()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use spectral::prelude::*;

    #[test]
    fn test_add_jpg_file() {
        let mut phc = PhotoCollection::new();
        phc.add_file(OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.jpg"), 4096);

        assert_eq!(phc.file_map.is_empty(), false);
        let key = Rc::new("Policji".to_owned());
        assert_eq!(phc.file_map.contains_key(&key), true);
        let element = phc.file_map.get(&key).unwrap();
        assert_eq!(element.stem, key);
        assert_eq!(element.jpg.as_ref().unwrap().path, OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.jpg"));
        assert_that(&element.jpg.as_ref().unwrap().size).is_equal_to(4096);
        assert_that(&phc.get_entries_number()).is_equal_to(1);
        assert_that(&phc.get_duplicates_number()).is_equal_to(0);
    }

    #[test]
    fn test_add_raw_file() {
        let mut phc = PhotoCollection::new();
        phc.add_file(OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.nef"), 8192);

        assert_eq!(phc.file_map.is_empty(), false);
        let key = Rc::new("Policji".to_owned());
        assert_eq!(phc.file_map.contains_key(&key), true);
        let element = phc.file_map.get(&key).unwrap();
        assert_eq!(element.stem, key);
        assert_eq!(element.raw.as_ref().unwrap().path, OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.nef"));
        assert_that(&element.raw.as_ref().unwrap().size).is_equal_to(8192);
        assert_that(&phc.get_entries_number()).is_equal_to(1);
        assert_that(&phc.get_duplicates_number()).is_equal_to(0);
    }

    #[test]
    fn test_add_jpg_duplicate() {
        let mut phc = PhotoCollection::new();
        phc.add_file(OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.jpg"), 4096);
        phc.add_file(OsStr::new("C:\\Kij\\W\\Oko\\Policji.jpg"), 4096);

        assert_eq!(phc.file_map.is_empty(), false);
        let key = Rc::new("Policji".to_owned());
        assert_eq!(phc.file_map.contains_key(&key), true);

        let element = phc.file_map.get(&key).unwrap();
        assert_eq!(element.stem, key);
        assert_eq!(element.jpg.as_ref().unwrap().path, OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.jpg"));
        assert_eq!(element.jpg.as_ref().unwrap().size, 4096);

        let opt_dupl_vec = phc.duplicates.get(&key);
        assert!(opt_dupl_vec.is_some());
        let dupl_vec = opt_dupl_vec.unwrap();
        assert_eq!(dupl_vec.len(), 1);
        assert_that(&phc.get_entries_number()).is_equal_to(1);
        assert_that(&phc.get_duplicates_number()).is_equal_to(1);
    }

    #[test]
    fn test_obtain_hashes() {
        use std::str::FromStr;

        let mut phc = PhotoCollection::new();
        phc.add_file(OsStr::new("C:\\Chuj\\W\\Dupę\\Policji.jpg"), 4096);
        phc.add_file(OsStr::new("C:\\Kij\\W\\Oko\\Policji.jpg"), 4096);

        phc.obtain_hashes(|path| {
            assert_that(path).matches(|x| {
                (*x == PathBuf::from_str("C:\\Chuj\\W\\Dupę\\Policji.jpg").unwrap()) 
                || (*x == PathBuf::from_str("C:\\Kij\\W\\Oko\\Policji.jpg").unwrap())
            });
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"foobarindeadbeef");
            hasher.finalize()
        });

        let key = Rc::new("Policji".to_owned());
        let element = phc.file_map.get(&key).unwrap();
        assert_that(&element.jpg.as_ref().unwrap().hash).is_some().matches(|x| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"foobarindeadbeef");
            *x == hasher.finalize()
        });

        let opt_dupl_vec = phc.duplicates.get(&key);
        let dupl_vec = opt_dupl_vec.unwrap();
        assert_that(&dupl_vec[0].jpg.as_ref().unwrap().hash).is_some().matches(|x| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"foobarindeadbeef");
            *x == hasher.finalize()
        });
    }
}