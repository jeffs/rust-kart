struct AppleVariety;
struct BananaVariety;
struct CherryVariety;

/// Compose with "AND"
///
/// A record type
///      ↘
struct FruitSalad {
    apple: AppleVariety, // ← Another type, the set of all possible apples
    banana: BananaVariety,
    cherry: CherryVariety,
}

/// Compose with "OR"
///
/// A choice type
///      ↘
enum Snack {
    Apple(AppleVariety),   // ← Again, the set of all possible apples
    Banana(BananaVariety), //
    Cherry(CherryVariety), // Not generally available in non-FP languages
}
