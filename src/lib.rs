use std::error::Error;
use std::fs::File;
use std::io::prelude::{Read, Write};
use std::path::Path;
use std::thread::{JoinHandle, scope, spawn};

/*
Notes:
 - Storing frequencies in a vector where indecies 0 - 2^8-1 represent bit patterns for 0 - 2^8-1, then index 2^8 is EoF and 2^8 + 1 is total
 - Each value in the vec is a u32, so stores 0 - 2^32-1. Thus we buffer in up to 2^32-1 bytes to build the vec.
 */
pub fn encode(src: &Path, des: &Path) -> Result<(), Box<dyn Error>> {
    let mut src_file = File::open(src)?;

    let cum_freq = gen_freq_table(&mut src_file)?;

    println!("{:?}", cum_freq);

    Ok(())
}

pub fn decode(src: &Path, des: &Path) -> Result<(), Box<dyn Error>> {
    todo!();
}

fn gen_freq_table(file: &mut File) -> Result<[u32; 258], Box<dyn Error>> {
    const BUFF_SIZE:usize = 1048576; //1 MiB
    const NO_THREADS:usize = 8;

    let mut buff = [0u8; BUFF_SIZE];
    let mut freq = vec![0u32; 257]; //0-255 = bytes 0x00 to 0xFF; 256 = EoF
    let mut total: usize = 0;

    // Add up to 2^32-1 symbols to the frequency table
    let fully_read = loop {
        let bytes_read = file.read(&mut buff)?;
        if bytes_read == 0 {
            break true;
        }
        // check if the total number of bytes read exceeds what can be stored in the table
        if total + bytes_read + 257 > (u32::MAX as usize) {
            break false;
        }

        scope(|s| {
            let handles: Vec<_> = buff[0..bytes_read]
                .chunks(BUFF_SIZE/NO_THREADS)
                .map(|chunk| {
                    s.spawn(move || {
                        let mut result = vec![0u32; 256];
                        chunk.iter().for_each(|byte| result[*byte as usize] += 1);
                        result
                    })
                })
                .collect();

            total += bytes_read;
            
            handles.into_iter().for_each(|h| {
                h.join()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .for_each(|(i, val)| freq[i] += *val)
            });
        });
    };

    // Make sure every symbol has atleast 1 in the frequency table as it could occour later in the file if not fully read. Otherwise just add the EoF marker.
    if !fully_read {
        let _: Vec<()> = freq.iter_mut().map(|f| *f += 1).collect();
        total += 257;
    } else {
        freq[256] = 1;
        total += 1;
    };

    // Convert frequency table to a cumulative frequency table
    let mut rtn = [0u32; 258];
    freq.iter().enumerate().for_each(|(i, _)| {
        rtn[i] = if i == 0 {
            freq[i]
        } else {
            rtn[i - 1] + freq[i]
        }
    });
    rtn[257] = total as u32;

    Ok(rtn)
}
