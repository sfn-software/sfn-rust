use super::Opcode;

use std::io;
use std::io::{Write, BufRead};
use std::fmt;

extern crate byteorder;
use self::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
extern crate md5;
use self::md5::Digest;


pub struct SMFileHeader {
	pub opcode: Opcode,
	pub filename: String,
	pub size: u64,
	pub md5sum: Option<Digest>,
}

impl fmt::Display for SMFileHeader {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.md5sum {
			None => write!(f, "[{}, {} byte(s)]", self.filename, self.size),
			Some(md5) => write!(f, "[{}, {} byte(s), MD5: {:?}]", self.filename, self.size, md5),
		}
	}
}

/// Parse a hex digit.
///
/// ```
/// # use sfn::protocol::sm_header::parse_hex;
/// assert_eq!(parse_hex('1'), 0x01);
/// assert_eq!(parse_hex('C'), 0x0C);
/// assert_eq!(parse_hex('d'), 0x0D);
/// ```
pub fn parse_hex(c: char) -> u8 {
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

/// Parse a MD5 hash from a string.
///
/// ```
/// # extern crate md5;
/// # use sfn::protocol::sm_header::parse_md5;
/// # fn main() {
/// assert_eq!(
///   parse_md5("881d8cd98f00b204e9800998ecf8427e").unwrap(),
///   md5::Digest([0x88, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8, 0x42, 0x7e])
/// );
/// # }
/// ```
pub fn parse_md5(s: &str) -> io::Result<Digest> {
	const MD5_LENGTH_BYTES: usize = 128 / 8;
	if s.len() != MD5_LENGTH_BYTES * 2 {
		panic!("Invalid MD5 hash: {}", s);
	}

	let s = s.as_bytes();
	let mut buf: [u8; MD5_LENGTH_BYTES] = [0x00; MD5_LENGTH_BYTES];
	for i in 0..buf.len() {
		let c1: u8 = parse_hex(s[i*2]   as char);
		let c2: u8 = parse_hex(s[i*2+1] as char);
		buf[i] = c1 * 16 + c2;
	}

	Ok(Digest(buf))
}

impl SMFileHeader {
	pub fn read_from(mut stream: impl BufRead) -> io::Result<SMFileHeader> {
		let opcode = Opcode::from( stream.read_u8()? );
		if opcode == Opcode::Done {
			return Ok(SMFileHeader{ opcode, filename: String::new(), size: 0, md5sum: None });
		}

		let filename = read_line(&mut stream)?;
		let size = stream.read_u64::<LittleEndian>()?;
		let mut md5 = None;
		if opcode == Opcode::MD5WithFile {
			md5 = Some( parse_md5( &read_line(&mut stream)? )? );
		};
		Ok(SMFileHeader{ opcode, filename, size, md5sum: md5 })
	}

	pub fn write_with_opcode(&self, mut stream: impl Write) -> io::Result<()> {
		stream.write_u8(self.opcode as u8)?;

		stream.write_all(self.filename.as_bytes())?;
		stream.write_u8(0x0A)?;

		stream.write_u64::<LittleEndian>(self.size)?;

		if self.md5sum.is_some() {
			panic!("Not implemented");
		}

		Ok(())
	}
}
