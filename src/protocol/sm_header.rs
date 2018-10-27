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

fn read_md5_lf(mut stream: impl Read) -> io::Result<Digest> {
	const MD5_LENGTH_BYTES: usize = 128 / 8;
	let mut buf: [u8; MD5_LENGTH_BYTES] = [0x00; MD5_LENGTH_BYTES];

	let mut b2: [u8; 2] = [0x00; 2];
	for i in 0..buf.len() {
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
	pub fn read_from(mut stream: impl BufRead, with_md5: bool) -> io::Result<SMFileHeader> {
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
