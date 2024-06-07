use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;

/// Struct showing the result for an individual letter.
#[derive(Debug, PartialEq, Eq)]
enum LetterResult {
    CorrectLetterCorrectPlace,
    CorrectLetterWrongPlace,
    WrongLetter,
}

/// Struct showing the result for a guessed word.
struct GuessResult {
    letter_results: Vec<LetterResult>,
    both_words: bool,
}

/// Struct showing the result for a guessed word in a user-friendly way.
struct ProcessedGuessResult {
    is_correct: bool,
    message: String,
}

/// Filter words with repeated letters out of "data/word-bank.txt" and write to file "showing the result for a guessed word."
pub fn write_unique_words() {
    let contents = fs::read_to_string("data/word-bank.txt").unwrap();
    let words: Vec<&str> = contents
        .lines()
        .filter(|line| None == check_repeated_letters(line))
        .collect();
    let text = words.join("\n");
    let _ = fs::write("data/word-bank-unique.txt", text);
}

/// Return set of unique characters in `word`.
fn get_unqiue_chars(word: &str) -> HashSet<char> {
    word.chars().collect()
}

/// Check if `word` contains repeated letters.
/// 
/// Return `None` if all letters are unique.
/// Otherwise, return HashMap with count for each letter.
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

/// Read "data/word-bank-unique.txt" to vector.
fn get_word_bank() -> Vec<String> {
    // let contents = fs::read_to_string("data/word-bank-unique.txt").unwrap();
    let contents = include_str!("../data/word-bank-unique.txt").to_owned();
    contents.split_whitespace().map(str::to_string).collect()
}

/// Read "data/valid-words.txt" to vector.
fn get_valid_words() -> Vec<String> {
    // let contents = fs::read_to_string("data/valid-words.txt").unwrap();
    let contents = include_str!("../data/valid-words.txt").to_owned();
    contents.split_whitespace().map(str::to_string).collect()
}

/// Pick two words (with no overlapping letters) from word bank.
/// 
/// # Arguments
/// 
/// * `max_iterations` - Maximum number of attempts to find words that don't share letters.
fn select_words(max_iterations: u8) -> Vec<String> {
    let word_bank = get_word_bank();

    let mut count = 0;
    let words = loop {
        if count >= max_iterations {
            panic!("Could not find non-overlapping words in {} iterations", max_iterations)
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

/// Struct holding the game state whilst in operation.
struct GameState {
    answers: Vec<String>,
    valid_words: Vec<String>,
    guess_count: u8,
    max_guesses: u8,
}

impl GameState {

    /// Create new GameState
    /// 
    /// # Arguments
    /// 
    /// * `max_guesses` - The maximum number of guesses a user is allowed.
    pub fn new(max_guesses: u8) -> GameState {
        let valid_words = get_valid_words();
        let answers = select_words(100);

        GameState {
            answers: answers,
            valid_words: valid_words,
            guess_count: 0,
            max_guesses: max_guesses,
        }
    }

    /// Guess an answer, returning `ProcessedGuessResult` (or Error if `guess` is not a valid word.)
    /// 
    /// # Arguments
    /// 
    /// * `guess` - The user's guess
    pub fn guess(&mut self, guess: &String) -> Result<ProcessedGuessResult, String> {
        if self.validate_guess(guess) == false {
            let s = format!("Not a valid word! Please guess again");
            return Err(s);
        }

        let result = self.process_guess(guess);
        self.increment_guess_count();
        Ok(result)
    }

    /// Return True if the maximum number of guesses has been met or exceeded.
    pub fn out_of_guesses(&self) -> bool {
        self.guess_count >= self.max_guesses
    }

    /// Return message to display to user if game is lost.
    pub fn game_lost_message(&self) -> String {
        let answers_string = self.answers_string();
        format!("Bad luck! {}", answers_string)
    }

    /// Return message to display to user if game is won.
    pub fn game_won_message(&self) -> String {
        let answers_string = self.answers_string();
        format!("Congratulations! {}", answers_string)
    }

    /// Return string of current guess count / maximum guesses.
    pub fn guess_count_message(&self) -> String {
        let print_guess_count = self.guess_count + 1;
        format!("Guess {}/{}:", print_guess_count, self.max_guesses)
    }

    /// Return user-friendly string detailing the answers.
    fn answers_string(&self) -> String {
        format!(
            "The answers were {} and {}",
            self.answers[0].to_ascii_uppercase(),
            self.answers[1].to_ascii_uppercase(),
        )
    }

    fn increment_guess_count(&mut self) {
        self.guess_count += 1;
    }

    /// Return true if `guess` is in valid words list.
    fn validate_guess(&self, guess: &String) -> bool {
        self.valid_words.contains(guess)
    }

    /// Return `GuessResult` for `guess`, detailing the result for each letter in `guess` and whether this represents both answers.
    fn check_guess(&self, guess: &String) -> GuessResult {
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

            let result = if letter == self.answers[0].chars().nth(i).unwrap() {
                guessed_in_answers[0] = true;
                LetterResult::CorrectLetterCorrectPlace
            } else if letter == self.answers[1].chars().nth(i).unwrap() {
                guessed_in_answers[1] = true;
                LetterResult::CorrectLetterCorrectPlace
            } else if self.answers[0].contains(letter) {
                guessed_in_answers[0] = true;
                LetterResult::CorrectLetterWrongPlace
            } else if self.answers[1].contains(letter) {
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

    /// Return `ProcessedGuessResult`, with whether `guess` was correct and a message to display to the user.
    fn process_guess(&self, guess: &String) -> ProcessedGuessResult {
        let mut format_guess_check = String::new();
        let guess_result = self.check_guess(&guess);

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

        let is_correct = self.answers.contains(guess);

        if is_correct == false {
            let s = if guess_result.both_words {
                format!("  (both words)")
            } else {
                format!("  (same word)")
            };
            format_guess_check += &s;
        }

        ProcessedGuessResult {
            is_correct: is_correct,
            message: format_guess_check,
        }
    }
}

pub fn play() {
    let mut state = GameState::new(6);

    write("Welcome to QWordle!");

    loop {
        if state.out_of_guesses() {
            let s = state.game_lost_message();
            write(&s);
            break;
        }

        let s = state.guess_count_message();
        write(&s);

        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer).unwrap();

        buffer = buffer.trim().to_string().to_ascii_lowercase();

        match state.guess(&buffer) {
            Err(s) => {
                write(&s);
                continue;
            }
            Ok(result) => {
                write(&result.message);

                if result.is_correct {
                    let s = state.game_won_message();
                    write(&s);
                    break;
                }
            }
        }

        write("");
    }
}

fn write(s: &str) {
    println!("{s}")
}
