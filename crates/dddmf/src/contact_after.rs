struct String50(String);

struct PersonalName {
    first_name: String50,
    /// I couldn't bring myself to declare Wlaschin's `String1`.
    middle_initial: Option<char>,
    last_name: String50,
}

struct EmailAddress(String);
struct PhoneNumber(String);

struct CustomerId(u32);
struct OrderId(u32);

/// a new concept
struct VerifiedEmail(EmailAddress);
struct VerificationHash(String);
type VerificationService<'a> = &'a dyn Fn(EmailAddress, &VerificationHash) -> Option<VerifiedEmail>;

enum EmailContactInfo {
    Unverified(EmailAddress),
    Verified(VerifiedEmail),
}

struct Contact {
    name: PersonalName,
    email: EmailContactInfo,
}

/// Constraints code points, not characters, as in Wlaschin's F# example.
/// <https://learn.microsoft.com/en-us/dotnet/api/system.string.length#remarks>
fn create_string50(s: String) -> Option<String50> {
    (s.len() <= 50).then_some(String50(s))
}

fn create_email_address(s: String) -> Option<EmailAddress> {
    s.contains('@').then_some(EmailAddress(s))
}

/// No validation needed. Errors can't occur.
fn send_password_reset(_: &VerifiedEmail) {
    // ...
}
