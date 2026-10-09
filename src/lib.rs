use std::fs::File;
use std::io::prelude::{Read, Seek, Write};
use std::io::{BufReader, BufWriter, Error, ErrorKind, SeekFrom, stdout};
use std::path::Path;

const VERSION: u8 = 0;
const MSB: u32 = 0b10000000_00000000_00000000_00000000;
const SEC_MSB: u32 = 0b01000000_00000000_00000000_00000000;
const EOF: usize = 256;

pub fn encode(src: &Path, des: &Path) -> Result<(), Error> {
    println!("Compressing File...");

    // Setup
    let mut src_file = File::open(src)?;
    let size = src_file.metadata()?.len();
    let mut in_stream = BufReader::new(&mut src_file);
    let freq = gen_freq_table(&mut in_stream)?;
    let cum_freq = gen_cum_freq(&freq);
    let mut low: u32 = 0;
    let mut high: u32 = u32::MAX; // High = true upper bound - 1
    let mut range: u32; // Range = true range - 1
    let mut underflow_count: u64 = 0;
    let mut current_byte = [0u8; 1];
    let mut symbol: usize;
    let mut bytes_processed: u64 = 0;
    let mut out_stream: OBitStream = OBitStream::new(gen_file(des, &freq)?);

    loop {
        if in_stream.read(&mut current_byte)? == 0 {
            symbol = EOF;
        } else {
            symbol = current_byte[0] as usize;
        }

        bytes_processed += 1;

        if bytes_processed % u16::MAX as u64 == 0 {
            output_progress(size, bytes_processed)?;
        }

        range = high - low;
        high =
            ((low as u64) + ((range as u64) + 1) * cum_freq[symbol + 1] / cum_freq[257] - 1) as u32;
        low = ((low as u64) + ((range as u64) + 1) * cum_freq[symbol] / cum_freq[257]) as u32;

        loop {
            // If MSB of low and high match, push the MSB then handle any overflow.
            if (low ^ high) & MSB == 0 {
                out_stream.push(low & MSB != 0)?;
                for _ in 0..underflow_count {
                    out_stream.push(low & MSB == 0)?;
                }
                underflow_count = 0;
                low <<= 1;
                high = (high << 1) + 1;
            } else if SEC_MSB & low != 0 && SEC_MSB & high == 0 {
                low = (MSB & low) + ((low & (!SEC_MSB)) << 1);
                high = (MSB & high) + ((high & (!SEC_MSB)) << 1) + 1;
                underflow_count += 1;
            } else {
                break;
            }
        }

        if symbol == EOF {
            out_stream.finish()?;
            break;
        }
    }

    // Output that its finished
    print!("\r\x1b[2K");
    print!("Done!\n");

    Ok(())
}

pub fn decode(src: &Path, des: &Path) -> Result<(), Error> {
    println!("Decompressing File...");

    // Setup
    let mut src_file = File::open(src)?;
    let cum_freq = gen_cum_freq(&process_header(&mut src_file)?);
    let size = src_file.metadata()?.len();
    let mut low: u32 = 0;
    let mut high: u32 = u32::MAX; // High = true upper bound - 1
    let mut range: u32; // Range = true range - 1
    let mut encoded: u32 = 0;
    let mut scaled_symbol: u32;
    let mut symbol: usize;
    let mut bytes_processed: u64 = 0;
    let mut bits_processed: u8 = 0;
    let mut out_stream = BufWriter::new(File::create(des)?);
    let mut in_stream = IBitStream::new(BufReader::new(src_file));

    // read in first 32 bits
    for _ in 0..32 {
        encoded = (encoded << 1) + (in_stream.pop()? as u32);
    }

    // decoding loop
    loop {
        range = high - low;
        scaled_symbol =
            ((cum_freq[257] * (encoded as u64 - low as u64 + 1) - 1) / (range as u64 + 1)) as u32;
        symbol = get_symbol(scaled_symbol, &cum_freq);

        if symbol == EOF {
            break;
        };

        out_stream.write(&[symbol as u8])?;
        high =
            ((low as u64) + ((range as u64) + 1) * cum_freq[symbol + 1] / cum_freq[257] - 1) as u32;
        low = ((low as u64) + ((range as u64) + 1) * cum_freq[symbol] / cum_freq[257]) as u32;

        if bytes_processed % u16::MAX as u64 == 0 {
            output_progress(size, bytes_processed)?;
        }

        loop {
            if (low ^ high) & MSB == 0 {
                low <<= 1;
                high = (high << 1) + 1;
                encoded = (encoded << 1) + in_stream.pop()? as u32;
                (bits_processed, bytes_processed) =
                    incrument_processing(bits_processed, bytes_processed);
            } else if SEC_MSB & low != 0 && SEC_MSB & high == 0 {
                low = (MSB & low) + ((low & (!SEC_MSB)) << 1);
                high = (MSB & high) + ((high & (!SEC_MSB)) << 1) + 1;
                encoded = (MSB & encoded) + ((encoded & (!SEC_MSB)) << 1) + in_stream.pop()? as u32;
                (bits_processed, bytes_processed) =
                    incrument_processing(bits_processed, bytes_processed);
            } else {
                break;
            }
        }
    }

    print!("\r\x1b[2K");
    print!("Done!\n");

    Ok(())
}

/// A bitstream, which is able to write individual btis to an output.
struct OBitStream {
    bits: u8,
    no_bits: u8,
    writer: BufWriter<File>,
}

impl OBitStream {
    fn new(writer: BufWriter<File>) -> Self {
        OBitStream {
            bits: 0,
            no_bits: 0,
            writer: writer,
        }
    }

    fn push(&mut self, bit: bool) -> Result<(), Error> {
        self.bits <<= 1;
        self.bits += bit as u8;
        self.no_bits += 1;

        if self.no_bits == 8 {
            self.writer.write(&[self.bits])?;
            self.bits = 0;
            self.no_bits = 0;
        }

        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.push(false)?;
        self.push(true)?;
        while self.no_bits != 0 {
            self.push(true)?;
        }
        Ok(())
    }
}

struct IBitStream {
    reader: BufReader<File>,
    current_byte: [u8; 1],
    bits_outputted: u8,
}

impl IBitStream {
    fn new(reader: BufReader<File>) -> Self {
        IBitStream {
            reader: reader,
            current_byte: [0],
            bits_outputted: 8,
        }
    }

    fn pop(&mut self) -> Result<bool, Error> {
        if self.bits_outputted == 8 {
            let bytes_read = self.reader.read(&mut self.current_byte)?;
            self.bits_outputted = 0;
            // assume infinite number of 1s at EoF
            if bytes_read != 1 {
                self.current_byte[0] = u8::MAX;
            };
        };

        self.bits_outputted += 1;
        let rtn = (self.current_byte[0] & 0b10000000) != 0;
        self.current_byte[0] <<= 1;
        Ok(rtn)
    }
}

/// Reads in `file` and ouputs a cumulative frequency table.
/// Indecies 0-255 represent symbols for the bytes 0x00 to 0xFF. Index 256 represents the end of file symbol.
fn gen_freq_table(reader: &mut BufReader<&mut File>) -> Result<[u8; 257], Error> {
    let mut freq = [0u8; 257]; //0-255 = bytes 0x00 to 0xFF; 256 = EoF
    let mut buff = [0u8; 1];
    let mut symbol: usize;
    let mut total: u64 = 0;
    let mut max_reached = false;

    // Add symbols to frequency table
    let early_exit = loop {
        let bytes_read = reader.read(&mut buff)?;
        // check if enough bytes have been read to either fill up the table or each maximum value of total
        if max_reached {
            break true;
        }

        if bytes_read == 0 {
            break false;
        }

        symbol = buff[0] as usize;

        if freq[symbol] == u8::MAX || total + 1 > u32::MAX as u64 {
            max_reached = true;
        } else {
            freq[symbol] += 1;
            total += 1;
        }
    };

    if early_exit {
        freq = freq.map(|f| if f == 0 { 1 } else { f });
    } else {
        freq[256] = 1; // EoF
    }

    reader.seek(SeekFrom::Start(0))?;

    Ok(freq)
}

/// Generates the destination file and writes the header to it.
fn gen_file(file: &Path, freq: &[u8; 257]) -> Result<BufWriter<File>, Error> {
    let mut writer = BufWriter::new(File::create(file)?);
    writer.write(&[VERSION])?;
    writer.write(freq)?;

    Ok(writer)
}

/// Prints out a progress bar to the command line
fn output_progress(size: u64, bytes_processed: u64) -> Result<(), Error> {
    let perc = (bytes_processed as f64 / size as f64) * 100.0;
    let size_kb = size / 1000;
    let bytes_processed_kb = bytes_processed / 1000;
    let left = if perc < 5.0 {
        "        "
    } else if perc < 10.0 {
        "#       "
    } else if perc < 15.0 {
        "##      "
    } else if perc < 20.0 {
        "###     "
    } else if perc < 25.0 {
        "####    "
    } else if perc < 30.0 {
        "#####   "
    } else if perc < 35.0 {
        "######  "
    } else if perc < 40.0 {
        "####### "
    } else {
        "########"
    };
    let right = if perc < 60.0 {
        "        "
    } else if perc < 65.0 {
        "#       "
    } else if perc < 70.0 {
        "##      "
    } else if perc < 75.0 {
        "###     "
    } else if perc < 80.0 {
        "####    "
    } else if perc < 85.0 {
        "#####   "
    } else if perc < 90.0 {
        "######  "
    } else if perc < 95.0 {
        "####### "
    } else if perc < 100.0 {
        "####### "
    } else {
        "########"
    };

    print!("\r\x1b[2K");
    print!(
        "[{} {:.2}% {}]  {} / {}Kb",
        left, perc, right, bytes_processed_kb, size_kb
    );
    stdout().flush()?;
    Ok(())
}

/// Read the header of a compressed file, and returns the frequency table
fn process_header(file: &mut File) -> Result<[u8; 257], Error> {
    // Parse version number
    let mut header = [0u8; 258];
    let bytes_read = file.read(&mut header)?;
    if bytes_read != 258 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "Expected 258 byte header but reached EoF after {} bytes",
                bytes_read
            ),
        ));
    } else if header[0] > VERSION {
        return Err(Error::new(
            ErrorKind::Unsupported,
            format!("File version {} unsupported", header[0]),
        ));
    };

    let mut rtn = [0u8; 257];
    rtn.copy_from_slice(&header[1..=257]);
    Ok(rtn)
}

/// Converts a frequency table to a cumulative frequency table where the index i is the lower bound for the symbol.
/// Indecies 0-255 represent bytes 0x00 to 0xFF. Index 256 represents EoF. Index 257 is the total.
fn gen_cum_freq(freq: &[u8; 257]) -> [u64; 258] {
    let mut cum_freq = [0u32; 258];

    for (i, f) in freq.iter().enumerate() {
        cum_freq[i + 1] = if i != 0 {
            cum_freq[i] + *f as u32
        } else {
            *f as u32
        };
    }

    cum_freq.map(|f| f as u64)
}

fn get_symbol(scaled_symbol: u32, cum_freq: &[u64; 258]) -> usize {
    let s = scaled_symbol as u64;
    for (i, v) in cum_freq.iter().enumerate() {
        if *v <= s && cum_freq[i + 1] > s {
            return i;
        }
    }
    panic!()
}

fn incrument_processing(bits_processed: u8, bytes_processed: u64) -> (u8, u64) {
    if bits_processed + 1 == 8 {
        (0, bytes_processed + 1)
    } else {
        (bits_processed + 1, bytes_processed)
    }
}
