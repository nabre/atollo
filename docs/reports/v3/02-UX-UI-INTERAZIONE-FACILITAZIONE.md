# Atollo — Relazione Tecnica di Valutazione `v3`
## Documento 02 · UX / UI, Ergonomia Cognitiva, Accessibilità e Facilitazione Utente

**Punteggio Dimensione D4 (UX / UI, Ergonomia e Accessibilità)**: **91 / 100** *(v1: 86/100 · v2: 86/100 · Target v3.1: 96/100)*  
**Punteggio Dimensione D5 (Facilitazione Utente e Onboarding)**: **90 / 100** *(v1: 81/100 · v2: 81/100 · Target v3.1: 95/100)*  

---

## 1. Verifica degli Interventi UX / UI ed Accessibilità in `HEAD 39b74bc`

Lo Sciame (commit `fd1e6dc`, `b3d4a96`, `b83e9eb` in PR #1) ha risolto le principali frizioni d'interazione e accessibilità segnalate nei report `v1` e `v2`:

| Area UX / A11y | Implementazione Verificata a `HEAD 39b74bc` | File Coinvolti | Beneficio Misurato |
|---|---|---|---|
| **Code-Splitting Lazy (`ARCH-LAZY`)** | [`GuidePage.tsx`](../../../src/pages/GuidePage.tsx) (~1.375 righe) è ora importata dinamicamente tramite `React.lazy(() => import("./pages/GuidePage"))` avvolta in `<Suspense>` dentro [`src/App.tsx`](../../../src/App.tsx). | [`src/App.tsx`](../../../src/App.tsx) | Il bundle iniziale non carica i 1.375+ nodi della guida finché l'utente non apre la scheda **Guida**, migliorando il Time-to-Interactive all'avvio. |
| **Navigazione da Tastiera Sidebar (`BUG-11r`)** | La barra laterale implementa `onKeyDown` completo per `ArrowDown`, `ArrowUp`, `Home` e `End`, spostando il focus e la selezione ciclicamente tra le 8 voci di navigazione. | [`src/components/Sidebar.tsx`](../../../src/components/Sidebar.tsx) | Conformità al pattern WAI-ARIA Navigation / Tabs senza necessità di usare il mouse. |
| **Focus Trap & Restore in `ConfirmModal` (`BUG-20r`)** | `ConfirmModal` cattura l'elemento attivo prima dell'apertura (`previousActiveElement`), intrappola il ciclo `Tab` / `Shift+Tab` tra gli elementi focalizzabili del dialogo, chiude con `Escape` e ripristina il focus all'elemento chiamante alla chiusura. | [`src/components/ConfirmModal.tsx`](../../../src/components/ConfirmModal.tsx) | Eliminata la fuga del focus sullo sfondo durante le conferme distruttive. |
| **Error Recovery Selettivo in `SetupModal` (`UI-POLISH`)** | Quando uno o più componenti falliscono durante un'installazione multipla, `SetupModal` calcola `failedIds` e mostra il pulsante primario **«Riprova»** (`onRetry(failedIds)`), oltre a una barra di progresso determinata basata sui byte/step completati. | [`src/components/SetupModal.tsx`](../../../src/components/SetupModal.tsx) | L'utente non deve più riselezionare manualmente i componenti falliti dopo un timeout di rete. |
| **Integrazione Visuale Pool Web (`GAP-RUN`)** | Quando un progetto con sito Apache o Nginx è associato a un pool condiviso (`ServersPage`), cliccare **«Avvia»** nella scheda Progetti avvia/ricarica automaticamente il pool e mostra lo stato `Running` collegato al pool con URL `.localhost:<port>`. | [`tauri/src/lib.rs`](../../../tauri/src/lib.rs), [`tauri/src/projects/run.rs`](../../../tauri/src/projects/run.rs#L676-L734) | Coerenza totale tra la vista **Progetti** e la vista **Server Web**. |
| **Filtro Componenti per OS/Arch (`BUG-24a`)** | [`setup/catalog.rs`](../../../tauri/src/setup/catalog.rs#L112-L120) espone `available_for(kit.target())` nella vista di setup: i componenti incompatibili con l'OS/Arch corrente non vengono proposti come installabili per errore. | [`tauri/src/setup/mod.rs`](../../../tauri/src/setup/mod.rs), [`tauri/src/setup/catalog.rs`](../../../tauri/src/setup/catalog.rs) | Zero errori criptici di download su componenti non supportati dal target. |

---

## 2. Frizioni UX / UI Residue e Soluzioni Concrete Promosse in `v3`

Per portare le dimensioni **D4 (UX/UI)** da `91` a `96/100` e **D5 (Facilitazione)** da `90` a `95/100` in un contesto **100% Portable & Multiplatform**, promuoviamo **4 miglioramenti concreti**:

### 2.1 Banner e Badge di Diagnostica Distribuzione OS e Portabilità (`StatusPage` & `ToolsPage`)
Su Linux e su macOS, l'usabilità di alcuni strumenti dipende dalle caratteristiche della distribuzione ospite (es. versione di `glibc` vs `musl` su Linux; presenza del loader FHS `/lib64/ld-linux` su NixOS; presenza di Xcode Command Line Tools su macOS per compilare crate Rust/C nativi).
- **Problema attuale**: L'interfaccia mostra solo `target` (`win32-x64`, `linux-x64`, `darwin-arm64`), senza indicare:
  1. La distribuzione OS rilevata (`Ubuntu 24.04 · glibc 2.39`, `Alpine 3.21 · musl`, `NixOS`, `Windows 11 24H2 x64`, `macOS 15.2 · CLT presenti`).
  2. La modalità di portabilità attiva (`100% Kit-Contained` vs `Temp su Host D-90`).
  3. Eventuali componenti in stato `.reinstall` dopo uno spostamento (finché non viene applicata la rilocazione offline del [`Documento 04 §2`](../../../reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md)).
- **Soluzione Concreta Promossa**:
  Aggiungere nel comando Tauri `status_overview` (in [`tauri/src/status.rs`](../../../tauri/src/status.rs)) una struct `HostDistroInfo`:

```rust
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostDistroInfo {
    /// E.g. "Windows 11 24H2", "Ubuntu 24.04.1 LTS", "Alpine Linux v3.21", "macOS 15.3"
    pub distro_name: String,
    /// E.g. "UCRT", "glibc 2.39", "musl 1.2.5", "libSystem (Darwin)"
    pub libc_kind: String,
    /// True if FHS dynamic loader (/lib64/ld-linux* or /lib/ld-linux*) or kit sysroot is present.
    pub fhs_loader_ok: bool,
    /// True if 100% of installed components are relocatable offline without re-download.
    pub portable_100_ok: bool,
}
```

Nella pagina [`StatusPage.tsx`](../../../src/pages/StatusPage.tsx), mostrare una pillola informativa **"Salute Portabilità & OS"** con semaforo verde (`100% Portatile · 36/36 compatibili`) o avviso contestuale azionabile (es. su macOS senza Xcode CLT: *"Installa C/C++ del kit per compilare crate Rust nativi"*).

### 2.2 Azione Rapida «Ripara / Riloca Ambienti Offline» senza Re-download
- **Problema attuale**: Se un componente ha un file `<id>.reinstall` (generato da `kit.rs:relocate_non_relocatable()`), la UI lo mostra semplicemente come "Non installato", costringendo l'utente a riscaricarlo da zero e lasciando i file nella cartella su disco finché non parte il download.
- **Soluzione Concreta Promossa**:
  Con la funzione `relocate_conda_prefix` descritta nel [`Documento 04 §2`](../../../reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md), la rilocazione avviene **automaticamente e silenziosamente all'avvio in meno di 40 ms** senza toccare la rete e senza mai creare `<id>.reinstall`. Inoltre, nella pagina **Strumenti** ([`ToolsPage.tsx`](../../../src/pages/ToolsPage.tsx)), aggiungiamo nel menu contestuale di ogni strumento l'azione **«Verifica integrità e percorsi portatili»**, che controlla la presenza del binario chiave (`has_key`), ripara gli script wrapper (`composer`, `make`, `R`, `git`) e rigenera le junction/symlink di versione (`versions::sync_with_disk`).

### 2.3 Estensione del Focus Trap a Tutte le Modali (`SetupModal`, `ProjectModal`, `BackupModal`)
- **Problema attuale**: Lo Sciame ha introdotto il focus trap completo (`Tab` / `Shift+Tab` + restore del focus) in [`ConfirmModal.tsx`](../../../src/components/ConfirmModal.tsx), ma [`SetupModal.tsx`](../../../src/components/SetupModal.tsx), [`ProjectModal.tsx`](../../../src/components/ProjectModal.tsx) e [`BackupModal.tsx`](../../../src/components/BackupModal.tsx) gestiscono solo `Escape` senza intrappolare `Tab`.
- **Soluzione Concreta Promossa**: Estrarre la logica di `ConfirmModal.tsx` in un custom hook condiviso `useModalFocusTrap(ref, open, onClose)` in `src/hooks/useModalFocusTrap.ts` (28 righe) e agganciarlo a tutte le 4 modali dell'applicazione.

```tsx
// src/hooks/useModalFocusTrap.ts
import { RefObject, useEffect } from "react";

const FOCUSABLE =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function useModalFocusTrap(
  containerRef: RefObject<HTMLElement | null>,
  open: boolean,
  onEscape?: () => void,
) {
  useEffect(() => {
    if (!open || !containerRef.current) return;
    const prev = document.activeElement as HTMLElement | null;
    const root = containerRef.current;
    const first = root.querySelector<HTMLElement>(FOCUSABLE);
    first?.focus();

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && onEscape) {
        e.preventDefault();
        onEscape();
        return;
      }
      if (e.key !== "Tab") return;
      const nodes = Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE));
      if (nodes.length === 0) return;
      const firstEl = nodes[0];
      const lastEl = nodes[nodes.length - 1];
      if (e.shiftKey && document.activeElement === firstEl) {
        e.preventDefault();
        lastEl.focus();
      } else if (!e.shiftKey && document.activeElement === lastEl) {
        e.preventDefault();
        firstEl.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      prev?.focus?.();
    };
  }, [open, containerRef, onEscape]);
}
```

### 2.4 Terminale Multi-OS nella UI (`Terminal` / Console Launcher)
Su Windows, il pulsante **«Terminale»** avvia Windows Terminal portatile (`wt.exe`) se installato, oppure `conhost.exe` con PowerShell (`pwsh` / `powershell.exe`). Su macOS e Linux (dove il componente `terminal` Windows Terminal non esiste), l'azione **«Terminale»** deve aprire il terminale grafico di sistema (`Terminal.app` / `iTerm.app` su macOS; `x-terminal-emulator` / `kgx` / `konsole` / `gnome-terminal` / `xfce4-terminal` / `alacritty` / `kitty` su Linux) iniettando l'ambiente isolato `KitEnv` tramite uno script di bootstrap generato in `data/tmp/atollo-shell.sh`. La soluzione completa lato Rust (`wterm.rs` e `os/jobs.rs`) è dettagliata nel [`Documento 04 §5.5`](../../../reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md).
