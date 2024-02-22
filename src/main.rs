use std::env;
use std::fs::File;
use std::io::{ self, Write, BufRead, BufReader, Error };
use tokio::io::{ AsyncBufReadExt, AsyncWriteExt };
use rodio::{ Decoder, OutputStream, Sink };
use std::os::unix::net::UnixStream;
use std::path::Path;

mod daemon;
mod media;
use daemon::start_daemon;

macro_rules! error {
	($msg:expr) => {
		Err(std::io::Error::new(std::io::ErrorKind::Other, $msg))
	};
}

pub static CWD: &'static str = "/var/run/muse";
pub static SOCKET: &'static str = "/var/run/muse/muse.socket";

async fn prompt(prompt: &str) -> String {
	// Prompt user
	print!("{}", prompt);
	// Flush
	io::stdout().flush().unwrap();

	let mut input = String::new();

	// Read line
	io::stdin().read_line(&mut input).expect("Failed to read line");
	return input.trim().to_lowercase()
}

fn send(message: &str) {
	let socket = Path::new(SOCKET);

	let mut stream = match UnixStream::connect(socket) {
		Err(e) => {
			eprintln!("Error connecting to socket {}: {}", SOCKET, e);
			return;
		},
		Ok(stream) => stream,
	};

	match stream.write_all(message.as_bytes()) {
		Err(e) => {
			eprintln!("Error writing to stream: {}", e);
		},
		Ok(_) => {},
	};
}

#[tokio::main]
async fn main() {
	// Get arguments
	let args: Vec<String> = env::args().collect();

	// Check if an argument was supplied
	if args.len() < 2 {
		println!("usage: {} <command>

Commands:
  start
    Starts muse daemon.
  stop
    Stops muse daemon.
  play
    Resumes from queue, if possible.
  pause
    Pauses, if possible.
  queue
    Lists current queue.
  enqueue <filename>
    Adds <filename> to queue.
  dequeue <filename>
    Removes first instance of <filename> from queue.
  clear", args[0]);
		return;
	}

	match &args[1][..] {
		"start" => {
			tokio::spawn(start_daemon());

			// Give the daemon a maximum of 5 seconds to initialize
			std::thread::sleep(std::time::Duration::from_secs(5));
			return;
		},
		"stop" => {
			send("stop");
			return;
		},
		"play" => {
			send("play");
			return;
		},
		"pause" => {
			send("pause");
			return;
		},
		"enqueue" => {
			send(&format!("enqueue {}", args[2])[..]);
		},
		&_ => return,
	}
}