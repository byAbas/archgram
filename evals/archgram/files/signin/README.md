# signin

The sign-in service. The browser posts an email and a password; the API
verifies the session token with the auth service, looks the user up in
Postgres and checks the password, then answers with a session or a 401.
Each successful sign-in is written to the audit stream in Redis.
