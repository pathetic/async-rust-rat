use std::io::Read;

pub fn read_console_buffer<I>(stream: &mut I) -> core::result::Result<Vec<u8>, ()> where I: Read {
    let mut buffer = [0_u8; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            if size == 0 {
                return Err(());
            }

            let mut vect: Vec<u8> = Vec::new();
            for v in &buffer[0..size] {
                vect.push(*v);
            }

            Ok(vect)
        }
        Err(_) => { Err(()) }
    }
}