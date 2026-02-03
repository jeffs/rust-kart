#[rustfmt::skip]
enum Suit { Club, Diamond, Spade, Heart }

#[rustfmt::skip]
enum Rank { Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace }

struct Card(Suit, Rank);

struct Hand(Vec<Card>);
struct Deck(Vec<Card>);

struct ShuffledDeck(Vec<Card>);
type Shuffle = fn(Deck) -> ShuffledDeck;

#[rustfmt::skip]
struct Player { name: String, hand: Hand }

#[rustfmt::skip]
struct Game { deck: Deck, players: Vec<Player> }

type Deal = fn(ShuffledDeck) -> (ShuffledDeck, Card);

type PickupCard = fn(Hand, Card) -> Hand;
