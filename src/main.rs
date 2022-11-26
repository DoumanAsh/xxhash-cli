#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

use arg::Args;

use std::io::{self, Read};
use std::fs::File;

pub struct ChunkedReader<T, const N: usize> {
    buffer: [u8; N],
    io: T,
}

impl<T: Read, const N: usize> ChunkedReader<T, N> {
    pub fn new(io: T) -> Self {
        debug_assert_ne!(N, 0);

        Self {
            buffer: [0u8; N],
            io
        }
    }

    ///Gets next chunk, if any.
    pub fn next(&mut self) -> io::Result<Option<&[u8]>> {
        let mut total_size = 0usize;
        let mut buf = self.buffer.as_mut_slice();
        loop {
            match self.io.read(buf) {
                Ok(0) => break,
                Ok(size) => {
                    total_size = total_size.saturating_add(size);
                    buf = &mut buf[size..];
                },
                Err(error) => match error.kind() {
                    io::ErrorKind::Interrupted => continue,
                    _ => return Err(error)
                },
            }
        }

        if total_size == 0 {
            Ok(None)
        } else {
            Ok(Some(&mut self.buffer[..total_size]))
        }
    }
}


#[derive(Debug)]
enum HashKind {
    Xxh3,
    Xxh3_64,
    Xxh64,
    Xxh32,
}

impl core::str::FromStr for HashKind {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.eq_ignore_ascii_case("xxh3") {
            Ok(Self::Xxh3)
        } else if text.eq_ignore_ascii_case("xxh3_64") {
            Ok(Self::Xxh3_64)
        } else if text.eq_ignore_ascii_case("xxh32") {
            Ok(Self::Xxh32)
        } else if text.eq_ignore_ascii_case("xxh64") {
            Ok(Self::Xxh64)
        } else {
            Err(())
        }
    }
}

#[derive(Args, Debug)]
///xxhash
///Hashsum utility
struct Cli {
    #[arg(short = "s", long = "seed", default_value = "0")]
    ///Seed for hash to use. Defaults to 0.
    pub seed: u64,
    #[arg(long, default_value = "false")]
    ///Specifies to generate hash as UUID v4 for xxh3 128bit variant.
    pub uuid: bool,
    #[arg(required)]
    ///Hash algorithm to use
    pub kind: HashKind,
    ///File to hash
    pub file: Vec<String>,
}

fn open_file(path: &str) -> io::Result<ChunkedReader<File, 4096>> {
    Ok(ChunkedReader::new(File::open(path)?))
}

fn main() {
    let args = arg::parse_args::<Cli>();

    if args.file.is_empty() {
        println!("No file specified...");
        return;
    }

    match args.kind {
        HashKind::Xxh3 => {
            let mut hasher = xxhash_rust::xxh3::Xxh3::with_seed(args.seed);
            for file in args.file.iter() {
                let mut reader = match open_file(file) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("{}: cannot open: {}", file, error);
                        return;
                    }
                };

                loop {
                    match reader.next() {
                        Ok(None) => break,
                        Ok(Some(chunk)) => hasher.update(chunk),
                        Err(error) => {
                            eprintln!("{}: error reading: {}", file, error);
                            return;
                        }
                    }
                }

                let hash = hasher.digest128();
                if args.uuid {
                    let uuid = lolid::Uuid::from_bytes(hash.to_le_bytes()).set_variant().set_version(lolid::Version::Random);
                    println!("{file}:{uuid}");
                } else {
                    println!("{file}:{hash}");
                }
                hasher.reset();
            }
        },
        HashKind::Xxh3_64 => {
            let mut hasher = xxhash_rust::xxh3::Xxh3::with_seed(args.seed);
            for file in args.file.iter() {
                let mut reader = match open_file(file) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("{}: cannot open: {}", file, error);
                        return;
                    }
                };

                loop {
                    match reader.next() {
                        Ok(None) => break,
                        Ok(Some(chunk)) => hasher.update(chunk),
                        Err(error) => {
                            eprintln!("{}: error reading: {}", file, error);
                            return;
                        }
                    }
                }

                let hash = hasher.digest();
                println!("{file}:{hash}");
                hasher.reset();
            }
        },
        HashKind::Xxh64 => {
            let mut hasher = xxhash_rust::xxh64::Xxh64::new(args.seed);
            for file in args.file.iter() {
                let mut reader = match open_file(file) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("{}: cannot open: {}", file, error);
                        return;
                    }
                };

                loop {
                    match reader.next() {
                        Ok(None) => break,
                        Ok(Some(chunk)) => hasher.update(chunk),
                        Err(error) => {
                            eprintln!("{}: error reading: {}", file, error);
                            return;
                        }
                    }
                }

                let hash = hasher.digest();
                println!("{file}:{hash}");
                hasher.reset(args.seed);
            }
        },
        HashKind::Xxh32 => {
            let seed: u32 = match args.seed.try_into() {
                Ok(seed) => seed,
                Err(_) => {
                    eprint!("{} is not valid seed for 32bit hash", args.seed);
                    return;
                }
            };
            let mut hasher = xxhash_rust::xxh32::Xxh32::new(seed);
            for file in args.file.iter() {
                let mut reader = match open_file(file) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("{}: cannot open: {}", file, error);
                        return;
                    }
                };

                loop {
                    match reader.next() {
                        Ok(None) => break,
                        Ok(Some(chunk)) => hasher.update(chunk),
                        Err(error) => {
                            eprintln!("{}: error reading: {}", file, error);
                            return;
                        }
                    }
                }

                let hash = hasher.digest();
                println!("{file}:{hash}");
                hasher.reset(seed);
            }
        }
    }
}
