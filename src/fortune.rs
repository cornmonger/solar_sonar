use rand::RngExt;


const FORTUNE: &'static [&'static str] = &[
    "I talk in intel when you're not watching.",
    "All their ice are belong to us.",
    "Locked, stocked, and ready to interrupt your Netflix.",
    "We don't need showers where we are going.",
];

pub(crate) fn fortune() -> &'static str {
    let mut rng = rand::rng();
    let idx = rng.random_range(0..FORTUNE.len());
    FORTUNE[idx]
}
