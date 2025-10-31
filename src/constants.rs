// Reminder duration constants (in hours)
pub const REMINDER_DURATIONS: &[(i32, &str)] = &[
    (4, "4 hours"),
    (8, "8 hours"),
    (12, "12 hours"),
    (24, "24 hours"),
    (48, "2 days"),
    (72, "3 days"),
    (96, "4 days"),
    (120, "5 days"),
    (144, "6 days"),
    (168, "7 days"),
    (192, "8 days"),
    (360, "15 days"),
    (504, "21 days"),
    (720, "30 days"),
];

/// Check if a given hour value is a valid reminder duration
pub fn is_valid_reminder_duration(hours: i32) -> bool {
    REMINDER_DURATIONS.iter().any(|(h, _)| *h == hours)
}

/// Get human-readable label for hours
pub fn get_duration_label(hours: i32) -> Option<&'static str> {
    REMINDER_DURATIONS.iter()
        .find(|(h, _)| *h == hours)
        .map(|(_, label)| *label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_durations() {
        assert!(is_valid_reminder_duration(4));
        assert!(is_valid_reminder_duration(24));
        assert!(is_valid_reminder_duration(168));
        assert!(is_valid_reminder_duration(720));
        assert!(!is_valid_reminder_duration(1));
        assert!(!is_valid_reminder_duration(999));
    }

    #[test]
    fn test_duration_labels() {
        assert_eq!(get_duration_label(4), Some("4 hours"));
        assert_eq!(get_duration_label(24), Some("24 hours"));
        assert_eq!(get_duration_label(168), Some("7 days"));
        assert_eq!(get_duration_label(720), Some("30 days"));
        assert_eq!(get_duration_label(1), None);
        assert_eq!(get_duration_label(999), None);
    }
}
