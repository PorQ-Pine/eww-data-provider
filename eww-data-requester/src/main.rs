use std::env;
use std::fs;
use std::io::{self, BufReader, BufRead};
use std::os::unix::net::UnixListener;
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <socket_name>", args[0]);
        std::process::exit(1);
    }

    let socket_name = &args[1];
    let dir_path = "/tmp/eww_data";
    let socket_path = format!("{}/{}.socket", dir_path, socket_name);
    let path = Path::new(&socket_path);

    fs::create_dir_all(dir_path)?;

    if path.exists() {
        eprintln!("Removing existing socket: {}", socket_path);
        fs::remove_file(path)?;
    }

    // eprintln!("Listening on socket: {}", socket_path);
    let listener = UnixListener::bind(path)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                while reader.read_line(&mut line)? > 0 {
                    print!("{}", line);
                    line.clear();
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}
