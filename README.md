# Algorithmic Group Project

### Arithmetic coding

In this project we intend to make a text compression algorithm using arithmetic coding. In doing so we are hoping to make the base algoithm for a universal compression algorithm for use with all file types however for simplicity we are only working with text files for this project.

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

### Algorithm pseudocode
Encoding:
```
low = 0
high = 0
for each symbol in input
{
  range = high - low
  high = low + range * high_range(symbol)
 low = low + range * low_range(symbol)
}
output(low)
```

Decoding:
```
number = input_code()
while (symbol != EOF)
{
  symbol = find_symbol_within_this_range(number)
  output(symbol)
  range = high_range(symbol) - low_range(symbol)
  number = number - low_range(symbol)
  number = number / range
}
```

#### Time And Space Complexity
The time complexity of Arithmetic encoding is linear.
The space complexity of arithmetic encoding is constant

#### Test Cases And Results
(a concise explanation or pseudocode of the algorithm and its time and space complexity;
– the test cases used and a summary of the results)

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
