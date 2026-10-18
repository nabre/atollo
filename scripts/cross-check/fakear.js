const fs = require('fs');
const a = process.argv.slice(2);
const lib = a.find((x) => x.endsWith('.a') || x.endsWith('.lib'));
if (lib) fs.writeFileSync(lib, '!<arch>\n');
