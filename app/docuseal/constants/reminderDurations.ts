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

export const hashId = (value: any) => {
  // Convert value thành string
  const str = value.toString();

  // Tạo hash 32-bit từ value, nhưng combine nhiều bước để dài hơn
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash + char) | 0;
  }

  // Tạo thêm vài bước biến đổi để đủ 32 ký tự hex
  let hex = '';
  for (let i = 0; i < 8; i++) {
    const h = ((hash >> (i * 4)) & 0xF).toString(16);
    hex += h;
  }
  hex += hex; // nhân đôi để đủ 32 ký tự
  hex = hex.toUpperCase(); // chữ hoa

  // format UUID
  return (
    hex.substring(0, 8) + '-' +
    hex.substring(8, 12) + '-' +
    hex.substring(12, 16) + '-' +
    hex.substring(16, 20) + '-' +
    hex.substring(20, 32)
  );
};
