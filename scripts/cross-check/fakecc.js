// Fake C compiler for cargo check cross builds: creates empty outputs.
const fs = require('fs');
const a = process.argv.slice(2);
const i = a.indexOf('-o');
if (i >= 0) fs.writeFileSync(a[i + 1], '');
for (const x of a) if (x.startsWith('-o') && x.length > 2) fs.writeFileSync(x.slice(2), '');
if (a.includes('-E')) process.stdout.write('clang\nRUST_VERSION_NEW_OPENSSL_3_0_2\n');
