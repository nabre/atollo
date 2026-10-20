// The interface's dev server for `npm run dev` (tauri dev, beforeDevCommand
// in tauri/tauri.conf.json): Vite in this very process, so closing the app
// ends it too.
//
// Tauri starts the command through a shell (cmd on Windows) and, when the
// app closes, stops that shell only: a Vite started with `npm run dev`
// under it (npm → cmd → node) stayed alive and kept 127.0.0.1:5173, and the
// next `npm run dev` failed with "Port 5173 is already in use". Here:
// - Vite runs in this process, started with its API;
// - the process ends when the shell that started it is gone (polled every
//   second; on macOS and Linux the parent becomes another process), when
//   the stdin pipe Tauri hands down closes, or on SIGINT/SIGTERM/SIGHUP;
// - a port already taken is reported with who has it and how to stop it,
//   never stopped from here.
//
//   node ../scripts/dev-ui.mjs        (cwd: ui, as Tauri runs it)
import { execFileSync } from "node:child_process";
import { createServer as createNetServer, Socket } from "node:net";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const ui = join(repo, "ui");
const HOST = "127.0.0.1";
const PORT = 5173;

/** Whether nothing listens on the port. */
function portFree() {
  return new Promise((done) => {
    const probe = createNetServer()
      .once("error", () => done(false))
      .once("listening", () => probe.close(() => done(true)))
      .listen(PORT, HOST);
  });
}

/** The processes listening on the port: pid and command line. */
function listeners() {
  try {
    if (process.platform === "win32") {
      const script =
        `Get-NetTCPConnection -LocalPort ${PORT} -State Listen -ErrorAction SilentlyContinue | ` +
        `ForEach-Object { $p = Get-CimInstance Win32_Process -Filter "ProcessId=$($_.OwningProcess)"; ` +
        `"$($_.OwningProcess)\`t$($p.CommandLine)" }`;
      const out = execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], {
        encoding: "utf8",
      });
      return lines(out);
    }
    const pids = execFileSync("lsof", ["-t", `-iTCP:${PORT}`, "-sTCP:LISTEN"], { encoding: "utf8" })
      .split(/\s+/)
      .filter(Boolean);
    return pids.map((pid) => ({
      pid: Number(pid),
      cmd: execFileSync("ps", ["-o", "command=", "-p", pid], { encoding: "utf8" }).trim(),
    }));
  } catch {
    return [];
  }
}

function lines(out) {
  return out
    .split(/\r?\n/)
    .filter((l) => l.includes("\t"))
    .map((l) => {
      const [pid, ...cmd] = l.split("\t");
      return { pid: Number(pid), cmd: cmd.join("\t") };
    });
}

/** The port taken: who has it and how to free it, then stop. A program
 * still in use is never stopped from here (a running `tauri dev` holds the
 * port too). */
async function checkPort() {
  if (await portFree()) return;
  const who = listeners();
  const list = who.map((p) => `pid ${p.pid}: ${p.cmd}`).join("\n  ") || "unknown";
  const how =
    process.platform === "win32"
      ? who.map((p) => `Stop-Process -Id ${p.pid}`).join("; ")
      : who.map((p) => `kill ${p.pid}`).join("; ");
  console.error(`dev-ui: ${HOST}:${PORT} is in use:\n  ${list}`);
  if (how) console.error(`dev-ui: if it is a dev server left open, stop it with: ${how}`);
  process.exit(1);
}

await checkPort();
const server = await createServer({ root: ui, configFile: join(ui, "vite.config.ts") });
await server.listen();
server.printUrls();

let closing = false;
async function stop(why) {
  if (closing) return;
  closing = true;
  console.log(`dev-ui: ${why}, stopping Vite`);
  await server.close().catch(() => undefined);
  process.exit(0);
}

for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(signal, () => void stop(signal));
const parent = process.ppid;
setInterval(() => {
  let alive = process.ppid === parent;
  if (alive) {
    try {
      process.kill(parent, 0);
    } catch {
      alive = false;
    }
  }
  if (!alive) void stop("the command that started it ended");
}, 1000).unref();
// Windows keeps `process.ppid` after the parent ends, and the pid can go to
// another process meanwhile; Tauri hands its shell a stdin pipe that closes
// when Tauri ends, however it ends (PORT-V5-01). Only a pipe: a terminal,
// a file or no stdin at all (run by hand) ends nothing.
if (process.stdin instanceof Socket && !process.stdin.isTTY) {
  for (const event of ["end", "close"]) process.stdin.on(event, () => void stop("its stdin closed"));
  process.stdin.on("error", () => undefined);
  process.stdin.resume();
}
