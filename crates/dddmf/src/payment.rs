struct EmailAddress(String);
struct CardNumber(String);

#[rustfmt::skip]
enum CardType { Visa, Mastercard }

struct CreditCardInfo {
    card_type: CardType,
    card_number: CardNumber,
}

enum PaymentMethod {
    Cash,
    Paypal(EmailAddress),
    Card(CreditCardInfo),
}

struct PaymentAmount(u64);

#[rustfmt::skip]
enum Currency { Eur, Usd }

/// Final type built from many smaller types:
///
/// Composition ftw!
struct Payment {
    amount: PaymentAmount,
    currency: Currency,
    method: PaymentMethod,
}
