<?php
// phpMyAdmin's config.inc.php on the kit's MySQL (DECISIONS.md D-107),
// copied into phpMyAdmin's folder at every start by NEXKORA: a hand edit is
// replaced. Everything comes from the environment of the server the app
// starts (`db/phpmyadmin.rs`); outside it nothing is set and no one logs in.

if (getenv('DEVKIT_PMA_TOKEN') === false) {
    exit;
}

// Root's login (the router lets in only the app's token): the password
// stays in the process's environment.
$cfg['Servers'][1]['auth_type'] = 'config';
$cfg['Servers'][1]['host'] = '127.0.0.1';
$cfg['Servers'][1]['port'] = (string) getenv('DEVKIT_DB_PORT');
$cfg['Servers'][1]['user'] = 'root';
$cfg['Servers'][1]['password'] = (string) getenv('DEVKIT_DB_PASSWORD');
$cfg['Servers'][1]['AllowNoPassword'] = false;
$cfg['AllowArbitraryServer'] = false;

// A new key at every start, 32 bytes as phpMyAdmin wants.
$cfg['blowfish_secret'] = sodium_hex2bin((string) getenv('DEVKIT_PMA_SECRET'));

$cfg['TempDir'] = (string) getenv('DEVKIT_PMA_TMP');
$cfg['DefaultLang'] = (string) (getenv('DEVKIT_LANG') ?: 'en');

// Nothing leaves the PC; no notices about what the kit does not use.
$cfg['VersionCheck'] = false;
$cfg['SendErrorReports'] = 'never';
$cfg['ShowPhpInfo'] = false;
$cfg['PmaNoRelation_DisableWarning'] = true;
$cfg['ZeroConf'] = false;
