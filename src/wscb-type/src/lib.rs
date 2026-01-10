pub mod error;
pub mod graph;
pub mod graph_f;

/// The hashmap that wscb use.
pub type HashMap<K, V> = ::std::collections::HashMap<K, V, ::ahash::RandomState>;

/// The hashset that wscb use.
pub type HashSet<T> = ::std::collections::HashSet<T, ::ahash::RandomState>;
