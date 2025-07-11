use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "AdminPassword123";
    let hashed = hash(password, DEFAULT_COST).expect("Failed to hash password");
    println!("{}", hashed);
}