//! A blazingly fast media (audio/image/video) file finder
//! (almost as fast as walkdir).
//!
#![warn(missing_docs)]
#![deny(rustdoc::missing_doc_code_examples)]
#![allow(unused)]
use infer;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use walkdir::WalkDir;

const AUDIO: &str = "audio";
const IMAGE: &str = "image";
const VIDEO: &str = "video";

/// This struct contains the result for a single found file.
/// - `path`: The path of the found file.
/// - `mime`: The file's MIME type.
/// - `result`:
///   - _bool_:
///     - `true`: A file and a media type for it was found.
///     - `false`: A file was found, but no media
///         type could not be found for it.
///   - _io::Error_: Something went wrong while trying to figure out
///         the media type.
pub struct MediaWalkResult {
    /// The path of the found file.
    pub path: String,
    /// The file's MIME type.
    pub mime: String,
    /// - _bool_:
    ///   - `true`: A file and a media type for it was found.
    ///   - `false`: A file was found, but no media
    ///       type could not be found for it.
    /// - _io::Error_: Something went wrong while trying to figure out
    ///       the media type.
    pub result: Result<bool, io::Error>,
}

/// Start walkding through the given directory. Returns a channel of
/// MediaWalkResult structs.
///
/// # Examples
///
/// ```
/// let rx = start_walking(&resource_dir);
/// for received in rx {
///     match received.result {
///         Ok(result) => {
///             if result == true {
///                 println!("A good file: {}", received.path);
///             } else {
///                 println!("Unknown media type: {}", received.path);
///             }
///         }
///         Err(err) => {
///             println!("{}: {:?}", received.path, err);
///         }
///     }
/// }

pub fn start_walking(first_step: &PathBuf) -> Receiver<MediaWalkResult> {
    let (tx, rx) = mpsc::channel();

    let starter = first_step.clone();
    thread::spawn(move || {
        let walker = WalkDir::new(starter).follow_links(true).into_iter();
        for entry_result in walker {
            if let Ok(entry) = entry_result {
                if entry.file_type().is_file() {
                    if let Some(path) = entry.path().to_str() {
                        let mut walk_result = MediaWalkResult {
                            path: path.to_string(),
                            mime: "".to_string(),
                            result: Ok(true),
                        };
                        match infer::get_from_path(path.to_string()) {
                            Ok(Some(info)) => {
                                if info.mime_type().starts_with(AUDIO)
                                    || info.mime_type().starts_with(IMAGE)
                                    || info.mime_type().starts_with(VIDEO)
                                {
                                    walk_result.mime = info.mime_type().to_string();
                                    tx.send(walk_result).unwrap();
                                }
                            }
                            Ok(None) => {
                                // eprintln!("Unknown file type");
                                walk_result.result = Ok(false);
                                tx.send(walk_result).unwrap();
                            }
                            Err(e) => {
                                // eprintln!("Looks like something went wrong");
                                // eprintln!("{}", e);
                                walk_result.result = Err(e);
                                tx.send(walk_result).unwrap();
                            }
                        }
                    }
                }
            }
        }
    });
    return rx;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn it_finds_the_expected_amount_of_files() {
        let mut resource_dir = PathBuf::new();
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            resource_dir.push(manifest_dir);
        }
        resource_dir.push("resources");
        resource_dir.push("test");
        let mut items: Vec<String> = vec![];
        let rx = start_walking(&resource_dir);
        let mut invalid_count = 0;
        for received in rx {
            match received.result {
                Ok(result) => {
                    if result == true {
                        items.push(received.path);
                    } else {
                        println!("Unknown media type: {}", received.path);
                        invalid_count += 1;
                    }
                }
                Err(err) => {
                    println!("{}: {:?}", received.path, err);
                }
            }
        }
        // Real amount is 8 media files, but for now we accept the one Markdown file as well.
        assert_eq!(items.len(), 8);
        assert_eq!(invalid_count, 1);
    }
}
