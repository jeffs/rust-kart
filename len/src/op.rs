/// Enumerates the basic operations len can perform.
///
/// The three-letter ops require O(1) RAM, whereas the sorts require O(N).
#[derive(Debug)]
pub enum Op {
    All,         // Print all lines
    Max,         // Print the longest line
    Min,         // Print the shortest line
    One,         // Print the first line
    ReverseSort, // Print all lines, sorted by decreasing length
    Sort,        // Print all lines, sorted by length
}
