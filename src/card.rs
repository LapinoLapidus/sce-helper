pub mod card {
    use std::fmt::{Debug, Formatter};
    use ansi_term::Colour::{Blue, Yellow, RGB, White};

    pub struct Card {
        pub game_name: String,
        pub game_id: String,
        pub name: String,
        pub stock: String,
        pub worth: String,
        pub price: String,
    }

    impl Debug for Card {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} - {}\n{} - {}\n{} - {}\n{} - {}\n",
                   Yellow.bold().paint(&self.game_name),
                   Blue.bold().paint(&self.name),
                   RGB(255, 165, 0).bold().paint("Stock"),
                   White.bold().paint(&self.stock),
                   RGB(230, 230, 250).bold().paint("Price"),
                   White.paint(&self.price),
                   RGB(131, 67, 32).bold().paint("Worth"),
                   White.paint(&self.worth)
                    )
        }
    }
}
