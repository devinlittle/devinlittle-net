import { expose } from "comlink";
import sqlite3InitModule from "@sqlite.org/sqlite-wasm";

let db: any;

const init = async () => {
  const sqlite3 = await sqlite3InitModule();

  if (sqlite3.oo1.OpfsDb) {
    try {
      db = new sqlite3.oo1.OpfsDb("/devinlittlenet.db");
      console.log("Opened in-browser database");
    } catch (e) {
      console.warn("OpfsDb failed; falling back: ", e)
      db = new sqlite3.oo1.DB("/devinlittlenet.db", "ct");
    }
  } else {
    console.warn("OPFS unavailable; using in-memory fallback")
    db = new sqlite3.oo1.DB("/devinlittlenet.db", "ct");
  }

};

const api = {
  init,
  exec: (sql: string, params: any[] = []) => {
    const bind = params && params.length ? Array.from(params) : undefined;
    return db.exec({
      sql,
      bind,
      returnValue: "resultRows",
      rowMode: "object",
    });
  },
  run: (sql: string, params: any[] = []) => {
    const bind = params && params.length ? Array.from(params) : undefined;
    db.exec({ sql, bind });
  },
};

expose(api);
