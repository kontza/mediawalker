use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use walkdir::WalkDir;

pub fn start_walking(first_step: &PathBuf) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();

    let starter = first_step.clone();
    thread::spawn(move || {
        let walker = WalkDir::new(starter).follow_links(true).into_iter();
        for entry_result in walker {
            if let Ok(entry) = entry_result {
                if entry.file_type().is_file() {
                    if let Some(path) = entry.path().to_str() {
                        tx.send(path.to_string()).unwrap();
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
        for received in rx {
            items.push(received);
        }
        // Real amount is 8 media files, but for now we accept the one Markdown file as well.
        assert_eq!(items.len(), 9);
    }
}
