use std::fs::File;
use std::io::prelude::{Read, Seek, Write};
use std::io::{BufWriter, Error, SeekFrom, stdout};
use std::path::Path;
use std::thread::scope;

const BUFF_SIZE: usize = 524288; //1/2 MiB
const NO_THREADS: usize = 8;
const VERSION: u8 = 0;
const MSB: u32 = 0b10000000_00000000_00000000_00000000;
const SEC_MSB: u32 = 0b01000000_00000000_00000000_00000000;
//const BYTE1: u32 = 0b11111111_00000000_00000000_00000000;
/// most significant byte
//const BYTE2: u32 = 0b00000000_11111111_00000000_00000000;
/// second most significant byte
//const BYTE3: u32 = 0b00000000_00000000_11111111_00000000;
/// third most significant byte
//const BYTE4: u32 = 0b00000000_00000000_00000000_11111111;
/// least significant byte

/*
Notes:
 - Storing frequencies in a vector where indecies 0 - 2^8-1 represent bit patterns for 0 - 2^8-1, then index 2^8 is EoF and 2^8 + 1 is total
 - Each value in the vec is a u32, so stores 0 - 2^32-1. Thus we buffer in up to 2^32-1 bytes to build the vec.
 */
pub fn encode(src: &Path, des: &Path) -> Result<(), Error> {
    let mut src_file = File::open(src)?;

    println!("Generating Frequnecy Table...");
    let freq = gen_freq_table(&mut src_file)?;
    let mut cum_freq: Vec<u64> = freq.iter().fold(Vec::new(), |mut acc: Vec<u64>, cur: &u8| {
        acc.push(match acc.last() {
            None => *cur as u64,
            Some(v) => *cur as u64 + *v,
        });
        acc
    });
    cum_freq.push(cum_freq.last().unwrap() + 1);

    println!("Compressing File...");
    let mut out_stream: BitStream = BitStream::new(gen_file(des, freq)?);

    let mut low: u32 = 0;
    let mut high: u32 = u32::MAX; // High = true upper bound - 1
    let mut range: u32 = u32::MAX; // Range = true range - 1
    let mut underflow_count: u64 = 0;
    let mut buff = [0u8; BUFF_SIZE];

    let size = src_file.metadata()?.len();
    let mut bytes_processed: u64 = 0;

    loop {
        let bytes_read = src_file.read(&mut buff)?;
        if bytes_read == 0 {
            break out_stream.finish();
        }

        bytes_processed += bytes_read as u64;
        output_progress(size, bytes_processed)?;

        buff[0..bytes_read]
            .iter()
            .try_for_each(|byte: &u8| -> Result<(), Error> {
                range = high - low;
                high = ((low as u64)
                    + ((range as u64) + 1) * cum_freq[*byte as usize + 1] / cum_freq[257]
                    - 1) as u32;
                low = ((low as u64)
                    + ((range as u64) + 1) * cum_freq[*byte as usize] / cum_freq[257])
                    as u32;

                loop {
                    // If MSB of low and high match, push the MSB then handle any overflow.
                    if (low ^ high) & MSB == 0 {
                        out_stream.push(low & MSB != 0)?;
                        for _ in 0..underflow_count {
                            out_stream.push(low & MSB == 0)?;
                        }
                        underflow_count = 0;
                        low <<= 1;
                        high <<= 1;
                        high += 1;
                    } else if SEC_MSB & low != 0 && SEC_MSB & high == 0 {
                        low = (MSB & low) + ((low & (!SEC_MSB)) << 1);
                        high = (MSB & high) + ((high & (!SEC_MSB)) << 1) + 1;
                        underflow_count += 1;
                    } else {
                        break Ok(());
                    }
                }
            })?;
    }
}

pub fn decode(src: &Path, des: &Path) -> Result<(), Error> {
    todo!();
}

struct BitStream {
    bits: u8,
    no_bits: u8,
    writer: BufWriter<File>,
}

impl BitStream {
    fn new(writer: BufWriter<File>) -> Self {
        BitStream {
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

/// Reads in `file` and ouputs a cumulative frequency table.
/// Indecies 0-255 represent symbols for the bytes 0x00 to 0xFF. Index 256 represents the end of file symbol. Index 257 represents the total number of symbols in the table.
/// Will parse up to 2^32-1 bytes.
fn gen_freq_table(file: &mut File) -> Result<[u8; 257], Error> {
    let mut buff = [0u8; BUFF_SIZE];
    let mut freq = [1u8; 257]; //0-255 = bytes 0x00 to 0xFF; 256 = EoF
    let mut total: u64 = 257;
    let mut max_reached = false;

    // Add up to 2^32-1 symbols to the frequency table
    loop {
        let bytes_read = file.read(&mut buff)?;
        // check if the total number of bytes read exceeds what can be stored in the table
        if total + bytes_read as u64 > (u32::MAX as u64) || max_reached || bytes_read == 0 {
            break;
        }

        scope(|s| {
            let handles: Vec<_> = buff[0..bytes_read]
                .chunks(BUFF_SIZE / NO_THREADS)
                .map(|chunk| {
                    s.spawn(move || {
                        let mut result = vec![0u8; 256];
                        let mut local_max_reached = false;
                        chunk.iter().for_each(|byte| {
                            if !local_max_reached {
                                if result[*byte as usize] == u8::MAX {
                                    local_max_reached = true;
                                } else {
                                    result[*byte as usize] += 1
                                }
                            }
                        });
                        result
                    })
                })
                .collect();

            total += bytes_read as u64;

            handles.into_iter().for_each(|h| {
                h.join().unwrap().iter().enumerate().for_each(|(i, val)| {
                    if freq[i] as u16 + *val as u16 > u8::MAX as u16 {
                        freq[i] = u8::MAX;
                        max_reached = true;
                    } else {
                        freq[i] += *val
                    }
                })
            });
        });
    }

    file.seek(SeekFrom::Start(0))?;

    Ok(freq)
}

/// Generates the destination file and writes the header to it.
fn gen_file(file: &Path, freq: [u8; 257]) -> Result<BufWriter<File>, Error> {
    let mut writer = BufWriter::new(File::create(file)?);
    writer.write(&[VERSION])?;
    writer.write(&freq)?;

    Ok(writer)
}

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

    print!("\r");
    print!(
        "[{} {:.2} {}]  {} / {}Kb",
        left, perc, right, bytes_processed_kb, size_kb
    );
    stdout().flush()?;
    Ok(())
}
