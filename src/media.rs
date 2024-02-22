use std::fs::File;
use std::io::{ self, Write, BufRead, BufReader, Error, ErrorKind };
use rodio::{ Decoder, OutputStream, Sink };

fn decode(filename: &str) -> Result<Decoder<BufReader<File>>, Error> {
	// Decode the file
	fn mp3(reader: BufReader<File>) -> Decoder<BufReader<File>> {
		return Decoder::new(reader).unwrap();
	}

	// Open the file
	let file = File::open(filename).unwrap();
	let reader = BufReader::new(file);

	// Determine file extension and decode accordingly
	match filename.split(".").last().unwrap() {
		"mp3" => return Ok(mp3(reader)),
		&_ => return Err(Error::new(ErrorKind::Other, format!("Unknown file extension encountered when decoding {}", filename))),
	}
}

pub fn play(sink: &Sink) {
	sink.play();
}

pub fn pause(sink: &Sink) {
	sink.pause();
}

pub fn queue(sink: &Sink, filename: &str) {
	sink.append(decode(filename).unwrap());
}

pub fn stop(sink: &Sink) {
	sink.stop();
}