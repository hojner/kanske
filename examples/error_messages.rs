use kanske_lib::parser::{lexer::Lexer, parse::Parser};

fn main() {
    println!("=== Test 1: Invalid transform ===");
    let input1 = r#"profile test { output DP-1 transform 45 }"#;
    let mut lexer1 = Lexer::new(input1.to_string());
    let tokens1 = lexer1.tokenizer().unwrap();
    let mut parser1 = Parser::new(tokens1);
    match parser1.parse() {
        Err(e) => println!("Error: {}\n", e),
        _ => {}
    }

    println!("=== Test 2: Invalid adaptive_sync ===");
    let input2 = r#"profile test { output DP-1 adaptive_sync maybe }"#;
    let mut lexer2 = Lexer::new(input2.to_string());
    let tokens2 = lexer2.tokenizer().unwrap();
    let mut parser2 = Parser::new(tokens2);
    match parser2.parse() {
        Err(e) => println!("Error: {}\n", e),
        _ => {}
    }

    println!("=== Test 3: Invalid resolution ===");
    let input3 = r#"profile test { output DP-1 mode 1920-1080 }"#;
    let mut lexer3 = Lexer::new(input3.to_string());
    let tokens3 = lexer3.tokenizer().unwrap();
    let mut parser3 = Parser::new(tokens3);
    match parser3.parse() {
        Err(e) => println!("Error: {}\n", e),
        _ => {}
    }

    println!("=== Test 4: Unterminated string ===");
    let input4 = "profile \"test\n    output DP-1 enable\n}";
    let mut lexer4 = Lexer::new(input4.to_string());
    match lexer4.tokenizer() {
        Err(e) => println!("Error: {}\n", e),
        _ => {}
    }
    
    println!("=== Test 5: Invalid position ===");
    let input5 = r#"profile test { output DP-1 position 1920 }"#;
    let mut lexer5 = Lexer::new(input5.to_string());
    let tokens5 = lexer5.tokenizer().unwrap();
    let mut parser5 = Parser::new(tokens5);
    match parser5.parse() {
        Err(e) => println!("Error: {}", e),
        _ => {}
    }
}
