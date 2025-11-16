use cp_test::lexer::{default_rules, Lexer};

fn main() {
    let input = "let x = 42 + 3.14\nif x > 10 {\n    println!(\"hello\")\n}";

    let rules = default_rules();
    let mut lexer = Lexer::from_str(input, rules);

    println!("Tokenizing: {}\n", input);
    println!("Tokens:");

    for (i, token) in lexer.tokenize().iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
}

