import pg from "pg";

const pool = new pg.Pool();

export async function findUser(email: string) {
  const { rows } = await pool.query("select id, hash from users where email = $1", [email]);
  return rows[0];
}
