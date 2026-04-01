import express from "express";

const app = express();
const port = Number(process.env.PORT ?? 3000);

app.get("/healthz", (_req, res) => {
  res.json({ status: "ok" });
});

app.listen(port, () => {
  console.log(`web-service listening on ${port}`);
});
