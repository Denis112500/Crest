const MILLISECONDS_PER_SECOND = 1000;
const RESET_TIME_FORMAT = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" });
const RESET_DAY_AND_TIME_FORMAT = new Intl.DateTimeFormat(undefined, {
  weekday: "short",
  hour: "numeric",
  minute: "2-digit",
});

// When a reached usage limit resets, in the user's own clock format: "1:50" later today, with the
// day ("Fri 9:00") when it resets on another day (a weekly limit can be days away).
export function formatUsageLimitResetTime(resetsAtUnixSeconds: number): string {
  const resetMoment = new Date(resetsAtUnixSeconds * MILLISECONDS_PER_SECOND);
  const isResetToday = resetMoment.toDateString() === new Date().toDateString();
  return (isResetToday ? RESET_TIME_FORMAT : RESET_DAY_AND_TIME_FORMAT).format(resetMoment);
}
