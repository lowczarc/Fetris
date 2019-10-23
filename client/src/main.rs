use fetris_protocol::{ClientRequest, ServerRequest};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::{thread, time};

fn main() -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect("127.0.0.1:3000")?;

    let reader = stream.try_clone().unwrap();
    thread::spawn(move || {
        loop {
            if let Ok(request) = ServerRequest::from_reader(&reader) {
                println!("{:?}", request);
            } else {
                break;
            }
        }
        println!("Invalid package");
    });

    stream.write(&ClientRequest::AskForAGame.into_bytes())?;
    loop {
        thread::sleep(time::Duration::from_secs(1));
    }
    Ok(())
}
