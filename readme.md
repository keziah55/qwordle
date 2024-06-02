# QWordle

A command line QWordle implementation, in Rust.

## Build

`cargo build --release` builds the executable `target/release/qwordle`

## How to play

QWordle is like Wordle. However, there are two possible correct answers.
You only have to find one of the words; the words do not contain repeated
letters, nor do they share any letters.

The results are shown in the same way as wordle: green indicates the letter
is correct; yellow that the letter is correct but in the wrong position.

Additionally, you are told whether any correct letters you have found are 
split across both words or are all in the same word.