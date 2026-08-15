//! Hand-pig duel resolution.

use std::cmp::Ordering;

use rand::RngExt;

use crate::enums::DuelResult;

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct DuelOutcome {
    /// A draw reports `true` — both sides are credited a win.
    pub first_wins: bool,
    pub status: DuelResult,
    /// Weight transferred from loser to winner.
    pub damage: i32,
}

/// Weights are clamped to 1: a stored 0 would divide by zero below and build
/// an empty `random_range`.
pub fn resolve_duel<R: RngExt>(
    rng: &mut R,
    first_weight: i32,
    second_weight: i32,
) -> DuelOutcome {
    let first_weight = first_weight.max(1);
    let second_weight = second_weight.max(1);

    let mut first_chance = first_weight;
    let mut second_chance = second_weight;

    // Past 5x, the *lighter* pig's ceiling is raised — the heavier one's
    // range is untouched. Tiers below still measure against the real weight.
    if first_chance / second_chance > 5 {
        second_chance = first_chance / 5;
    } else if second_chance / first_chance > 5 {
        first_chance = second_chance / 5;
    }

    let first_random = rng.random_range(0..first_chance);
    let second_random = rng.random_range(0..second_chance);

    let win_variant = |random: i32, weight: i32| match random {
        r if r >= (weight * 99) / 100 => DuelResult::Knockout,
        r if r >= (weight * 90) / 100 => DuelResult::Critical,
        _ => DuelResult::Win,
    };

    let (status, first_wins) = match first_random.cmp(&second_random) {
        Ordering::Greater => (win_variant(first_random, first_weight), true),
        Ordering::Less => (win_variant(second_random, second_weight), false),
        Ordering::Equal => (DuelResult::Draw, true),
    };

    let (winner_weight, looser_weight) = if first_wins {
        (first_weight, second_weight)
    } else {
        (second_weight, first_weight)
    };

    let damage = match status {
        DuelResult::Win => looser_weight / 8,
        DuelResult::Critical => looser_weight / 3,
        DuelResult::Knockout => (looser_weight as f32 / 1.5) as i32,
        DuelResult::Draw => looser_weight.max(winner_weight) / 8,
    };

    DuelOutcome { first_wins, status, damage }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    fn roll(seed: u64, first: i32, second: i32) -> DuelOutcome {
        resolve_duel(&mut StdRng::seed_from_u64(seed), first, second)
    }

    #[test]
    fn is_reproducible_from_a_seed() {
        assert_eq!(roll(7, 500, 400), roll(7, 500, 400));
    }

    #[test]
    fn equal_rolls_are_a_draw_and_credit_the_first_pig() {
        let outcome = roll(1, 1, 1);

        assert_eq!(outcome.status, DuelResult::Draw);
        assert!(outcome.first_wins);
        assert_eq!(outcome.damage, 0); // max(1, 1) / 8
    }

    #[test]
    fn damage_follows_the_result_tier() {
        for seed in 0..500u64 {
            let outcome = roll(seed, 800, 640);
            let looser = if outcome.first_wins { 640 } else { 800 };
            let winner = if outcome.first_wins { 800 } else { 640 };

            let expected = match outcome.status {
                DuelResult::Win => looser / 8,
                DuelResult::Critical => looser / 3,
                DuelResult::Knockout => (looser as f32 / 1.5) as i32,
                DuelResult::Draw => looser.max(winner) / 8,
            };

            assert_eq!(outcome.damage, expected, "seed {seed}");
        }
    }

    #[test]
    fn produces_wins_criticals_and_knockouts_over_many_seeds() {
        let mut seen = (false, false, false);

        for seed in 0..20_000u64 {
            match roll(seed, 1_000, 1_000).status {
                DuelResult::Win => seen.0 = true,
                DuelResult::Critical => seen.1 = true,
                DuelResult::Knockout => seen.2 = true,
                DuelResult::Draw => {},
            }
        }

        assert_eq!(seen, (true, true, true));
    }

    #[test]
    fn both_sides_win_across_many_seeds() {
        let mut first_won = false;
        let mut second_won = false;

        for seed in 0..1_000u64 {
            if roll(seed, 500, 500).first_wins {
                first_won = true;
            } else {
                second_won = true;
            }
        }

        assert!(first_won && second_won);
    }


    #[test]
    fn the_ratio_rule_raises_the_lighter_pigs_ceiling_not_the_heavier_pigs() {
        // The light pig rolls 0..2_000 (heavy / 5) while the heavy one keeps
        // its full 0..10_000, so the rule softens the gap without inverting it.
        let light_wins = (0..2_000u64)
            .filter(|&seed| !roll(seed, 10_000, 1_000).first_wins)
            .count();

        assert!(
            (50..300).contains(&light_wins),
            "expected the heavy pig to keep a big edge, light won {light_wins}/2000"
        );
    }

    #[test]
    fn the_boosted_light_pig_scores_knockouts_easily() {
        // A documented consequence of the handicap: the light
        // pig rolls up to 2_000 but its Knockout threshold is 99% of its own
        // 1_000 kg, so most of its wins land in the top tier.
        let mut knockouts = 0;
        let mut light_wins = 0;

        for seed in 0..5_000u64 {
            let outcome = roll(seed, 10_000, 1_000);
            if !outcome.first_wins {
                light_wins += 1;
                if outcome.status == DuelResult::Knockout {
                    knockouts += 1;
                }
            }
        }

        assert!(light_wins > 0);
        assert!(
            knockouts * 2 > light_wins,
            "expected most light-pig wins to be knockouts, got {knockouts}/{light_wins}"
        );
    }

    #[test]
    fn the_ratio_rule_applies_in_both_directions() {
        let heavy_first = (0..2_000u64)
            .filter(|&seed| roll(seed, 10_000, 1_000).first_wins)
            .count();
        let heavy_second = (0..2_000u64)
            .filter(|&seed| !roll(seed, 1_000, 10_000).first_wins)
            .count();

        // Not identical (the RNG draws the two ranges in argument order),
        // but the heavy pig dominates either way.
        assert!(heavy_first > 1_500, "{heavy_first}");
        assert!(heavy_second > 1_500, "{heavy_second}");
    }

    #[test]
    fn exactly_five_times_heavier_is_not_boosted() {
        // The guard is `> 5`, so at exactly 5x both pigs roll their own
        // weight and the heavier one wins about five times out of six.
        let heavy_wins = (0..3_000u64)
            .filter(|&seed| roll(seed, 5_000, 1_000).first_wins)
            .count();

        assert!(
            (2_300..2_800).contains(&heavy_wins),
            "expected ~5/6 heavy wins, got {heavy_wins}/3000"
        );
    }


    #[test]
    fn zero_and_negative_weights_do_not_panic() {
        for (first, second) in
            [(0, 0), (0, 100), (100, 0), (-50, 100), (100, -50), (-1, -1)]
        {
            let outcome = roll(11, first, second);
            assert!(outcome.damage >= 0, "{first} vs {second}");
        }
    }

    #[test]
    fn a_zero_weight_pig_is_treated_as_one_kg() {
        assert_eq!(roll(11, 0, 400), roll(11, 1, 400));
        assert_eq!(roll(11, -900, 400), roll(11, 1, 400));
    }
}
