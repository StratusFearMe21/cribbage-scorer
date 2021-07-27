use std::{borrow::Borrow, env};

use console::style;
use once_cell::sync::Lazy;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

const SETSOF2: [[usize; 2]; 10] = [
    [0, 1],
    [0, 2],
    [0, 3],
    [0, 4],
    [1, 2],
    [1, 3],
    [1, 4],
    [2, 3],
    [2, 4],
    [3, 4],
];
const SETSOF3: [[usize; 3]; 10] = [
    [0, 1, 2],
    [0, 1, 3],
    [0, 2, 3],
    [1, 2, 3],
    [0, 1, 4],
    [0, 2, 4],
    [1, 2, 4],
    [0, 3, 4],
    [1, 3, 4],
    [2, 3, 4],
];
const SETSOF4: [[usize; 4]; 5] = [
    [0, 1, 2, 3],
    [0, 1, 2, 4],
    [0, 1, 3, 4],
    [0, 2, 3, 4],
    [1, 2, 3, 4],
];

static CARDS: Lazy<[Card; 5]> = Lazy::new(|| {
    let mut args = env::args();
    let mut cards = [
        Card::from(args.nth(1).unwrap()),
        Card::from(args.next().unwrap()),
        Card::from(args.next().unwrap()),
        Card::from(args.next().unwrap()),
        Card::from(args.next().unwrap()),
    ];
    cards.par_sort();
    cards
});

#[derive(Hash)]
enum Card {
    Number(u8),
    Queen,
    King,
    Jack,
    Ace,
}

enum Scoring {
    Fifteen,
    Pair,
    ThirtyOne,
    Stop,
}

impl From<String> for Card {
    fn from(s: String) -> Self {
        match s.borrow() {
            "Q" | "q" => Self::Queen,
            "K" | "k" => Self::King,
            "J" | "j" => Self::Jack,
            "A" | "a" => Self::Ace,
            _ => Self::Number(s.parse().unwrap()),
        }
    }
}

impl ToString for Card {
    fn to_string(&self) -> String {
        match &self {
            Card::Ace => "A".to_string(),
            Card::King => "K".to_string(),
            Card::Jack => "J".to_string(),
            Card::Queen => "Q".to_string(),
            Card::Number(n) => n.to_string(),
        }
    }
}

impl Card {
    fn into_u8_cribbage(&self) -> u8 {
        match &self {
            Self::Ace => 1,
            Self::King | Self::Jack | Self::Queen => 10,
            Self::Number(num) => num.to_owned(),
        }
    }
    fn into_u8_normal(&self) -> u8 {
        match &self {
            Self::Ace => 1,
            Self::Jack => 11,
            Self::Queen => 12,
            Self::King => 13,
            Self::Number(num) => num.to_owned(),
        }
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.into_u8_normal().cmp(&other.into_u8_normal())
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.into_u8_normal() == other.into_u8_normal()
    }
}

impl Eq for Card {}

fn print_cards(cardsindexes: Vec<usize>) {
    for i in 0..5 {
        if cardsindexes.contains(&i) {
            print!("{} ", style(CARDS[i].to_string()).green());
        } else {
            print!("{} ", CARDS[i].to_string());
        }
    }
}

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded();
    rayon::scope(|t| {
        t.spawn(move |_| {
            let mut points: u8 = 0;
            loop {
                match rx.recv() {
                    Ok((Scoring::Fifteen, cardsindexes)) => {
                        points += 2;
                        print_cards(cardsindexes);
                        println!("15 for {}", points);
                    }
                    Ok((Scoring::ThirtyOne, cardsindexes)) => {
                        points += 2;
                        print_cards(cardsindexes);
                        println!("31 for {}", points);
                    }
                    Ok((Scoring::Pair, cardsindexes)) => {
                        points += 2;
                        print_cards(cardsindexes);
                        println!("Pair for {}", points);
                    }
                    Ok((Scoring::Stop, _)) => return,
                    _ => {}
                }
            }
        });
        rayon::scope(|s| {
            s.spawn(|_| {
                SETSOF2.par_iter().for_each(|i| {
                    if CARDS[i[0]] == CARDS[i[1]] {
                        tx.send((Scoring::Pair, i.to_vec())).unwrap();
                    }
                });
            });
            s.spawn(|_| {
                SETSOF2.par_iter().for_each(|i| {
                    if CARDS[i[0]].into_u8_cribbage() + CARDS[i[1]].into_u8_cribbage() == 15 {
                        tx.send((Scoring::Fifteen, i.to_vec())).unwrap();
                    }
                });
            });
            s.spawn(|_| {
                SETSOF3.par_iter().for_each(|i| {
                    if CARDS[i[0]].into_u8_cribbage()
                        + CARDS[i[1]].into_u8_cribbage()
                        + CARDS[i[2]].into_u8_cribbage()
                        == 15
                    {
                        tx.send((Scoring::Fifteen, i.to_vec())).unwrap();
                    }
                });
            });
            s.spawn(|_| {
                SETSOF4.par_iter().for_each(|i| {
                    if CARDS[i[0]].into_u8_cribbage()
                        + CARDS[i[1]].into_u8_cribbage()
                        + CARDS[i[2]].into_u8_cribbage()
                        + CARDS[i[3]].into_u8_cribbage()
                        == 15
                    {
                        tx.send((Scoring::Fifteen, i.to_vec())).unwrap();
                    }
                });
            });
            s.spawn(|_| {
                if CARDS[0].into_u8_cribbage()
                    + CARDS[1].into_u8_cribbage()
                    + CARDS[2].into_u8_cribbage()
                    + CARDS[3].into_u8_cribbage()
                    + CARDS[4].into_u8_cribbage()
                    == 15
                {
                    tx.send((Scoring::ThirtyOne, vec![0, 1, 2, 3, 4])).unwrap();
                }
            });
            s.spawn(|_| {
                SETSOF4.par_iter().for_each(|i| {
                    if CARDS[i[0]].into_u8_cribbage()
                        + CARDS[i[1]].into_u8_cribbage()
                        + CARDS[i[2]].into_u8_cribbage()
                        + CARDS[i[3]].into_u8_cribbage()
                        == 31
                    {
                        tx.send((Scoring::ThirtyOne, i.to_vec())).unwrap();
                    }
                });
            });
            s.spawn(|_| {
                if CARDS[0].into_u8_cribbage()
                    + CARDS[1].into_u8_cribbage()
                    + CARDS[2].into_u8_cribbage()
                    + CARDS[3].into_u8_cribbage()
                    + CARDS[4].into_u8_cribbage()
                    == 31
                {
                    tx.send((Scoring::ThirtyOne, vec![0, 1, 2, 3, 4])).unwrap();
                }
            });
        });
        tx.send((Scoring::Stop, Vec::with_capacity(0))).unwrap();
    });
}
