use std::collections::HashMap;
use std::process::exit;

#[derive(Default, Debug)]
pub struct RpnCalculator {
    stack: Vec<f64>,
    pub mem: HashMap<String, f64>,
}

impl RpnCalculator {
    pub fn interpret_string(&mut self, input: &String) {
        let mut store = false;
        for word in input.split_ascii_whitespace() {
            if let Ok(val) = word.parse::<f64>() {
                self.stack.push(val);
            } else if word == "sto" {
                store = true;
            } else if word == "+" {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(a + b);
            } else if word == "*" {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(a * b);
            } else if word == "/" {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b / a);
            } else if word == "sqrt" {
                let a = self.stack.pop().unwrap();
                self.stack.push(a.sqrt());
            } else if self.mem.contains_key(word) {
                self.stack.push(*self.mem.get(word).unwrap());
            } else if word == "pi" {
                self.stack.push(std::f64::consts::PI);
            } else if store {
                let a = self.stack.pop().unwrap();
                self.mem.insert(word.to_string(), a);
            } else {
                eprintln!("Unrecognised token in rpnstr: {}", word);
                exit(1);
            }
        }
    }
}
