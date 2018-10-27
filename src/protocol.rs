mod sm_header;
use self::sm_header::SMFileHeader;

use std::io;
// use std::result::Result;
// use std::error::Error;
use std::io::{Read, Write, BufRead, BufReader, Cursor};
use std::net::{TcpStream};
use std::thread;
// use std::fmt;
use std::fs::File;

extern crate byteorder;
use self::byteorder::{LittleEndian, ReadBytesExt};

extern crate md5;
use self::md5::{compute, Digest};


const SFN_FILE: u8 = 0x01;
const SFN_DONE: u8 = 0x02;
const SFN_FILE_WITH_MD5: u8 = 0x03;

const BUFFER_SIZE: usize = 64*1024; // bytes


fn send_files(mut stream: impl Write, files: Vec<String>) -> io::Result<()> {
	for filename in files {
		let mut file = File::open(&filename)?;
		println!("Sending file: {}", filename);

		let mut file = BufReader::new(file);

		// TODO
	}
	println!("Local done.");
	stream.write(&[ SFN_DONE ])?;
	Ok(())
}

fn recv_files(mut stream: impl Read) -> io::Result<()> {
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
