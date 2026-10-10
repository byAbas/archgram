import { Redis } from "ioredis";

const redis = new Redis();

// Fire and forget: the sign-in does not wait for the audit record.
export function recordSignIn(userId: string) {
  void redis.xadd("audit", "*", "event", "signed-in", "user", userId);
}
