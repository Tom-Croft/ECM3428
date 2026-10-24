# Algorithmic Group Project

### Arithmetic coding

In this project we intend to make a text compression algorithm using arithmetic coding. In doing so we are hoping to make the base algoithm for a universal compression algorithm for use with all file types however for simplicity we are only working with text files for this project.

Our project is intended to be used by anyone who would like to compress text files.

The compiled exe file will be able to run as a standalone file for a linux operating system. 
You can also compile it for use with another system using cargo along with the project files.

#### Compiling Project:

With a terminal window open in the project folder 
- Use `cargo build` to compile the program
- Use `cargo run` to run the program

#### Running tests:

With a terminal window open in the project folder 
- To run all tests in the test directory use `cargo test`
- To run all tests in a specific file use `cargo test --test FILENAME`
- To run a specific test in a specific file use `cargo test --test FILENAME TEST_FUNCTION_NAME -- --exact`

### Algorithm Explanation
Symbols are 

encoding:
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

decoding:
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


The time complexity of Arithmetic encoding is linear.


(a concise explanation or pseudocode of the algorithm and its time and space complexity;
– the test cases used and a summary of the results)

### Current Limitations

The current limitations of our compression algorithm is firstly that it is currently only designed for use with text files, and secondly while our compression algorithm does currently decrease the file size for most files it does on ocassion not make the file any smaller or even make it slightly bigger due to storage overhead.

### Project dependencies

Our project has no dependencies or third-party resources beyond standard Rust libraries and the code we have written.

### Contribution statement
Tom - Algorithm implementation, backend, and project lead
Dawid - Testing Lead and Command Line Interface
Ben - Documentation and Demonstration Lead
Will - Presentation Lead
Max - 

### AI use declaration
(the required acknowledgement of generative AI use)
