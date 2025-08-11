fn process_data() {
    let user_name = get_user_name();
    println!("Processing user: {}", user_name);
    update_user_name(user_name);
}

fn get_user_name() -> String {
    "John Doe".to_string()
}

fn update_user_name(user_name: String) {
    println!("Updating: {}", user_name);
}