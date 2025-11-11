use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run --example hash_password <password>");
        std::process::exit(1);
    }

    let password = &args[1];

    if password.len() < 8 {
        eprintln!("Error: Password must be at least 8 characters");
        std::process::exit(1);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            println!("Password hash:");
            println!("{}", hash);
            println!();
            println!("You can now use this hash to create an admin user:");
            println!();
            println!("INSERT INTO users (id, username, password_hash, is_admin)");
            println!("VALUES (UUID(), 'admin', '{}', true);", hash);
        }
        Err(e) => {
            eprintln!("Error generating hash: {}", e);
            std::process::exit(1);
        }
    }
}
