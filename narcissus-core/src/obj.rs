use std::io::Read;

use fast_float::parse_partial;

const MAX_LINE_SIZE: usize = 8 * 1024;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(inner: std::io::Error) -> Self {
        Error::Io(inner)
    }
}

pub trait Visitor {
    /// Parsed a vertex position, `w` is optional and defaults to `1.0`.
    fn visit_position(&mut self, x: f32, y: f32, z: f32, w: f32);

    /// Parsed a vertex texcoord, `v`, and `w` are optional and default to `0.0`.
    fn visit_texcoord(&mut self, u: f32, v: f32, w: f32);

    /// Parsed a vertex normal. All components are required, they are not necessarily normalized.
    fn visit_normal(&mut self, x: f32, y: f32, z: f32);

    /// Parsed a face. Each element of `indices` is comprised of three components, the position index, texcoord index,
    /// and normal index, respectively.
    ///
    /// Indices are 1-based. Negative indices are not supported.
    /// The texcoord index is optional and will be zero if not present in the source file.
    ///
    /// `indices.len()` must be greater than or equal to three.
    fn visit_face(&mut self, indices: &[(i32, i32, i32)]);
    fn visit_object(&mut self, name: &str);
    fn visit_group(&mut self, name: &str);
    fn visit_smooth_group(&mut self, group: i32);
}

/// Very basic obj parser.
pub struct Parser<T>
where
    T: Read,
{
    reader: T,
    pos: usize,
    cap: usize,
    buf: Box<[u8; MAX_LINE_SIZE]>,
}

impl<T> Parser<T>
where
    T: Read,
{
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            pos: 0,
            cap: 0,
            buf: Box::new([0; MAX_LINE_SIZE]),
        }
    }

    pub fn visit<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), Error> {
        fn parse_line<V: Visitor>(line: &[u8], visitor: &mut V) -> Result<(), Error> {
            debug_assert!(!line.is_empty());

            let mut i = 0;

            #[inline(always)]
            fn consume(i: &mut usize, line: &[u8], c: u8) -> bool {
                if line.get(*i) == Some(&c) {
                    *i += 1;
                    true
                } else {
                    false
                }
            }

            #[inline(always)]
            fn expect(i: &mut usize, line: &[u8], c: u8) -> Result<(), Error> {
                if consume(i, line, c) {
                    Ok(())
                } else {
                    Err(Error::Parse)
                }
            }

            #[inline(always)]
            fn parse_f32(i: &mut usize, line: &[u8]) -> Result<f32, Error> {
                if let Ok((x, digits)) = parse_partial(&line[*i..]) {
                    *i += digits;
                    return Ok(x);
                }
                Err(Error::Parse)
            }

            #[inline(always)]
            fn parse_f32_opt(i: &mut usize, line: &[u8], default: f32) -> f32 {
                if let Ok((x, digits)) = parse_partial(&line[*i..]) {
                    *i += digits;
                    x
                } else {
                    default
                }
            }

            #[inline(always)]
            fn parse_i32(i: &mut usize, line: &[u8]) -> i32 {
                let mut consumed = 0;
                let mut acc = 0;
                for &c in &line[*i..] {
                    let c = c.wrapping_sub(b'0');
                    if c > 9 {
                        break;
                    }
                    acc = acc * 10 + c as i32;
                    consumed += 1;
                }
                *i += consumed;
                acc
            }

            #[inline(always)]
            fn parse_vertex(i: &mut usize, line: &[u8]) -> Result<(i32, i32, i32), Error> {
                let vertex_index = parse_i32(i, line);
                if vertex_index == 0 {
                    return Err(Error::Parse);
                }
                match line.get(*i) {
                    Some(&b'/') => {
                        *i += 1;
                        let texcoord_index = parse_i32(i, line);
                        expect(i, line, b'/')?;
                        let normal_index = parse_i32(i, line);
                        Ok((vertex_index, texcoord_index, normal_index))
                    }
                    Some(&b' ') => Ok((vertex_index, 0, 0)),
                    _ => Err(Error::Parse),
                }
            }

            let mut vertices = Vec::with_capacity(4);

            match line[i] {
                b'v' => {
                    i += 1;
                    match line.get(i) {
                        Some(b' ') => {
                            i += 1;
                            let x = parse_f32(&mut i, line)?;
                            expect(&mut i, line, b' ')?;
                            let y = parse_f32(&mut i, line)?;
                            expect(&mut i, line, b' ')?;
                            let z = parse_f32(&mut i, line)?;
                            consume(&mut i, line, b' ');
                            let w = parse_f32_opt(&mut i, line, 1.0);
                            visitor.visit_position(x, y, z, w)
                        }
                        Some(b't') => {
                            i += 1;
                            expect(&mut i, line, b' ')?;
                            let u = parse_f32(&mut i, line)?;
                            consume(&mut i, line, b' ');
                            let v = parse_f32_opt(&mut i, line, 0.0);
                            consume(&mut i, line, b' ');
                            let w = parse_f32_opt(&mut i, line, 0.0);
                            visitor.visit_texcoord(u, v, w)
                        }
                        Some(b'n') => {
                            i += 1;
                            expect(&mut i, line, b' ')?;
                            let x = parse_f32(&mut i, line)?;
                            expect(&mut i, line, b' ')?;
                            let y = parse_f32(&mut i, line)?;
                            expect(&mut i, line, b' ')?;
                            let z = parse_f32(&mut i, line)?;
                            visitor.visit_normal(x, y, z)
                        }
                        _ => return Err(Error::Parse),
                    }
                }
                b'f' => {
                    i += 1;
                    expect(&mut i, line, b' ')?;
                    vertices.clear();

                    let vertex0 = parse_vertex(&mut i, line)?;
                    expect(&mut i, line, b' ')?;
                    let vertex1 = parse_vertex(&mut i, line)?;
                    expect(&mut i, line, b' ')?;
                    let vertex2 = parse_vertex(&mut i, line)?;

                    vertices.reserve(3);
                    vertices.push(vertex0);
                    vertices.push(vertex1);
                    vertices.push(vertex2);

                    while consume(&mut i, line, b' ') {
                        let vertex_n = parse_vertex(&mut i, line)?;
                        vertices.push(vertex_n);
                    }

                    visitor.visit_face(&vertices)
                }
                b's' => {
                    i += 1;
                    expect(&mut i, line, b' ')?;
                    let group = parse_i32(&mut i, line);
                    visitor.visit_smooth_group(group)
                }
                b'o' => {
                    i += 1;
                    expect(&mut i, line, b' ')?;
                    if let Ok(name) = std::str::from_utf8(&line[i..]) {
                        visitor.visit_object(name)
                    }
                }
                b'g' => {
                    i += 1;
                    expect(&mut i, line, b' ')?;
                    if let Ok(name) = std::str::from_utf8(&line[i..]) {
                        visitor.visit_group(name)
                    }
                }
                b'#' => {}
                _ => {}
            }

            Ok(())
        }

        loop {
            // refill
            let remainder = self.cap - self.pos;

            if remainder == MAX_LINE_SIZE {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "line too long",
                )));
            }

            if remainder != 0 {
                self.buf.copy_within(self.pos.., 0);
            }

            self.pos = 0;
            self.cap = remainder;

            let read = self.reader.read(&mut self.buf[self.cap..])?;
            self.cap += read;

            for (i, &c) in self.buf[..self.cap].iter().enumerate() {
                let is_newline = (c == b'\n') | (c == b'\r');
                // skip empty lines
                if (i - self.pos) > 1 && is_newline {
                    parse_line(&self.buf[self.pos..i], visitor)?;
                    self.pos = i + 1;
                }
            }

            // eof
            if read == 0 {
                if self.pos != self.cap {
                    parse_line(&self.buf[self.pos..self.cap], visitor)?;
                }
                break;
            }
        }

        Ok(())
    }
}
