// The auth service is run by another team; this is its client.
const AUTH_URL = process.env.AUTH_URL ?? "https://auth.internal";

export async function verifySession(token: string): Promise<{ session: string }> {
  const response = await fetch(`${AUTH_URL}/sessions/verify`, {
    method: "POST",
    headers: { authorization: token },
  });
  return response.json();
}
