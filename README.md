# Algorithmic Group Project

### Arithmetic coding

In this project we intend to make a text compression algorithm using arithmetic coding. In doing so we are making the base algoithm for a universal compression algorithm for use with all file types, however for simplicity we are only working with text files for this project.

Our project is intended to be used by anyone who would like to compress text files.

#### Installation and Running Project:

First download and unzip the project, then with a terminal window open in the project folder 
- To encode a file use `cargo run enc [SRC PATH] [OUT PATH]`
- To decode a file use `cargo run dec [SRC PATH] [OUT PATH]`
- To see the frequency table for an encoded file use `cargo run freq [SRC PATH]`

#### Recompiling Project:

With a terminal window open in the project folder 
- Use `cargo build` to recompile the program

#### Running tests:

With a terminal window open in the project folder 
- To run all tests in the test directory use `cargo test`
- To run all tests in a specific file use `cargo test --test FILENAME`
- To run a specific test in a specific file use `cargo test --test FILENAME TEST_FUNCTION_NAME -- --exact`

### Algorithm pseudocode and explanation
Encoding:
```
low = 0
high = 0
cum_freq = cumulative frequency of each symbol in the alphabet 
for each symbol in input
{
  range = high - low 
  high = low + range * cum_freq(symbol-1) 
  low = low + range * cum_freq(symbol)
}
output(low)
```

- Symbols are assigned a portion of a narrowing interval based on the probability of that symbol (starting as [0, 1))
- Divide interval into the probability of the letters appearing
- Keep the probability of the current symbol and discard the rest
- Do this for each symbol and narrow the interval until you reach the end
- The message is encoded as a number in the final interval

Decoding:
```
while symbol left to decode
{
  symbol = find symbol between range cum_freq[symbol] <= low/high <   cum_freq[symbol-1]
  output(symbol)
  range = high - low
  high = low + range*cum_freq[symbol-1]
  low = low + range*cum_freq[symbol]
}
```

- This algorithm is the reverse of the encoding algorithm
- Symbols are decoded based on the current interval
- The interval is slowly widened as symbols are decoded
- The message has been decoded once the interval has returned to [0, 1)

#### Time And Space Complexity
The time complexity of Arithmetic encoding is linear.

The space complexity of arithmetic encoding is constant.

#### Test Cases And Results
For testing the algorithm we used various unit and integration tests.

Integration tests:
- no argument given
- unknown command
- check help command is case sensitive, correct error for wrong number of arguments, test works when used properly
- check enc, dec, and freq behaves properly with wrong arguments
- test freq with empty path
- file cannot be found
- a directory is given instead of a file
- selected file is read only
- file cannot be written to
- storage is full
- storage quota exceeded
- resources are busy
- filename given is invalid
- file ends unexpectedly
- program cannot get access to enough memory
- version of the file given is more recent than the decoder version being used
- file cannot be accessed due to inadequate permisions
- any other i/o error is raised

Unit tests:
- test that incrementing bits works properly for edge cases
- test that correct cumulative frequency table is generated for given inputs
- test getting a symbol from the cumulative frequency table
- test generating frequency table from a test file
- test to see if file is encoded properly
- test that the program can detect if the header is valid
- test that the push function pushes and writes bytes correctly
- test our implementation of pop works correctly
- test that encoding and decoding process creates a file identical to the original

These test showed that our project worked as expected since all of these tests showed the correct errors or worked as expected for these situations.

### Current Limitations

The current limitations of our compression algorithm is firstly that it is currently only designed for use with text files, and secondly while our compression algorithm does currently decrease the file size for most files it does on ocassion not make the file any smaller or even make it slightly bigger due to storage overhead.

### Project Dependencies

For our project you will need rust installed you can get this from the rust website https://rust-lang.org/tools/install/.
Our project has no dependencies or third-party resources beyond standard Rust libraries and the code we have written.

### Contribution Statement
- Tom - Algorithm implementation, backend, and project lead
- Dawid - Testing Lead and Command Line Interface
- Ben - Documentation and Demonstration Lead
- Will - Presentation Lead
- Max - Presentation results section

### AI Use Declaration
No AI was used in the making of this project
