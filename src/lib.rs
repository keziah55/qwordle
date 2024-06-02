use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;

#[derive(Debug, PartialEq, Eq)]
enum LetterResult {
    CorrectLetterCorrectPlace,
    CorrectLetterWrongPlace,
    WrongLetter,
}

struct GuessResult {
    letter_results: Vec<LetterResult>,
    both_words: bool,
}

struct ProcessedGuessResult {
    is_correct: bool,
    message: String,
}

pub fn write_unique_words() {
    let contents = fs::read_to_string("data/word-bank.txt").unwrap();
    let words: Vec<&str> = contents
        .lines()
        .filter(|line| None == check_repeated_letters(line))
        .collect();
    let text = words.join("\n");
    let _ = fs::write("data/word-bank-unique.txt", text);
}

fn get_unqiue_chars(word: &str) -> HashSet<char> {
    word.chars().collect()
}

fn check_repeated_letters(word: &str) -> Option<HashMap<char, Vec<u8>>> {
    let letters: HashSet<char> = get_unqiue_chars(word);
    if letters.len() == word.len() {
        None
    } else {
        let mut letter_pos: HashMap<char, Vec<u8>> = HashMap::new();
        for (i, letter) in word.chars().enumerate() {
            let idx = i as u8;
            match letter_pos.get_mut(&letter) {
                None => {
                    letter_pos.insert(letter, vec![idx]);
                    ()
                }
                Some(v) => {
                    v.push(idx);
                    ()
                }
            }
        }
        Some(letter_pos)
    }
}

fn get_word_bank() -> Vec<String> {
    // let contents = fs::read_to_string("data/word-bank-unique.txt").unwrap();
    let contents = include_str!("../data/word-bank-unique.txt").to_owned();
    contents.split_whitespace().map(str::to_string).collect()
}

fn get_valid_words() -> Vec<String> {
    // let contents = fs::read_to_string("data/valid-words.txt").unwrap();
    let contents = include_str!("../data/valid-words.txt").to_owned();
    contents.split_whitespace().map(str::to_string).collect()
}

fn select_words() -> Vec<String> {
    let word_bank = get_word_bank();

    let mut count = 0;
    let words = loop {
        if count >= 100 {
            panic!("Could not find non-overlapping words in 20 iterations")
        }

        let words: Vec<_> = word_bank
            .choose_multiple(&mut rand::thread_rng(), 2)
            .collect();
        let join_words = format!("{}{}", words[0], words[1]);
        match check_repeated_letters(&join_words) {
            None => break words,
            Some(_) => count += 1,
        }
    };

    vec![words[0].clone(), words[1].clone()]
}

fn validate_guess(guess: &String, valid_words: &Vec<String>) -> bool {
    valid_words.contains(guess)
}

fn check_guess(guess: &String, answers: &Vec<String>) -> GuessResult {
    let mut letter_results = Vec::new();
    let mut guessed_in_answers = vec![false, false];

    let repeated_guess_letters = check_repeated_letters(&guess);

    for (i, letter) in guess.chars().enumerate() {
        let idx = i as u8;
        if let Some(_) = repeated_guess_letters {
            // if there's a repeated letter in guess, only get info about the first occurrence
            // (because we know there aren't repeated letters in the answers)
            let map = repeated_guess_letters.as_ref().unwrap();
            let indices = map.get(&letter).unwrap();
            if indices.first().unwrap() != &idx {
                // if we're past the first occurrence, go to next letter in for loop
                continue;
            }
        }

        let result = if letter == answers[0].chars().nth(i).unwrap() {
            guessed_in_answers[0] = true;
            LetterResult::CorrectLetterCorrectPlace
        } else if letter == answers[1].chars().nth(i).unwrap() {
            guessed_in_answers[1] = true;
            LetterResult::CorrectLetterCorrectPlace
        } else if answers[0].contains(letter) {
            guessed_in_answers[0] = true;
            LetterResult::CorrectLetterWrongPlace
        } else if answers[1].contains(letter) {
            guessed_in_answers[1] = true;
            LetterResult::CorrectLetterWrongPlace
        } else {
            LetterResult::WrongLetter
        };

        letter_results.push(result);
    }

    let both_words = guessed_in_answers.iter().all(|&b| b);

    GuessResult {
        letter_results: letter_results,
        both_words: both_words,
    }
}

fn process_guess(guess: &String, answers: &Vec<String>) -> ProcessedGuessResult {
    let mut format_guess_check = String::new();
    let guess_result = check_guess(&guess, &answers);

    for (i, letter_result) in guess_result.letter_results.iter().enumerate() {
        let letter_upper = guess.chars().nth(i).unwrap().to_ascii_uppercase();

        let append_char = match letter_result {
            LetterResult::CorrectLetterCorrectPlace => {
                format!("\x1b[92m{letter_upper}\x1b[0m")
            }
            LetterResult::CorrectLetterWrongPlace => {
                format!("\x1b[93m{letter_upper}\x1b[0m")
            }
            LetterResult::WrongLetter => {
                format!("{letter_upper}")
            }
        };

        format_guess_check += &append_char;
    }

    let is_correct = answers.contains(guess);

    if is_correct == false {
        let s = if guess_result.both_words {
            format!("  (both words)")
        } else {
            format!("  (same word)")
        };
        format_guess_check += &s;
    }

    ProcessedGuessResult{
        is_correct: is_correct, 
        message: format_guess_check
    }

}

pub fn play() {
    let valid_words = get_valid_words();

    let answers = select_words();
    let mut guess_count = 0;
    let max_guesses = 6;

    write("Welcome to QWordle!");

    loop {
        if guess_count >= max_guesses {
            let s = format!(
                "Bad luck! The answers were {} and {}",
                answers[0].to_ascii_uppercase(),
                answers[1].to_ascii_uppercase()
            );
            write(&s);
            break;
        }

        let print_guess_count = guess_count + 1;
        let s = format!("Guess {print_guess_count}/{max_guesses}:");
        write(&s);

        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer).unwrap();

        buffer = buffer.trim().to_string().to_ascii_lowercase();

        if validate_guess(&buffer, &valid_words) == false {
            let s = format!("Not a valid word! Please guess again");
            write(&s);
            continue;
        }

        let result = process_guess(&buffer, &answers);

        write(&result.message);

        if result.is_correct {
            let s = format!(
                "Congratulations! The answers were {} and {}",
                answers[0].to_ascii_uppercase(),
                answers[1].to_ascii_uppercase()
            );
            write(&s);
            break;
        }

        guess_count += 1;
        write("");
    }
}

fn write(s: &str) {
    println!("{s}")
}
