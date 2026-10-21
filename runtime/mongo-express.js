'use strict';
// Starter and gate of mongo-express on the kit's MongoDB (DECISIONS.md
// D-147), started by Atollo from the Databases page with the kit's Node.js.
// Written by the app to data\kit\mongo-express.js: a hand edit is replaced
// at the next start.
//
// mongo-express's own server (app.js) is not used: its only gates are a
// fixed user and password (basic auth) or OpenID Connect. This file loads
// its middleware (lib/middleware.js) and puts in front of it the same gate
// as phpMyAdmin's router (runtime/phpmyadmin.php):
//
// - only whoever has this start's token gets in: the app opens
//   `/?atollo=<token>`, which leaves a cookie (HttpOnly, SameSite=Strict)
//   and goes back to `/` without it; any other request without the cookie
//   gets 403;
// - only requests addressed to this server's own loopback address and port
//   (no DNS rebinding); the app picks an address other than 127.0.0.1,
//   where the projects' sites run, because browsers send a host's cookies
//   to all its ports;
// - the password of MongoDB's `admin` and the token come from this
//   process's environment, which they leave before mongo-express is
//   loaded: never on the command line, in a file, the address or the
//   browser. mongo-express connects with them as options of the driver,
//   not inside the connection string it may print;
// - mongo-express's own settings from the environment (ME_CONFIG_*) and a
//   `.env` of the working folder (an empty folder of the app) do not count:
//   basic auth and OpenID off, admin view on, CSRF protection on
//   (NODE_ENV=production; `test` would turn it off).
//
// Environment: ATOLLO_ME_ROOT (mongo-express's folder), ATOLLO_ME_HOST,
// ATOLLO_ME_PORT, ATOLLO_ME_TOKEN, ATOLLO_DB_PORT, ATOLLO_DB_PASSWORD.

const crypto = require('node:crypto');
const http = require('node:http');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

const COOKIE = 'atollo_me';

/** `a` and `b` equal, in a time that does not tell how much of them is. */
function same(a, b) {
    const x = crypto.createHash('sha256').update(String(a)).digest();
    const y = crypto.createHash('sha256').update(String(b)).digest();
    return crypto.timingSafeEqual(x, y);
}

/** The value of cookie `name` in a `Cookie` header. */
function cookie(header, name) {
    for (const part of String(header || '').split(';')) {
        const i = part.indexOf('=');
        if (i > 0 && part.slice(0, i).trim() === name) return part.slice(i + 1).trim();
    }
    return '';
}

/**
 * What the gate does with a request (`url`, `headers`) for the server at
 * `self` (`host:port`) and this start's `token`: `enter` (the token in the
 * address: the cookie, then `/`), `pass` (the cookie: mongo-express
 * answers) or `deny` (403).
 */
function gate(req, self, token) {
    if (!token || !same(String(req.headers.host || '').toLowerCase(), self)) return 'deny';
    let given = null;
    try {
        given = new URL(req.url || '/', 'http://gate').searchParams.get('atollo');
    } catch {
        return 'deny';
    }
    if (given !== null && same(given, token)) return 'enter';
    return same(cookie(req.headers.cookie, COOKIE), token) ? 'pass' : 'deny';
}

async function main() {
    const env = process.env;
    const root = env.ATOLLO_ME_ROOT || '';
    const host = (env.ATOLLO_ME_HOST || '').toLowerCase();
    const port = Number(env.ATOLLO_ME_PORT);
    const token = env.ATOLLO_ME_TOKEN || '';
    const dbPort = Number(env.ATOLLO_DB_PORT);
    const password = env.ATOLLO_DB_PASSWORD || '';
    for (const name of Object.keys(env)) {
        if (/^(ATOLLO_ME_TOKEN|ATOLLO_DB_PASSWORD|ME_CONFIG_.*|VCAP_.*|PORT)$/i.test(name)) delete env[name];
    }
    env.NODE_ENV = 'production';
    if (!root || !host || !Number.isInteger(port) || port < 1 || token.length < 32
        || !Number.isInteger(dbPort) || dbPort < 1 || !password) {
        console.error('mongo-express: start it from Atollo, Databases page.');
        process.exit(2);
    }
    const self = `${host}:${port}`;

    const base = pathToFileURL(root.endsWith(path.sep) ? root : root + path.sep);
    const pkg = require(path.join(root, 'package.json'));
    const { default: config } = await import(new URL('config.default.js', base).href);
    const { default: middleware } = await import(new URL('lib/middleware.js', base).href);
    config.mongodb = {
        ...config.mongodb,
        connectionString: `mongodb://127.0.0.1:${dbPort}/?authSource=admin&directConnection=true`,
        connectionOptions: {
            ...config.mongodb.connectionOptions,
            tls: false,
            auth: { username: 'admin', password },
            authSource: 'admin',
            serverSelectionTimeoutMS: 10000,
        },
        admin: true,
    };
    config.site = {
        ...config.site,
        baseUrl: '/',
        host,
        port,
        sslEnabled: false,
        cookieSecret: crypto.randomBytes(32).toString('hex'),
        sessionSecret: crypto.randomBytes(32).toString('hex'),
    };
    config.useBasicAuth = false;
    config.useOidcAuth = false;
    config.options = { ...config.options, console: false };
    const app = await middleware(config);
    const server = http.createServer((req, res) => {
        const verdict = gate(req, self, token);
        if (verdict === 'enter') {
            res.writeHead(302, {
                'Set-Cookie': `${COOKIE}=${token}; Path=/; HttpOnly; SameSite=Strict`,
                // The token leaves the address.
                Location: '/',
                'Cache-Control': 'no-store',
            });
            res.end();
        } else if (verdict === 'pass') {
            app(req, res);
        } else {
            res.writeHead(403, { 'Content-Type': 'text/plain; charset=utf-8' });
            res.end('mongo-express: open it from Atollo, Databases page.\n');
        }
    });
    server.on('error', (e) => {
        console.error(`mongo-express: ${e.message}`);
        process.exit(1);
    });
    server.listen(port, host, () => {
        console.log(`mongo-express ${pkg.version}: listening on ${self}`);
    });
}

if (require.main === module) {
    main().catch((e) => {
        console.error(`mongo-express: ${e && e.stack ? e.stack : e}`);
        process.exit(1);
    });
}

module.exports = { gate, cookie };
