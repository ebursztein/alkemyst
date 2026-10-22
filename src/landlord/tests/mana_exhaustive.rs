use landlord::mana::{Color, ManaCost, Source};

fn assign(pips: &[u8], sources: &[u8], spent: u8) -> bool {
    let Some((pip, remaining)) = pips.split_first() else {
        return true;
    };
    sources.iter().enumerate().any(|(i, source)| {
        spent & (1 << i) == 0 && source & pip != 0 && assign(remaining, sources, spent | (1 << i))
    })
}

#[test]
fn source_assignments_match_exhaustive_small_boros_graphs() {
    let variants = [
        (1, vec![Color::White]),
        (2, vec![Color::Red]),
        (3, vec![Color::White, Color::Red]),
        (4, vec![Color::Colorless]),
    ];
    let cases = [
        ("{W}{R}", vec![1, 2], 0),
        ("{W}{W}", vec![1, 1], 0),
        ("{R/W}{C}", vec![3, 4], 0),
        ("{1}{W}{R}", vec![1, 2], 1),
        ("{2}", vec![], 2),
        ("{W}{R}{R/W}", vec![1, 2, 3], 0),
    ];
    for size in 0..=4 {
        for encoded in 0..4usize.pow(size) {
            let mut code = encoded;
            let choices = (0..size)
                .map(|_| {
                    let choice = code % 4;
                    code /= 4;
                    choice
                })
                .collect::<Vec<_>>();
            let sources = choices
                .iter()
                .map(|i| Source::new(&variants[*i].1).unwrap())
                .collect::<Vec<_>>();
            let masks = choices.iter().map(|i| variants[*i].0).collect::<Vec<_>>();
            for (text, pips, generic) in &cases {
                let expected = sources.len() >= pips.len() + generic && assign(pips, &masks, 0);
                let payment = ManaCost::parse(text).unwrap().payment(&sources);
                assert_eq!(payment.is_some(), expected, "{text} with {masks:?}");
                if let Some(mut indices) = payment {
                    let n = indices.len();
                    indices.sort_unstable();
                    indices.dedup();
                    assert_eq!(indices.len(), n);
                }
            }
        }
    }
}
