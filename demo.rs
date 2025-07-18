// Sample Rust file for testing syntax highlighting and editing
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};

/// A simple struct to demonstrate syntax highlighting
#[derive(Debug, Clone)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

impl Person {
    /// Creates a new Person instance
    pub fn new(name: String, age: u32) -> Self {
        Self {
            name,
            age,
            email: None,
        }
    }

    /// Sets the email for this person
    pub fn set_email(&mut self, email: String) {
        self.email = Some(email);
    }

    /// Gets a greeting message
    pub fn greet(&self) -> String {
        match &self.email {
            Some(email) => format!("Hello {}, contact: {}", self.name, email),
            None => format!("Hello {}", self.name),
        }
    }
}

fn main() -> io::Result<()> {
    // Create some sample data
    let mut people = HashMap::new();

    let mut alice = Person::new("Alice".to_string(), 30);
    alice.set_email("alice@example.com".to_string());

    let bob = Person::new("Bob".to_string(), 25);

    people.insert(1, alice);
    people.insert(2, bob);

    // Print information
    for (id, person) in &people {
        println!("ID: {}, {}", id, person.greet());
    }

    // Demonstrate some pattern matching
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: u32 = numbers.iter().sum();

    println!("Sum of numbers: {}", sum);

    // File operations
    let filename = "test_output.txt";
    let mut file = File::create(filename)?;
    writeln!(file, "This is a test file created by the demo program")?;
    writeln!(file, "Number of people: {}", people.len())?;

    println!("Demo completed successfully!");

    Ok(())
}

// A macro for demonstration
macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("DEBUG: {}", format_args!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_creation() {
        let person = Person::new("Test".to_string(), 20);
        assert_eq!(person.name, "Test");
        assert_eq!(person.age, 20);
        assert!(person.email.is_none());
    }

    #[test]
    fn test_person_email() {
        let mut person = Person::new("Test".to_string(), 20);
        person.set_email("test@example.com".to_string());
        assert!(person.email.is_some());
    }
}
