# 01 — Bug, Sicurezza e Portabilità (`v6`)

> **Riferimento codice**: `HEAD 6ecd492` (21/10/2026). Ogni rilievo aveva: dove, scenario (stato → risultato sbagliato), evidenza, correzione.
> I rilievi `v5` sono chiusi (vedi [`CHANGELOG.md`](../../app/CHANGELOG.md), voce «Rilievi del report v5») e non sono ripetuti.

---

## 1. Rilievi aperti

**Nessuno.** I 49 rilievi `BUG/SEC/PORT/UX/TEST-V6-xx` sono stati corretti il 21/10/2026, ognuno con il suo commit, e tolti da questo documento man mano che entravano in `main`:

| Ramo | Rilievi chiusi |
|---|---|
| `fix/v6-ui-tests` | `UX-V6-01..03`, `TEST-V6-02`, `TEST-V6-03` |
| `fix/v6-versions-setup` | `BUG-V6-12`, `BUG-V6-20`, `BUG-V6-21`, `BUG-V6-24`, `PORT-V6-03`, `PORT-V6-04` |
| `fix/v6-agents-sync` | `BUG-V6-03`, `BUG-V6-16`, `BUG-V6-18`, `BUG-V6-19`, `BUG-V6-30`, `PORT-V6-02`, `SEC-V6-02` (fiducia) |
| `fix/v6-agents-env` | `BUG-V6-15`, `BUG-V6-17`, `BUG-V6-29`, `BUG-V6-32`, `PORT-V6-06`, `SEC-V6-02` (token) |
| `fix/v6-install-net` | `BUG-V6-02`, `BUG-V6-13`, `BUG-V6-14`, `BUG-V6-22`, `BUG-V6-23`, `SEC-V6-03`, `SEC-V6-04`, `PORT-V6-05` |
| `fix/v6-projects-editor` | `BUG-V6-08`, `BUG-V6-10`, `BUG-V6-11`, `BUG-V6-28`, `TEST-V6-01`, parti di `BUG-V6-25`, `BUG-V6-31`, `BUG-V6-33`, `PORT-V6-01` |
| `fix/v6-db-data` | `BUG-V6-01`, `BUG-V6-04..07`, `BUG-V6-09`, `BUG-V6-26`, `BUG-V6-27`, parti di `SEC-V6-01` e `BUG-V6-25` |
| `fix/v6-residues` | le ultime parti di `PORT-V6-01`, `SEC-V6-01`, `BUG-V6-25`, `BUG-V6-31`, `BUG-V6-33` |

Il testo completo dei rilievi è nel commit `4cfc6aa` (`git show 4cfc6aa:docs/reports/v6/01-BUG-SICUREZZA-PORTABILITA.md`).

### 1.1 Da verificare a mano

- **`PORT-V6-01`**: la prova con **due account Windows sullo stesso SSD NTFS** non si può automatizzare (la prova del codice non ricrea un secondo SID). Va fatta e annotata in [`TESTING.md`](../../app/TESTING.md).
- **`BUG-V6-01`, `BUG-V6-04`**: il giro completo dump → ripristino con una vista di `root` e un BLOB non UTF-8 è in `real_databases_start_and_stop`, che gira solo con `ATOLLO_TEST_KIT` impostato.
- **Codice solo Unix** (`editor/pty.rs` di `BUG-V6-25`, la chiave di sigillo di `PORT-V6-06`, il wrapper `runtime/bin/claude` di `SEC-V6-02`): compilato e provato solo dalla CI di macOS e Linux.

---

## 2. Punti controllati senza rilievi

- **Estrazione degli archivi**: `safe_join`, `no_link_on_the_way`, ordine dei collegamenti, sostituzioni con rollback (`replace_dir`, `stage_with`, `clean_leftovers`). Nessuna zip-slip.
- **Checksum**: niente si installa senza SHA fisso, hash del produttore, integrità npm o SHA-512/SHA3; `vc_redist` ha solo la firma Authenticode, ora confrontata con l'editore esatto. Pacchetti conda conservati ricontrollati. Aggiornamento automatico: digest GitHub verificato al download e allo scambio.
- **Lock delle installazioni**: mutex con nome su Windows, `flock` su Unix; cartelle di lavoro per id del kit.
- **Intervalli di versione**: caret, tilde, `~=`, trattino, jolly e `||` corretti secondo npm e PEP 440.
- **Porte**: registro comune coerente (pool 8081-8999, progetti 7100-7999, editor 7000-7099, DB).
- **Comandi IPC**: tutti i 123 chiamano `ensure_main`; argomenti dalla UI validati; nessun segreto nei log.
- **Runtime PHP**: `php-router.php`, `fcgi-guard.php`, phpMyAdmin; ascolti su 127.0.0.1 o 127.x casuale salvo LAN permessa.
