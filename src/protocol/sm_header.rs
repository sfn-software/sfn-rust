use std::io;
// use std::result::Result;
// use std::error::Error;
use std::io::{Read, Write, BufRead, BufReader, Cursor};
use std::net::{TcpStream};
use std::fmt;

extern crate byteorder;
use self::byteorder::{LittleEndian, ReadBytesExt};

extern crate md5;
use self::md5::Digest;


pub struct SMFileHeader {
	pub filename: String,
	pub size: u64,
	pub md5: Option<Digest>,
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

fn read_line(mut stream: impl BufRead) -> io::Result<String> {
	let mut ret: String = String::new();
	stream.read_line(&mut ret)?;
	Ok(ret.trim_end().to_string())
}

fn parse_md5(s: String) -> io::Result<Digest> {
	const MD5_LENGTH_BYTES: usize = 128 / 8;
	if s.len() != MD5_LENGTH_BYTES * 2 {
		panic!("Invalid MD5 hash: {}", s);
	}

	let s = s.as_bytes();
	let mut buf: [u8; MD5_LENGTH_BYTES] = [0x00; MD5_LENGTH_BYTES];
	for i in 0..buf.len() {
		let c1: u8 = parse_hex(s[i*2+0] as char);
		let c2: u8 = parse_hex(s[i*2+1] as char);
		buf[i] = c1 * 16 + c2;
	}

	Ok(Digest(buf))
}

impl SMFileHeader {
	pub fn read_from(mut stream: impl BufRead, with_md5: bool) -> io::Result<SMFileHeader> {
		let filename = read_line(&mut stream)?;
		let size = stream.read_u64::<LittleEndian>()?;
		let md5 = if with_md5 {
			Some( parse_md5( read_line(&mut stream)? )? )
		} else {
			None
		};
		return Ok(SMFileHeader{ filename, size, md5 });
	}
}
