export type BackupState =
  | { type: "NotRunYet" }
  | { type: "Successful"; timestamp: string }
  | { type: "Failed"; timestamp: string; error: string };
