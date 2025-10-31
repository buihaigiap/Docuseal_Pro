export const REMINDER_DURATIONS = [
  { hours: 4, label: "4 hours" },
  { hours: 8, label: "8 hours" },
  { hours: 12, label: "12 hours" },
  { hours: 24, label: "24 hours" },
  { hours: 48, label: "2 days" },
  { hours: 72, label: "3 days" },
  { hours: 96, label: "4 days" },
  { hours: 120, label: "5 days" },
  { hours: 144, label: "6 days" },
  { hours: 168, label: "7 days" },
  { hours: 192, label: "8 days" },
  { hours: 360, label: "15 days" },
  { hours: 504, label: "21 days" },
  { hours: 720, label: "30 days" },
] as const;

export type ReminderDuration = typeof REMINDER_DURATIONS[number];
