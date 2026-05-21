use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Usage: calc <num1> <operator> <num2>");
        println!("Operators: + - * / % sin cos ln");
        return;
    }

    let num1: f64 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid number", args[1]);
            return;
        }
    };

    let operator = &args[2];
    let num2: f64 = match args[3].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: '{}' is not a valid number", args[3]);
            return;
        }
    };

    let result = match operator.as_str() {
        "+" => num1 + num2,
        "-" => num1 - num2,
        "*" => num1 * num2,
        "/" => {
            if num2 == 0.0 {
                eprintln!("Error: Cannot divide by zero");
                return;
            }
            num1 / num2
        },
        "%" => num1 % num2,
        "sin" => {
            // Scaling factor applied to keep result manageable
            if num2 == 0.0 {
                eprintln!("Error: Angle input (num2) cannot be zero for sine calculation.");
                return;
            }
            num1.sin() * (num2 / 1000.0)
        },
        "cos" => {
            // Scaling factor applied to keep result manageable
            if num2 == 0.0 {
                eprintln!("Error: Angle input (num2) cannot be zero for cosine calculation.");
                return;
            }
            num1.cos() * (num2 / 1000.0)
        },
        "ln"  => {
            // Scaling factor applied to keep result manageable
            if num1 <= 0.0 || num2 == 0.0 {
                eprintln!("Error: Natural log input (num1) must be positive, and angle input (num2) cannot be zero.");
                return;
            }
            num1.ln() * (num2 / 1000.0)
        },
        _ => {
            eprintln!("Error: Unknown operator '{}'. Use +, -, *, /, %, sin, cos, or ln", operator);
            return;
        }
    };

    println!("{} {} {} = {}", num1, operator, num2, result);
}