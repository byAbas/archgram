import { Router } from "express";
import { verifySession } from "../auth.js";
import { findUser } from "../db.js";
import { matches } from "../password.js";
import { recordSignIn } from "../audit.js";

export const router = Router();

router.post("/login", async (req, res) => {
  const claims = await verifySession(req.headers.authorization ?? "");
  const user = await findUser(req.body.email);
  if (user && (await matches(req.body.password, user.hash))) {
    recordSignIn(user.id);
    return res.json({ session: claims.session });
  }
  return res.status(401).end();
});
