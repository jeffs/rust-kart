#[rustfmt::skip]
enum Suit { Club, Diamond, Spade, Heart }

#[rustfmt::skip]
enum Rank { Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King }

struct Card(Suit, Rank);

struct Hand(Vec<Card>);
struct Deck(Vec<Card>);

#[rustfmt::skip]
struct Player { name: String, hand: Hand }

#[rustfmt::skip]
struct Game { deck: Deck, players: Vec<Player> }

type Deal = fn(Deck) -> (Deck, Card);

type PickupCard = fn(Hand, Card) -> Hand;
