pub mod sm_header;
use self::sm_header::SMFileHeader;

use std::io;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpStream;
use std::thread;
use std::fs::File;
use std::path::Path;

extern crate byteorder;
extern crate md5;


pub enum ProtocolLevel {
	L1 = 1,
	L3 = 3,
	L4 = 4,
	L5 = 5,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
	File = 0x01,
	Done = 0x02,
	MD5WithFile = 0x03,
	FileWithMD5 = 0x04,
	FileL5 = 0x05,
}

impl From<u8> for Opcode {
	fn from(val: u8) -> Self {
		match val {
			0x01 => Opcode::File,
			0x02 => Opcode::Done,
			0x03 => Opcode::MD5WithFile,
			0x04 => Opcode::FileWithMD5,
			0x05 => Opcode::FileL5,
			_ => panic!("Unsupported opcode: {}", val),
		}
	}

}

const BUFFER_SIZE: usize = 64*1024; // bytes


fn send_files(mut stream: impl Write, files: Vec<String>, conn_protocol: ProtocolLevel) -> io::Result<()> {
	// On L3, we don't try to use MD5_WITH_FILE and fall back to FILE instead.
	let need_md5 = match conn_protocol {
		ProtocolLevel::L1 | ProtocolLevel::L3 => false,
		ProtocolLevel::L4 | ProtocolLevel::L5 => true,
	};
	let opcode = match conn_protocol {
		ProtocolLevel::L1 => Opcode::File,
		ProtocolLevel::L3 => Opcode::File,
		ProtocolLevel::L4 => Opcode::FileWithMD5,
		ProtocolLevel::L5 => Opcode::FileWithMD5,
	};

	for filename in files {
		let file = File::open(&filename)?;
		println!("Sending file: {}", filename);

		let mut file = BufReader::new(file);
		let mut md5 = md5::Context::new();

		let size = std::fs::metadata(&filename)?.len();

		// remove dirpath
		let filename: String = Path::new(&filename).file_name().unwrap().to_str().unwrap().to_string();

		let header = SMFileHeader{ opcode, filename, size, md5sum: None };
		header.write_with_opcode(&mut stream)?;

		let mut buf = Vec::with_capacity(BUFFER_SIZE);
		loop {
			buf.resize(BUFFER_SIZE, 0x00);
			let count = file.read(&mut buf)?;
			if count == 0 {
				break;
			}

			buf.resize(count, 0x00);
			stream.write_all(&buf)?;

			if need_md5 {
				md5.consume(&buf);
			}
		}

		if need_md5 {
			// convert to hex string
			let s = format!("{:x}", md5.compute());
			stream.write_all(s.as_bytes())?;
			stream.write_all(&[b'\n'])?;
		}
	}
	println!("Local done.");
	stream.write_all(&[ Opcode::Done as u8 ])?;
	Ok(())
}

fn recv_files(stream: impl Read) -> io::Result<()> {
	fn recv_file(mut stream: impl BufRead, header: SMFileHeader) -> io::Result<()> {
		println!("Receiving a file: {}", header);

		let mut md5_context = md5::Context::new();
		let has_md5 = match header.opcode {
			Opcode::File => false,
			Opcode::MD5WithFile => true,
			Opcode::FileWithMD5 => true,
			_ => panic!("Unexpected opcode: {:?}", header.opcode),
		};
		let mut expected_md5 = header.md5sum;

		let mut file = File::create(&header.filename)?;

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

			// Write to file
			file.write_all(&buf)?;

			// Calculate MD5 if it makes sense
			if has_md5 {
				md5_context.consume(&buf);
			}
		}

		if header.opcode == Opcode::FileWithMD5 {
			let md5_string = sm_header::read_line(&mut stream)?;
			expected_md5 = Some(sm_header::parse_md5(&md5_string)?);
		}

		if has_md5 {
			let expected_md5 = expected_md5.unwrap();
			println!("Checking MD5 (expected: {:?})", expected_md5);
			let computed_md5 = md5_context.compute();
			if expected_md5 != computed_md5 {
				println!("MD5 mismatch: expected {:?}, got {:?}\n\n", expected_md5, computed_md5);
				panic!("MD5 mismatch");
			}
		}

		Ok(())
	}

	let mut stream = BufReader::new(stream);
	loop {
		let header = SMFileHeader::read_from(&mut stream)?;
		let opcode = header.opcode;
		match opcode {
			Opcode::File => recv_file(&mut stream, header)?,
			Opcode::MD5WithFile => recv_file(&mut stream, header)?,
			Opcode::FileWithMD5 => recv_file(&mut stream, header)?,

			Opcode::Done => {
				println!("Remote done.");
				return Ok(());
			},
			_ => panic!("Unsupported SM opcode: {:?}", opcode),
		};
	}
}

pub fn handle_client(stream: TcpStream, files: Vec<String>) -> io::Result<()> {
	let stream_clone = stream.try_clone()?;
	let send_thread = thread::spawn(move || {
		send_files(&stream_clone, files, ProtocolLevel::L4).unwrap();
	});
	let recv_thread = thread::spawn(move || {
		recv_files(&stream).unwrap();
	});

	send_thread.join().unwrap();
	recv_thread.join().unwrap();
	println!("All done, closing connection.");
	Ok(())
}
