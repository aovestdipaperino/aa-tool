# el-tool

el-tool is a command line tool written in Rust to help converting/playing netlist nltool output logs.
It allows to:
- plot multiple log files into a PNG chart
- convert the log into a C++ array representing raw audio samples
- generate a wave file out of an input log
- play the log file without having to generate the wav

**Testing it:**

```
cargo build --release
./target/release/el-tool play --start=7 test/music.log
```
**Usage:**

```
el-tool <COMMAND>

Commands:
  plot       Generates a plot
  wav        Generates a wav file
  cpp-array  Generates a C++ array
  play       Plays the sound
```

Generating a plot:

```
Usage: el-tool plot [OPTIONS] [FILENAMES]...

Arguments:
  [FILENAMES]...  The input file to use

Options:
      --zoom <zoom>      The zoom factor
  -o, --output <OUTPUT>  The output file to use
  -s, --start <START>    Start time
```

Generating a wav file:

```
Usage: el-tool wav [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>  The input file to use

Options:
  -o, --output <OUTPUT>  The output file to use
  -s, --start <START>    Start time
```

Playing a sound

```
Usage: el-tool play [OPTIONS] [FILENAMES]...

Arguments:
  [FILENAMES]...  The input file to use

Options:
  -s, --start <START>  Start time
```
