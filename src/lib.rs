use std::fs::File;
use std::io::prelude::{Read, Seek, Write};
use std::io::{BufReader, BufWriter, Error, ErrorKind, SeekFrom, stdin, stdout};
use std::path::Path;

const VERSION: u8 = 0;
const MSB: u32 = 0b10000000_00000000_00000000_00000000;
const SEC_MSB: u32 = 0b01000000_00000000_00000000_00000000;
const EOF: usize = 256;
const ENC_UPDATE: u64 = 524280;
const DEC_UPDATE: u64 = ENC_UPDATE / 4;

/// Encodes the file in `src` and outputs the resultant file in `des`, overwriting any files that exist.
/// Occasionally prints to the consol progress updates.
pub fn encode(src: &Path, des: &Path) -> Result<(), Error> {
    // Setup
    let mut src_file = File::open(src)?;
    let in_size = src_file.metadata()?.len();
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

    // Check destination
    if des.is_file() && check_des_overwritable() {
        println!("Exiting...");
        return Ok(());
    }

    let mut out_stream: OBitStream = OBitStream::new(gen_file(des, &freq)?);

    println!("Compressing File...");

    // Encoding loop
    loop {
        // Read in 1 byte of data
        if in_stream.read(&mut current_byte)? == 0 {
            symbol = EOF;
        } else {
            symbol = current_byte[0] as usize;
        }

        // Provide a progress update
        bytes_processed += 1;
        if bytes_processed % ENC_UPDATE == 0 {
            output_progress(in_size, bytes_processed)?;
        }

        // Update the range, upper bound and lower bound
        range = high - low;
        high =
            ((low as u64) + ((range as u64) + 1) * cum_freq[symbol + 1] / cum_freq[257] - 1) as u32;
        low = ((low as u64) + ((range as u64) + 1) * cum_freq[symbol] / cum_freq[257]) as u32;

        // Process the new upper and lower bounds
        loop {
            // If MSB of low and high match, push the MSB then handle any overflow
            if (low ^ high) & MSB == 0 {
                out_stream.push(low & MSB != 0)?;
                for _ in 0..underflow_count {
                    out_stream.push(low & MSB == 0)?;
                }
                underflow_count = 0;
                low <<= 1;
                high = (high << 1) + 1;

            // If there is the start of underflow splice out the underflowing bits and incrument the underflow counter
            } else if SEC_MSB & low != 0 && SEC_MSB & high == 0 {
                low = (MSB & low) + ((low & (!SEC_MSB)) << 1);
                high = (MSB & high) + ((high & (!SEC_MSB)) << 1) + 1;
                underflow_count += 1;
            } else {
                break;
            }
        }

        // Output up to the end of the current byte if the end of the input file is reached
        if symbol == EOF {
            out_stream.finish()?;
            break;
        }
    }

    // Output that its finished
    print!("\r\x1b[2K");
    print!("Done!\n");
    drop(out_stream);
    drop(in_stream);
    let out_size = File::open(des)?.metadata()?.len();
    if out_size > in_size {
        println!(
            "Warning: Inflated file by {:.2} kb ({:.2}% larger). The source file may be too small, a non-byte encoding, or too close to entropy.",
            (out_size - in_size) as f64 / 1024 as f64,
            (out_size as f64 / in_size as f64 - 1.0) * 100.0
        );
    } else {
        println!(
            "Deflated file by {:.2} kb ({:.2}% smaller).",
            (in_size - out_size) as f64 / 1024 as f64,
            (1.0 - out_size as f64 / in_size as f64) * 100.0
        );
    }
    Ok(())
}

/// Decodes the arithmetic coded file in `src` and outputs the result in `des` overwriting any files already existing there.
/// Ocasionally prints progress updates to the console.
pub fn decode(src: &Path, des: &Path) -> Result<(), Error> {
    // Setup
    let mut src_file = File::open(src)?;
    let cum_freq = gen_cum_freq(&process_header(&mut src_file)?);
    let size = src_file.metadata()?.len();
    let mut low: u32 = 0;
    let mut high: u32 = u32::MAX; // High = true upper bound - 1
    let mut range: u32; // Range = true range - 1
    let mut encoded: u32 = 0; //Sliding 32 bit window into the encoded file 
    let mut scaled_symbol: u32; //The cumulative frequency for the current symbol 
    let mut symbol: usize;
    let mut bytes_processed: u64 = 0;
    let mut bits_processed: u8 = 0;
    let mut in_stream = IBitStream::new(BufReader::new(src_file));

    // Check destination
    if des.is_file() && check_des_overwritable() {
        println!("Exiting...");
        return Ok(());
    }

    let mut out_stream = BufWriter::new(File::create(des)?);

    println!("Decompressing File...");

    // read in first 32 bits
    for _ in 0..32 {
        encoded = (encoded << 1) + (in_stream.pop()? as u32);
    }

    // Decoding loop
    loop {
        // Determine which symbol encoded represents
        range = high - low;
        scaled_symbol =
            ((cum_freq[257] * (encoded as u64 - low as u64 + 1) - 1) / (range as u64 + 1)) as u32;
        symbol = get_symbol(scaled_symbol, &cum_freq);

        // If its EoF stop
        if symbol == EOF {
            break;
        };

        // Otherwise output it and update the new upper and lower bounds
        out_stream.write(&[symbol as u8])?;
        high =
            ((low as u64) + ((range as u64) + 1) * cum_freq[symbol + 1] / cum_freq[257] - 1) as u32;
        low = ((low as u64) + ((range as u64) + 1) * cum_freq[symbol] / cum_freq[257]) as u32;

        // Provide a progress update
        if bytes_processed % DEC_UPDATE == 0 {
            output_progress(size, bytes_processed)?;
        }

        // Process the new upper and lower bounds
        loop {
            // Shift out MSB if its common between low and high
            if (low ^ high) & MSB == 0 {
                low <<= 1;
                high = (high << 1) + 1;
                encoded = (encoded << 1) + in_stream.pop()? as u32;
                (bits_processed, bytes_processed) =
                    increment_processing(bits_processed, bytes_processed);

            // Splice out overflow
            } else if SEC_MSB & low != 0 && SEC_MSB & high == 0 {
                low = (MSB & low) + ((low & (!SEC_MSB)) << 1);
                high = (MSB & high) + ((high & (!SEC_MSB)) << 1) + 1;
                encoded = (MSB & encoded) + ((encoded & (!SEC_MSB)) << 1) + in_stream.pop()? as u32;
                (bits_processed, bytes_processed) =
                    increment_processing(bits_processed, bytes_processed);
            } else {
                break;
            }
        }
    }

    print!("\r\x1b[2K");
    print!("Done!\n");

    Ok(())
}

pub fn freq_anal(src: &Path) -> Result<(), Error> {
    let freq = process_header(&mut File::open(src)?)?;
    let max = freq.iter().fold(0u8, |ac, c| ac.max(*c)) as usize;
    for (i, f) in freq.iter().enumerate() {
        if *f == 0 {
            continue;
        } else if i == 256 {
            print!(
                "EoF   ({:0>3}) | {:#<2$}\n",
                *f,
                "",
                (*f as usize * 100 / max)
            );
        } else {
            print!(
                "0x{:<3x} ({:0>3}) | {:#<3$}\n",
                i,
                *f,
                "",
                (*f as usize * 100 / max)
            );
        }
    }
    Ok(())
}

/// A bitstream which is able to write individual bits to an output
struct OBitStream {
    bits: u8,
    no_bits: u8,
    writer: BufWriter<File>,
}

impl OBitStream {
    /// Constructs a new bitstream to the file containied within the BufWriter
    fn new(writer: BufWriter<File>) -> Self {
        OBitStream {
            bits: 0,
            no_bits: 0,
            writer: writer,
        }
    }

    /// Pushes a single bit into the stream
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

    /// Pushes 01 into the bitstream, then fills the rest of the byte with 1s
    fn finish(&mut self) -> Result<(), Error> {
        self.push(false)?;
        self.push(true)?;
        while self.no_bits != 0 {
            self.push(true)?;
        }
        Ok(())
    }
}

/// Input bitstream that can read one bit at a time
struct IBitStream {
    reader: BufReader<File>,
    current_byte: [u8; 1],
    bits_outputted: u8,
}

impl IBitStream {
    /// Construct a new IBitStream that reads from the file within the BufReader
    fn new(reader: BufReader<File>) -> Self {
        IBitStream {
            reader: reader,
            current_byte: [0],
            bits_outputted: 8,
        }
    }

    /// Pop one bit from the bitstream and seek along to the next bit
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

/// Reads in `file` and outputs a frequency table generated from the beginning of the file. Seeks the reader back to the start of the file upon completion.
/// Indecies 0-255 represent symbols for the bytes 0x00 to 0xFF. Index 256 represents the end of file symbol.
fn gen_freq_table(reader: &mut BufReader<&mut File>) -> Result<[u8; 257], Error> {
    let mut freq = [0u32; 257]; //0-255 = bytes 0x00 to 0xFF; 256 = EoF
    let mut buff = [0u8; 1];
    let mut symbol: usize;
    let mut max_reached = false;

    // Add symbols to frequency table
    let early_exit = loop {
        let bytes_read = reader.read(&mut buff)?;
        // Stop if enough bytes have been read to either fill up one entry or the total
        if max_reached {
            break true;
        }

        // Stop if the end of file is reached
        if bytes_read == 0 {
            break false;
        }

        // Add the symbol to the frequency table
        symbol = buff[0] as usize;

        if freq[symbol] == u32::MAX {
            max_reached = true;
        } else {
            freq[symbol] += 1;
        }
    };

    let max: u64 = freq.iter().fold(0, |c: u32, a: &u32| c.max(*a)) as u64;

    let mut freq = freq.map(|f| {
        if f == 0 {
            return 0;
        };
        let rtn = (f as u64 * u8::MAX as u64 / max) as u8;
        
        if rtn == 0 {
            1
        } else {
            rtn
        }
    }); // scale frequencies

    // Ensure every frequency is at least 1 if only part of the file was used to produce the table
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

/// Read the header of a compressed file, verifies it follows the correct protocol and returns the frequency table stored in it
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

/// Converts a frequency table to a cumulative frequency table where the index is the lower bound for the symbol.
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

/// Performs linear search to find the frequency band `scaled_symbol` is within in `cum_freq`
fn get_symbol(scaled_symbol: u32, cum_freq: &[u64; 258]) -> usize {
    let s = scaled_symbol as u64;
    for (i, v) in cum_freq.iter().enumerate() {
        if *v <= s && cum_freq[i + 1] > s {
            return i;
        }
    }
    panic!()
}

/// Increments the bits and bytes processed by 1 bit
fn increment_processing(bits_processed: u8, bytes_processed: u64) -> (u8, u64) {
    if bits_processed + 1 == 8 {
        (0, bytes_processed + 1)
    } else {
        (bits_processed + 1, bytes_processed)
    }
}

/// Asks the user if they want to overwrite the destination file, if yes returns false else true
fn check_des_overwritable() -> bool {
    println!("Destination file already exists. Would you like to overwrite it? (y/n): ");
    let mut answer: String = String::new();
    stdin()
        .read_line(&mut answer)
        .expect("Could not read input");
    let mut ans_low: String = answer.trim().to_lowercase();

    while ans_low != "y" && ans_low != "yes" {
        if ans_low == "n" || ans_low == "no" {
            return true;
        }

        println!("Please either enter 'y' or 'n':");
        answer.clear();

        stdin()
            .read_line(&mut answer)
            .expect("Could not read input");
        ans_low = answer.trim().to_lowercase();
    }

    false
}
