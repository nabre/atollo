// Fake pkg-config: every module exists, at a high version, with no flags.
const a = process.argv.slice(2);
if (a.includes('--modversion')) process.stdout.write('99.0.0\n');
else if (a.includes('--variable=prefix') || a.some((x) => x.startsWith('--variable'))) process.stdout.write('/usr\n');
