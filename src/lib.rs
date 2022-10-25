use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

const VECTOR_SIZE: u32 = 10;

pub fn start_walking() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for n in 1..(VECTOR_SIZE + 1) {
            let sendable = format!("Iteration #{}", n);
            tx.send(sendable).unwrap();
        }
    });
    return rx;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut items: Vec<String> = vec![];
        let rx = start_walking();
        for received in rx {
            items.push(received);
        }
        println!(">>> Got {} messages", items.len());
        let expected = VECTOR_SIZE as usize;
        assert_eq!(items.len(), expected);
    }
}
