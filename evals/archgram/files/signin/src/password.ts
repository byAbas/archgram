import bcrypt from "bcryptjs";

export function matches(password: string, hash: string): Promise<boolean> {
  return bcrypt.compare(password, hash);
}
