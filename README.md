# srt2txt

Convert SRT subtitle files into clean plain text.

## Features
- Removes numbering & timestamps
- Strips simple HTML tags (<i>, <b>, etc.)
- Joins multi-line captions into single lines
- Optionally joins multiple files into one output
- Parallel processing of many files
- Optional duplicate removal and blank line collapsing
- Optionally join all captions into continuous sentences (`--join-sentences`)

## Install
```bash
cargo install --path .
```

## Usage
```bash
srt2txt input.srt
srt2txt subtitles/ --output-dir cleaned/
srt2txt a.srt b.srt --join --join-name all.txt
srt2txt movie.srt --stdout
srt2txt movie.srt --join-sentences --stdout
```

Flags:
```
--stdout              Print combined output of all inputs to stdout
--join                Join all processed captions into a single output file
--join-name <NAME>    Output file name when using --join (default: combined.txt)
--join-sentences      Output a single continuous text block (no blank lines)
--collapse-blank      Collapse multiple blank lines into one
--remove-duplicates   Remove immediately repeated lines
-f, --force           Overwrite existing output files
-o, --output-dir DIR  Output directory (created if missing)
```

## License
MIT
