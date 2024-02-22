use std::fs;
use std::io::Read;
use std::io::Write;
use std::fs::OpenOptions;
use std::future::Future;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::process::Stdio;
use std::process::Command;
use std::path::Path;
use notify_rust::Notification;
use rodio::{ Decoder, OutputStream, Sink };

use crate::SOCKET;
use crate::CWD;

use crate::media::play;
use crate::media::pause;
use crate::media::queue;
use crate::media::stop;

async fn fork<F, Fut>(callback: F)
where
	F: FnOnce() -> Fut,
	Fut: Future<Output = ()>,
{
	// Fork process
	match unsafe { libc::fork() } {
		-1 => {
			// Failed
			eprintln!("Failed to fork");
			return;
		},
		0 => {
			// Child process
			if unsafe { libc::setsid() } == -1 {
				eprintln!("Failed to detach from terminal");
				return;
			}

			println!("Forked!");
			// Change working directory to muse
			let cwd = Path::new(CWD);
			if !cwd.exists() { fs::create_dir_all(&cwd).unwrap() }

			if let Err(err) = std::env::set_current_dir(&cwd) {
				eprintln!("Failed to change working directory to {}", CWD);
				return;
			}

			// Redirect standard I/O to /dev/null
			let dev_null = Command::new("true")
				.stdin(Stdio::null())
				.stdout(Stdio::null())
				.stderr(Stdio::null())
				.spawn();

			match dev_null {
				Err(err) => {
					eprintln!("Failed to redirect standard I/O: {}", err);
					return;
				},
				Ok(_) => callback().await,
			}
		},
		_ => {
			// Exit parent process
			std::process::exit(0);
		}
	}
}

pub async fn start_daemon() {
	fork(|| async {
		let socket = Path::new(SOCKET);

		if socket.exists() { fs::remove_file(&socket).unwrap() }

		let listener = match UnixListener::bind(&socket) {
			Err(e) => {
				eprintln!("Error binding socket: {}", e);
				return;
			},
			Ok(stream) => stream,
		};

		// Set socket permissions
		let metadata = fs::metadata(SOCKET).unwrap();
		let mut permissions = metadata.permissions();
		permissions.set_mode(0o666);
		if let Err(e) = fs::set_permissions(socket, permissions) {
			eprintln!("Error setting permissions on file: {}", SOCKET);
			return;
		}

		let mut file = OpenOptions::new()
			.write(true)
			.append(true)
			.create(true)
			.open(format!("{}/muse.log", CWD)).unwrap();

		// Successfully opened socket, initialize sink
		// Get output stream handle to default physical device
		let (_stream, stream_handle) = OutputStream::try_default().unwrap();
		// Define sink
		let sink = Sink::try_new(&stream_handle).unwrap();

		// Listen for streams
		for stream in listener.incoming() {
			match stream {
				Err(e) => {
					eprintln!("Error connecting: {}", e);
					return;
				},
				Ok(mut stream) => {
					let mut response = String::new();
					stream.read_to_string(&mut response);

					let parts: Vec<&str> = response.split_whitespace().collect();

					if let Some(action) = parts.get(0) {
						match *action {
							"play" => {
								file.write_all(b"playing");
								play(&sink);
							},
							"pause" => {
								file.write_all(b"pausing");
								pause(&sink);
							},
							"enqueue" => {
								if let Some(song) = parts.get(1) {
									file.write_all(format!("enqueueing: {}", response).as_bytes());
									queue(&sink, "/home/user/codes/songs/01 Psycho CEO.mp3");
								} else {
									file.write_all(b"no song provided");
								}
							},
							_ => {
								file.write_all(format!("unknown command: {}", response).as_bytes());
							},
						}
					} else {
						file.write_all(b"no command provided");
					}
				}
			}
		}
	}).await;
}

