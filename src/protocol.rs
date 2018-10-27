mod sm_header;
use self::sm_header::SMFileHeader;

use std::io;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpStream};
use std::thread;
use std::fs::File;

extern crate byteorder;
use self::byteorder::{ReadBytesExt};


const SFN_FILE: u8 = 0x01;
const SFN_DONE: u8 = 0x02;
const SFN_FILE_WITH_MD5: u8 = 0x03;

const BUFFER_SIZE: usize = 64*1024; // bytes


fn send_files(mut stream: impl Write, files: Vec<String>) -> io::Result<()> {
	for filename in files {
		let mut file = File::open(&filename)?;
		println!("Sending file: {}", filename);

		let mut file = BufReader::new(file);

		let size = std::fs::metadata(&filename)?.len();

		// TODO: remove path to dir
		let header = SMFileHeader{ filename, size, md5: None };
		header.write_with_opcode(&mut stream)?;

		let mut buf = Vec::with_capacity(BUFFER_SIZE);
		loop {
			buf.resize(BUFFER_SIZE, 0x00);
			let count = file.read(&mut buf)?;
			if count == 0 {
				break;
			}

			buf.resize(count, 0x00);
			stream.write(&buf)?;
		}
	}
	println!("Local done.");
	stream.write(&[ SFN_DONE ])?;
	Ok(())
}

fn recv_files(stream: impl Read) -> io::Result<()> {
	fn recv_file(mut stream: impl BufRead, with_md5: bool) -> io::Result<()> {
		let header = SMFileHeader::read_from(&mut stream, with_md5)?;
		println!("Receiving a file: {}", header);

		let mut remain = header.size;
		loop {
			let buf_size: usize = {
				if remain >= (BUFFER_SIZE as u64) { BUFFER_SIZE } else { remain as usize }
			};
			let mut buf = Vec::new();
			buf.resize(buf_size, 0x00);

			let read = stream.read(&mut buf)?;
			// println!("read -> {}", read);
			if read == 0 {
				break;
			}
			remain -= read as u64;
		}

		Ok(())
	}

	let mut stream = BufReader::new(stream);
	loop {
		let opcode = stream.read_u8()?;
		match opcode {
			SFN_FILE => recv_file(&mut stream, false)?,
			SFN_FILE_WITH_MD5 => recv_file(&mut stream, true)?,
			SFN_DONE => {
				println!("Remote done.");
				return Ok(());
			},
			_ => panic!("Unsupported SM opcode: {}", opcode),
		};
	}
}

pub fn handle_client(stream: TcpStream, files: Vec<String>) -> io::Result<()> {
	let stream_clone = stream.try_clone()?;
	let send_thread = thread::spawn(move || {
		send_files(&stream_clone, files).unwrap();
	});
	let recv_thread = thread::spawn(move || {
		recv_files(&stream).unwrap();
	});

	send_thread.join().unwrap(); // TODO: wtf is this return type
	recv_thread.join().unwrap();
	println!("All done, closing connection.");
	Ok(())
}
