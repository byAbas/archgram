import express from "express";
import { router } from "./routes/login.js";

const app = express();
app.use(express.json());
app.use(router);
app.listen(3000);
