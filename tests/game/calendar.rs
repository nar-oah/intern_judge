use super::*;

#[test]
fn calendar_counts_from_saturday_and_clamps_remaining_days() {
    let mut calendar = GameCalendar {
        total_days: 28,
        elapsed_days: 0,
    };
    for elapsed_days in 0..28 {
        calendar.elapsed_days = elapsed_days;
        let weekday = [6, 7, 1, 2, 3, 4, 5][(elapsed_days % 7) as usize];
        assert_eq!(calendar.weekday(), weekday);
        assert_eq!(calendar.remaining_days(), 28 - elapsed_days);
        assert_eq!(calendar.is_weekend(), matches!(weekday, 6 | 7));
        assert!(!calendar.is_finished());
    }

    for elapsed_days in [28, 29, u32::MAX] {
        calendar.elapsed_days = elapsed_days;
        assert_eq!(calendar.remaining_days(), 0);
        assert!(calendar.is_finished());
        assert!((1..=7).contains(&calendar.weekday()));
    }
}
