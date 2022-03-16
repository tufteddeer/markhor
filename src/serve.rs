use std::{
    error::Error,
    fs,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use log::error;
use log::{info, warn};
use percent_encoding::percent_decode_str;
use rayon::ThreadPoolBuilder;
use std::io::Read;
use std::io::Write;

pub fn serve_files<A>(address: A, dir: &'static Path) -> Result<(), Box<dyn Error>>
where
    A: ToSocketAddrs,
{
    let thread_pool = ThreadPoolBuilder::new().num_threads(10).build().unwrap();
    let listener = TcpListener::bind(address)?;

    info!("Running at http://{:#?}", listener.local_addr()?);
    info!("Serving directory '{}'", dir.display());

    let dir = Arc::new(Mutex::new(dir));
    for stream in listener.incoming() {
        let dir = dir.clone();
        match stream {
            Ok(stream) => {
                thread_pool.spawn(move || {
                    let dir = dir.lock().unwrap();
                    if let Err(error) = handle_conn(stream, *dir) {
                        error!("Error while handling request: {}", error);
                    }
                });
            }
            Err(err) => error!("Connection failed: {}", err),
        }
    }

    Ok(())
}

fn handle_conn<P>(mut stream: TcpStream, dir: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let mut buffer = [0; 1024];

    let _ = stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);

    let mut parts = request.split(' ');
    let method = parts.next().unwrap();
    let path_enc = parts.next().unwrap();

    let path = percent_decode_str(path_enc).decode_utf8().unwrap();

    if method.to_lowercase() != "get" {
        write_response(&mut stream, "Method not allowed".as_bytes(), 405)?;
        return Ok(());
    }
    let mut path = path.strip_prefix('/').unwrap();

    if path.is_empty() {
        path = "index.html";
    }

    let mut pathbuf = PathBuf::from(dir.as_ref());
    pathbuf.push(path);
    let path = pathbuf.as_path();

    if !path.is_file() {
        warn!("Could not find {}", path.display());
        write_response(&mut stream, "Not found".as_bytes(), 404)?;
        return Ok(());
    }

    // prevent http://localhost:8080//etc/somefile.txt style requests
    if PathBuf::from(path).is_absolute() {
        warn!("Absolute path requested: {}", path.display());
        warn!("Access denied.");
        write_response(&mut stream, "Forbidden".as_bytes(), 403)?;
        return Ok(());
    }

    let content = fs::read(path)?;

    write_response(&mut stream, &content, 200)?;

    Ok(())
}

fn write_response(
    stream: &mut TcpStream,
    content: &[u8],
    status: u32,
) -> Result<(), std::io::Error> {
    let len = content.len();
    write!(stream, "HTTP/1.1 {status}\r\nContent-Length: {len}\r\n\r\n")?;
    stream.write_all(content)
}
