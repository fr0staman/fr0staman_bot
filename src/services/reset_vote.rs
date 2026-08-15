//! Rules behind `/resetpigs`. See `SPEC.md` §4.4.

use chrono::NaiveDateTime;

/// A group may only be reset once per this many days.
pub const RESET_COOLDOWN_DAYS: i64 = 7;

/// Strict majority of present pig owners — both what `/resetpigs` advertises
/// and what [`vote_passed`] enforces.
pub fn quorum_for(total_players: i64) -> i64 {
    total_players / 2 + 1
}

pub fn vote_passed(yes_votes: i64, total_players: i64) -> bool {
    yes_votes >= quorum_for(total_players)
}

/// Days left before this group may be reset again, `None` once elapsed.
pub fn cooldown_days_left(
    reset_at: Option<NaiveDateTime>,
    now: NaiveDateTime,
) -> Option<i64> {
    let last_reset = reset_at?;
    let days_passed = (now - last_reset).num_days();

    (days_passed < RESET_COOLDOWN_DAYS)
        .then(|| RESET_COOLDOWN_DAYS - days_passed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::datetime;

    #[test]
    fn quorum_is_a_strict_majority() {
        let cases =
            [(1, 1), (2, 2), (3, 2), (4, 3), (5, 3), (10, 6), (11, 6)];

        for (total, expected) in cases {
            assert_eq!(quorum_for(total), expected, "total {total}");
        }
    }

    #[test]
    fn vote_passes_on_more_than_half() {
        assert!(!vote_passed(0, 4));
        assert!(!vote_passed(2, 4));
        assert!(vote_passed(3, 4));

        assert!(!vote_passed(1, 3));
        assert!(vote_passed(2, 3));
    }

    #[test]
    fn the_advertised_quorum_is_exactly_what_passes() {
        // `/resetpigs` announces `quorum_for(total)` when the vote opens and
        // `callback_reset_vote` resolves with `vote_passed`. Reaching the
        // announced number must carry the vote, and one vote fewer must not.
        for total in 1..200i64 {
            let advertised = quorum_for(total);

            assert!(
                vote_passed(advertised, total),
                "total {total}: the advertised quorum did not pass"
            );
            assert!(
                !vote_passed(advertised - 1, total),
                "total {total}: passed one vote below the advertised quorum"
            );
        }
    }

    #[test]
    fn the_two_forms_of_the_majority_rule_agree() {
        // `vote_passed` used to be spelled `yes * 2 > total`. The two are
        // equivalent everywhere, which is why the duplicate could sit there
        // unnoticed; this pins that before the old form is forgotten.
        for total in 0..500i64 {
            for yes in 0..=total + 1 {
                assert_eq!(
                    vote_passed(yes, total),
                    yes * 2 > total,
                    "yes {yes} of {total}"
                );
            }
        }
    }

    #[test]
    fn a_unanimous_vote_always_passes() {
        for total in 1..200i64 {
            assert!(vote_passed(total, total), "total {total}");
        }
    }

    #[test]
    fn no_votes_never_passes() {
        for total in 1..200i64 {
            assert!(!vote_passed(0, total), "total {total}");
        }
    }

    #[test]
    fn a_group_never_reset_has_no_cooldown() {
        assert_eq!(cooldown_days_left(None, datetime(2026, 7, 28, 12, 0)), None);
    }

    #[test]
    fn cooldown_counts_down_over_seven_days() {
        let now = datetime(2026, 7, 28, 12, 0);

        let cases = [
            (datetime(2026, 7, 28, 11, 0), Some(7)),
            (datetime(2026, 7, 27, 12, 0), Some(6)),
            (datetime(2026, 7, 24, 12, 0), Some(3)),
            (datetime(2026, 7, 22, 12, 0), Some(1)),
            // Exactly seven days: the cooldown is over.
            (datetime(2026, 7, 21, 12, 0), None),
            (datetime(2026, 7, 20, 12, 0), None),
        ];

        for (reset_at, expected) in cases {
            assert_eq!(
                cooldown_days_left(Some(reset_at), now),
                expected,
                "reset_at {reset_at}"
            );
        }
    }

    #[test]
    fn cooldown_is_one_second_short_of_expiring() {
        // `num_days` truncates, so 6 days 23:59:59 still reports 1 day left.
        let now = datetime(2026, 7, 28, 11, 59) + chrono::Duration::seconds(59);
        let reset_at = datetime(2026, 7, 21, 12, 0);

        assert_eq!(cooldown_days_left(Some(reset_at), now), Some(1));
    }
}
