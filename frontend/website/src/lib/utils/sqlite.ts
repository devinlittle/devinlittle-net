import { wrap } from "comlink";

let dbWorker: any;

export async function getDb() {
  if (dbWorker) return dbWorker;

  const worker = new Worker(new URL("./sqlite.worker.ts", import.meta.url), {
    type: "module",
  });

  dbWorker = wrap(worker);
  await dbWorker.init();
  return dbWorker;
}
