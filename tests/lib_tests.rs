use ecm3428::*;

// creating a new module so that we can access private 'lib.rs' functions for each test
mod lib {
    include!("../src/lib.rs");

    use std::fs;

    // tests if incrementing bits works properly for different edge-cases
    #[test]
    fn test_increment_processing() {
        assert_eq!(increment_processing(0, 0), (1, 0)); // bit increment starting from 0
        assert_eq!(increment_processing(3, 5), (4, 5)); // regular bit increment only increments num of bits by 1
        assert_eq!(increment_processing(7, 0), (0, 1)); // bit increment from 7 to 8 increases the num of bytes and resets bit count
    }

    // tests to see if the correct cumulative frequency is generated for a given input
    #[test]
    fn test_gen_cum_freq() {
        // random frequencies are chosen and we check if the cum freq is generated correctly
        let mut freq = [0u8; 257];
        freq[0] = 2;
        freq[1] = 3;
        freq[2] = 1;
        let result = gen_cum_freq(&freq);
        assert_eq!(result[0..4], [0, 2, 5, 6]);

        // all frequencies are empty, so we check if cum freq stays zero
        let freq = [0u8; 257];
        let result = gen_cum_freq(&freq);
        assert_eq!(result, [0u64; 258]);

        // all frequencies are the 8bit int limit, so we check if the cum freq of the final element is the correct sum
        let freq = [u8::MAX; 257];
        let result = gen_cum_freq(&freq);
        assert_eq!(result[257], 257 * 255);
    }

    #[test]
    fn test_get_symbol() {
        let mut freq = [0u8; 257];
        freq[0] = 2;
        freq[1] = 3;
        freq[2] = 1;
        let cum_freq = gen_cum_freq(&freq);

        assert_eq!(get_symbol(3, &cum_freq), 1); // checks for a value within a symbols range
        assert_eq!(get_symbol(2, &cum_freq), 1); // checks for a value at the very edge of a symbols range
        assert_eq!(get_symbol(5, &cum_freq), 2); // checks for the symbol of the highest valid value
    }

    // tests if the correct frequency table is generated for a given text file
    #[test]
    fn test_gen_freq_table() {
        // create a temporary file for testing
        let path = std::env::temp_dir().join("test_gen_freq_table_test.txt");

        // An empty text file should have equal frequencies
        fs::write(&path, []).unwrap();
        let mut file = File::open(&path).unwrap();
        let mut reader = BufReader::new(&mut file);
        let freq = gen_freq_table(&mut reader).unwrap();
        assert_eq!(freq[256], 1);

        // a file with one repeated byte should have a higher frequency for that byte
        fs::write(&path, [5, 5, 5]).unwrap();
        let mut file = File::open(&path).unwrap();
        let mut reader = BufReader::new(&mut file);
        let freq = gen_freq_table(&mut reader).unwrap();
        assert_eq!(freq[5], 255);

        // when a byte appears very infrequently, it's frequency must be at least 1 (not zero)
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push(0u8);
        }
        data.push(1);
        fs::write(&path, data).unwrap();
        let mut file = File::open(&path).unwrap();
        let mut reader = BufReader::new(&mut file);
        let freq = gen_freq_table(&mut reader).unwrap();
        assert_eq!((freq[0], freq[1]), (255, 1));

        fs::remove_file(path).unwrap();
    }

    // tests to see if the comp file is encoded properly
    #[test]
    fn test_gen_file() {
        let path = std::env::temp_dir().join("gen_file_test.bin");

        // we create a frequency table and store values inside of it
        let mut freq = [0u8; 257];
        freq[0] = 5;
        freq[256] = 1;
        let writer = gen_file(&path, &freq).unwrap();
        drop(writer);
        let data = fs::read(&path).unwrap();

        assert_eq!((data[1], data[257]), (5, 1)); // check to see if frequency table matches in the final file
        assert_eq!(data[0], VERSION); // check to see if the first file matches the file version
        assert_eq!(data.len(), 258); // check to see if the total length of the file is what we expected

        fs::remove_file(path).unwrap();
    }

    // tests if the program can correctly identify whether the header is valid
    #[test]
    fn test_process_header() {
        let path = std::env::temp_dir().join("process_header_test.bin");

        // check to see if a correct header can output the right version
        let mut header = [0u8; 258];
        header[1] = 5;
        header[257] = 1;
        fs::write(&path, header).unwrap();
        let mut file = File::open(&path).unwrap();
        assert_eq!(process_header(&mut file).unwrap()[0], 5);

        // checks if the program will throw an error if the length of the header is incorrect
        fs::write(&path, [0u8; 10]).unwrap();
        let mut file = File::open(&path).unwrap();
        assert_eq!(
            process_header(&mut file).unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );

        // checks if the program will throw an error if the version does not match what was expected
        header[0] = VERSION + 1;
        fs::write(&path, header).unwrap();
        let mut file = File::open(&path).unwrap();
        assert_eq!(
            process_header(&mut file).unwrap_err().kind(),
            ErrorKind::Unsupported
        );

        fs::remove_file(path).unwrap();
    }

    // tests whether the 'push' function implementation from OBitStream can push bits and write bytes
    #[test]
    fn test_obitstream_push() {
        let path = std::env::temp_dir().join("obitstream_push_test.bin");

        let file = File::create(&path).unwrap();
        let mut stream = OBitStream::new(BufWriter::new(file));

        // we push 1 bit onto the stream and see if the num of bits increased
        stream.push(true).unwrap();
        assert_eq!(stream.no_bits, 1);

        // we push 7 bits onto the stream and check if the num of bits is 0
        for bit in [true, true, true, true, true, true, true] {
            stream.push(bit).unwrap();
        }
        assert_eq!(stream.no_bits, 0); // the bits should have been turned into a byte and the counter reset

        drop(stream);
        assert_eq!(fs::read(&path).unwrap(), [0b11111111]); // check if the file reads our 8 current bits

        fs::remove_file(path).unwrap();
    }

    // tests our programs implementation of a 'pop' function for inspecting bits
    #[test]
    fn test_ibitstream_pop() {
        let path = std::env::temp_dir().join("ibitstream_pop_test.bin");

        // setup an 8bit binary number with a leading 1 as well as the ibitstream
        fs::write(&path, [0b10000000]).unwrap();
        let file = File::open(&path).unwrap();
        let mut stream = IBitStream::new(BufReader::new(file));

        assert_eq!(stream.pop().unwrap(), true); // first pop should give us the leftmost bit, which is a '1'
        assert_eq!(stream.pop().unwrap(), false); // second pop should give us the bit after '1', which is a '0'

        // iterate through remaining bits
        for i in 0..6 {
            stream.pop().unwrap();
        }
        assert_eq!(stream.pop().unwrap(), true); // any pop past the EoF should always return true or 0

        drop(stream);
        fs::remove_file(path).unwrap();
    }

    // tests whether the full process of encoding and decoding produces an uncompressed file identical to the original
    #[test]
    fn test_encode_decode_convertible() {
        // we create files where the original, encoded and decoded files will be stored
        let src = std::env::temp_dir().join("src_test.txt");
        let comp = std::env::temp_dir().join("comp_test.bin");
        let uncomp = std::env::temp_dir().join("uncomp_test.txt");
        let data = b"Hello world!"; // some normal data written in the file
        fs::write(&src, data).unwrap();
        encode(&src, &comp).unwrap();
        decode(&comp, &uncomp).unwrap();
        assert_eq!(fs::read(&uncomp).unwrap(), data); // check if the uncompressed data matches the original

        // checks if the enc-dec process works for empty files
        fs::write(&src, []).unwrap();
        fs::remove_file(&comp).unwrap();
        fs::remove_file(&uncomp).unwrap();
        encode(&src, &comp).unwrap();
        decode(&comp, &uncomp).unwrap();
        assert_eq!(fs::read(&uncomp).unwrap(), Vec::<u8>::new()); // same as above assert!

        // check if large files are convertible
        let mut data = Vec::new();
        for i in 0..10000 {
            data.push(0u8);
        }
        fs::write(&src, &data).unwrap();
        fs::remove_file(&comp).unwrap();
        fs::remove_file(&uncomp).unwrap();
        encode(&src, &comp).unwrap();
        decode(&comp, &uncomp).unwrap();

        assert_eq!(fs::read(&uncomp).unwrap(), data); // same as above assert!

        fs::remove_file(src).unwrap();
        fs::remove_file(comp).unwrap();
        fs::remove_file(uncomp).unwrap();
    }
}
