use std::io;
// use std::result::Result;
// use std::error::Error;
use std::io::{Read, Write, BufRead, BufReader, Cursor};
use std::net::{TcpStream};
use std::thread;
use std::fmt;

extern crate byteorder;
use self::byteorder::{LittleEndian, ReadBytesExt};

extern crate md5;
use self::md5::{compute, Digest};


const SFN_FILE: u8 = 0x01;
const SFN_DONE: u8 = 0x02;
const SFN_FILE_WITH_MD5: u8 = 0x03;

const BUFFER_SIZE: usize = 64*1024; // bytes

struct SMFileHeader {
	filename: String,
	size: u64,
	md5: Option<Digest>,
}

impl fmt::Display for SMFileHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.md5 {
			None => write!(f, "[{}, {} byte(s)]", self.filename, self.size),
			Some(md5) => write!(f, "[{}, {} byte(s), MD5: {:?}]", self.filename, self.size, md5),
		}
    }
}

fn parse_hex(c: char) -> u8 {
	match c {
		'0' => 0x00, '1' => 0x01, '2' => 0x02, '3' => 0x03, '4' => 0x04,
		'5' => 0x05, '6' => 0x06, '7' => 0x07, '8' => 0x08, '9' => 0x09,
		'a' => 0x0a, 'b' => 0x0b, 'c' => 0x0c, 'd' => 0x0d, 'e' => 0x0e, 'f' => 0x0f,
		'A' => 0x0a, 'B' => 0x0b, 'C' => 0x0c, 'D' => 0x0d, 'E' => 0x0e, 'F' => 0x0f,
		_ => panic!("Not a hex digit: {}", c),
	}
}

fn read_md5_lf(mut stream: impl Read) -> io::Result<Digest> {
	let mut buf: [u8; 16] = [0x00; 16];

	let mut b2: [u8; 2] = [0x00; 2];
	for i in 0..16 {
		stream.read_exact(&mut b2)?;

		let c1: u8 = parse_hex(b2[0] as char);
		let c2: u8 = parse_hex(b2[1] as char);

		buf[i] = c1 * 16 + c2;
	}

	// skip LF
	let mut b1: [u8; 1] = [0x00; 1];
	stream.read_exact(&mut b1)?;

	Ok(Digest(buf))
}

impl SMFileHeader {
	fn read_from(mut stream: impl BufRead, with_md5: bool) -> io::Result<SMFileHeader> {
		let mut filename: String = String::new();
		stream.read_line(&mut filename)?;
		let filename = filename.trim_end().to_string();

		let size = stream.read_u64::<LittleEndian>()?;

		let md5 = if with_md5 {
			Some(read_md5_lf(&mut stream)?)
		} else {
			None
		};

		return Ok(SMFileHeader{ filename, size, md5 });
	}
}


fn send_files(mut stream: impl Write) -> io::Result<()> {
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
			SFN_DONE => {
				println!("Remote done.");
				return Ok(());
			},
			SFN_FILE => {
				recv_file(&mut stream, false)?;
			},
			SFN_FILE_WITH_MD5 => {
				recv_file(&mut stream, true)?;
			},
			_ => panic!("Unsupported SM opcode: {}", opcode),
		};
	}
}

pub fn handle_client(stream: TcpStream) -> io::Result<()> {
	let stream_clone = stream.try_clone()?;
	let send_thread = thread::spawn(move || {
		send_files(&stream_clone).unwrap();
	});
	let recv_thread = thread::spawn(move || {
		recv_files(&stream).unwrap();
	});

	send_thread.join().unwrap(); // TODO: wtf is this return type
	recv_thread.join().unwrap();
	println!("All done, closing connection.");
	Ok(())
}
