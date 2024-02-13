use rand::Rng;
use std::io::{self};
fn main() {
    println!("Guess the number!");

    let number_to_guess = rand::thread_rng().gen_range(1..100);

    loop {
        let mut user_input = String::new();
        println!("Please input your guess.");

        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let user_input: u32 = match user_input.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed: {}", user_input);

        match user_input.cmp(&number_to_guess) {
            std::cmp::Ordering::Less => println!("Too small!"),
            std::cmp::Ordering::Greater => println!("Too big!"),
            std::cmp::Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
        println!()
        
    }
}
