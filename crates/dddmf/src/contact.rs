/// This code does not communicate important design decisions ☹
///
/// - Which values are optional?
/// - What are the constraints?
/// - Any domain logic?
///
/// Functional domain modeling CAN communicate all these decisions!
struct Contact {
    first_name: String,
    middle_initial: String,
    last_name: String,

    email_address: String,
    is_email_verified: bool,
}
