'use strict';
// From nabre/dev-kit (setup/node/loopback.js, MIT). Written by Dev-Kit.exe to
// data\kit\loopback.js and loaded into every Node.js of the kit
// (NODE_OPTIONS=--require, see tauri/src/env.rs). A server that listens on all
// the network cards (no address, 0.0.0.0 or ::, the default of most dev
// servers) listens on this PC only (127.0.0.1) instead: Windows Firewall
// (and the macOS one, if on) asks to allow a program only when it listens on
// the network, and in a portable kit that question should never come up.
// Pipes, sockets and explicit addresses (192.168.1.5, localhost) are left as
// they are. DEVKIT_ALLOW_LAN=1 (Settings > Local network): no change (a server to open
// from a phone on the same network; then the firewall asks).

const net = require('net');

if (process.env.DEVKIT_ALLOW_LAN !== '1' && !net.Server.prototype.listen.devkitLoopback) {
    const ANY = new Set(['', '0.0.0.0', '::', '[::]', '::0']);
    const loopback = host => host === undefined || host === null || ANY.has(host);
    const original = net.Server.prototype.listen;

    // listen(options, cb) or listen(port, [host], [backlog], [cb]); a pipe
    // (a path or \\.\pipe\...) and a handle have no port and stay as they are
    const listen = function (...args) {
        if (!args.length) args = [0];
        const first = args[0];
        if (first && typeof first === 'object' && !Array.isArray(first)) {
            if (first.port !== undefined && first.path === undefined && first.handle === undefined
                && first._handle === undefined && first.fd === undefined && loopback(first.host)) {
                args[0] = { ...first, host: '127.0.0.1' };
            }
        } else if (typeof first === 'number' || (typeof first === 'string' && /^\d+$/.test(first)) || first === undefined || first === null) {
            if (typeof args[1] === 'string' && loopback(args[1])) args[1] = '127.0.0.1';
            else if (typeof args[1] !== 'string') args.splice(1, 0, '127.0.0.1');
        }
        return original.apply(this, args);
    };
    listen.devkitLoopback = true;
    net.Server.prototype.listen = listen;
}
