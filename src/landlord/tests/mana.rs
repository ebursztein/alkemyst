use landlord::mana::{Color, ManaCost, Source};

fn source(colors: &[Color]) -> Source {
    Source::new(colors).unwrap()
}

#[test]
fn matching_reassigns_dual_sources_and_never_double_spends() {
    use Color::*;
    let cost = ManaCost::parse("{W}{R}").unwrap();
    let sources = [source(&[White, Red]), source(&[White])];
    let payment = cost.payment(&sources).unwrap();
    assert_eq!(payment.len(), 2);
    assert_ne!(payment[0], payment[1]);
    assert!(cost.payment(&sources[..1]).is_none());
    assert!(
        cost.payment(&[source(&[White]), source(&[White])])
            .is_none()
    );
}

#[test]
fn explicit_colorless_differs_from_generic_and_hybrid_is_one_pip() {
    use Color::*;
    let white = source(&[White]);
    assert!(ManaCost::parse("{1}").unwrap().payment(&[white]).is_some());
    assert!(ManaCost::parse("{C}").unwrap().payment(&[white]).is_none());
    assert!(
        ManaCost::parse("{C}")
            .unwrap()
            .payment(&[source(&[Colorless])])
            .is_some()
    );
    let hybrid = ManaCost::parse("{1}{R/W}{R/W}").unwrap();
    assert_eq!(hybrid.mana_value(), 3);
    assert!(
        hybrid
            .payment(&[white, source(&[Red]), source(&[Green])])
            .is_some()
    );
    assert!(
        hybrid
            .payment(&[white, source(&[Green]), source(&[Green])])
            .is_none()
    );
    assert_eq!(ManaCost::parse("").unwrap().payment(&[]), Some(vec![]));
}

#[test]
fn malformed_and_unmodeled_costs_are_rejected_instead_of_guessed() {
    for input in [
        "W", "{W", "{W}}", "{}", "{X}", "{W/P}", "{2/W}", "{S}", "{999}", "{W/U/B}", "{w}",
    ] {
        assert!(ManaCost::parse(input).is_err(), "{input}");
    }
    assert!(ManaCost::parse(&"{W}".repeat(65)).is_err());
    assert!(Source::new(&[]).is_err());
}

#[test]
fn upstream_adeliz_and_clarion_regressions_use_injected_sources() {
    use Color::*;
    // Original hand.rs regressions: Detection Tower cannot pay colored pips;
    // Temple Garden/Hallowed Fountain/Breeding Pool must be assigned jointly.
    let adeliz = ManaCost::parse("{1}{U}{R}").unwrap();
    assert!(
        adeliz
            .payment(&[
                source(&[Colorless]),
                source(&[Colorless]),
                source(&[Blue, Red])
            ])
            .is_none()
    );
    assert!(
        adeliz
            .payment(&[source(&[Colorless]), source(&[Blue]), source(&[Red])])
            .is_some()
    );
    let clarion = ManaCost::parse("{G}{G}{W}{W}{W}{U}{U}").unwrap();
    assert!(
        clarion
            .payment(&[
                source(&[Green, White]),
                source(&[Blue, White]),
                source(&[Green, Blue]),
                source(&[Green]),
                source(&[White]),
                source(&[Blue]),
                source(&[White])
            ])
            .is_some()
    );
}

#[test]
fn alternative_source_abilities_combine_colors_but_still_produce_one_unit() {
    use Color::*;
    let source = source(&[White]).union(source(&[Red]));
    assert!(ManaCost::parse("{R}").unwrap().payment(&[source]).is_some());
    assert!(ManaCost::parse("{W}").unwrap().payment(&[source]).is_some());
    assert!(
        ManaCost::parse("{W}{R}")
            .unwrap()
            .payment(&[source])
            .is_none()
    );
}

#[test]
fn generic_reductions_never_remove_colored_requirements() {
    let cost = ManaCost::parse("{3}{R}").unwrap();
    assert_eq!(cost.reduced_generic(2), ManaCost::parse("{1}{R}").unwrap());
    assert_eq!(cost.reduced_generic(99), ManaCost::parse("{R}").unwrap());
    assert!(
        cost.reduced_generic(99)
            .payment(&[source(&[Color::White])])
            .is_none()
    );
}

#[test]
fn source_color_inspection_does_not_change_payment_capacity() {
    let s = source(&[Color::White, Color::Red]);
    assert!(s.supports(Color::White));
    assert!(s.supports(Color::Red));
    assert!(!s.supports(Color::Blue));
}
