# Guida di Atollo

[English](GUIDE.md) · Italiano

Atollo è un ambiente di sviluppo completo che vive in una sola cartella: su una chiavetta, un disco esterno o il disco del PC. Non si installa e non chiede diritti di amministratore: copi la cartella su un altro PC e ritrovi tutto, progetti compresi.

## Indice

1. [Primi passi](#1-primi-passi)
   - [1.1 Benvenuto in Atollo](#11-benvenuto-in-atollo)
   - [1.2 Il primo avvio](#12-il-primo-avvio)
   - [1.3 Conoscere la finestra](#13-conoscere-la-finestra)
   - [1.4 Il primo progetto, passo per passo](#14-il-primo-progetto-passo-per-passo)
2. [Progetti](#2-progetti)
   - [2.1 Creare un progetto](#21-creare-un-progetto)
   - [2.2 La pagina Progetti e il «Pannello rapido»](#22-la-pagina-progetti-e-il-pannello-rapido)
   - [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte)
   - [2.4 Componenti e versioni di un progetto](#24-componenti-e-versioni-di-un-progetto)
   - [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni)
   - [2.6 Git di un progetto](#26-git-di-un-progetto)
   - [2.7 Eliminare un progetto e i «Dati senza progetto»](#27-eliminare-un-progetto-e-i-dati-senza-progetto)
3. [Database, siti e terminali](#3-database-siti-e-terminali)
   - [3.1 I server di database](#31-i-server-di-database)
   - [3.2 Il database di un progetto](#32-il-database-di-un-progetto)
   - [3.3 Siti web e indirizzi .localhost](#33-siti-web-e-indirizzi-localhost)
   - [3.4 Terminali del kit](#34-terminali-del-kit)
4. [Agenti IA](#4-agenti-ia)
   - [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi)
   - [4.2 Istruzioni, skill e server MCP condivisi](#42-istruzioni-skill-e-server-mcp-condivisi)
   - [4.3 Limiti e consumi degli agenti](#43-limiti-e-consumi-degli-agenti)
   - [4.4 Modelli locali: llama.cpp e i GGUF](#44-modelli-locali-llamacpp-e-i-gguf)
5. [Strumenti e linguaggi](#5-strumenti-e-linguaggi)
   - [5.1 La pagina Strumenti](#51-la-pagina-strumenti)
   - [5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento)
   - [5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti)
6. [Account e impostazioni](#6-account-e-impostazioni)
   - [6.1 L'account GitHub del kit](#61-laccount-github-del-kit)
   - [6.2 Le Impostazioni, una per una](#62-le-impostazioni-una-per-una)
   - [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale)
   - [6.4 Le istruzioni dei framework](#64-le-istruzioni-dei-framework)
7. [Portare il kit con sé](#7-portare-il-kit-con-sé)
   - [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza)
   - [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella)
   - [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza)
8. [Come funziona dentro](#8-come-funziona-dentro)
   - [8.1 Portabile e isolato: che cosa vuol dire](#81-portabile-e-isolato-che-cosa-vuol-dire)
   - [8.2 Le cartelle del kit su disco](#82-le-cartelle-del-kit-su-disco)
   - [8.3 Processi in background](#83-processi-in-background)
   - [8.4 Da dove arrivano gli strumenti](#84-da-dove-arrivano-gli-strumenti)
   - [8.5 La riga di comando](#85-la-riga-di-comando)
9. [Aiuto](#9-aiuto)
   - [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va)
   - [9.2 Domande frequenti](#92-domande-frequenti)
   - [9.3 Glossario](#93-glossario)

## 1. Primi passi

### 1.1 Benvenuto in Atollo

*Che cos'è Atollo, che cosa trovi dentro e come è organizzata questa guida. Il punto da cui partire se è la prima volta.*

> **In breve**
>
> - Atollo è un ambiente di sviluppo completo che vive in una sola cartella: su una chiavetta, un disco esterno o il disco del PC.
> - Non si installa e non chiede diritti di amministratore: copi la cartella su un altro PC e ritrovi tutto, progetti compresi.
> - Dentro trovi linguaggi in più versioni, database, server web, un editor, console, Git e GitHub, quattro agenti IA e i modelli linguistici locali.
> - Questa guida va dalle cose più semplici a quelle più tecniche: puoi leggerla in ordine o saltare dove ti serve.

#### Che cos'è, in parole semplici

Immagina uno zaino da lavoro già pronto: dentro c'è tutto ciò che serve per scrivere programmi e siti, ordinato e funzionante. Lo porti con te, lo apri su qualsiasi computer e lavori, senza installare niente e senza lasciare disordine sul PC che ti ospita. Quando hai finito, lo chiudi e lo porti via.

Atollo è quello zaino. È un unico programma che sta in una cartella insieme a due sottocartelle: projects, dove vivono i tuoi progetti, e data, dove il kit tiene i suoi programmi, le impostazioni e i dati dei database. Funziona su Windows 10 e 11, macOS e Linux, con processori x64 e ARM64 (su ARM64 serve Windows 11).

> [!NOTE]
> **A chi serve**
>
> A chi programma su più computer (casa, ufficio, laboratorio), a chi non può installare programmi sul PC di lavoro, a chi insegna o studia e vuole lo stesso ambiente per tutti, e a chi vuole provare un linguaggio o un framework senza sporcare il proprio sistema.

#### Che cosa trovi dentro

| Che cosa | A che cosa serve | Dove si trova nell'app |
| --- | --- | --- |
| Progetti | Le tue cartelle di lavoro, ognuna con le sue azioni, versioni, database e sito. | Progetti |
| Linguaggi e strumenti | Python, Node.js, PHP, Java, .NET, Go, Rust e molti altri, anche più versioni insieme. | Strumenti |
| Database | Un server MySQL e uno MongoDB, solo per questo PC, con le password già pronte. | Database |
| Server web | Caddy e Apache per vedere i tuoi siti su indirizzi come mio-sito.localhost. | Server Web |
| Editor | VSCodium, la versione libera di Visual Studio Code, in una finestra di Atollo. | Progetti › Editor |
| Console | PowerShell 7 con tutti gli strumenti del kit già pronti. | Terminali |
| Git e GitHub | Git e GitHub CLI con un solo accesso al tuo account per tutto il kit. | Impostazioni › Account GitHub |
| Agenti IA | Claude Code, Codex, Antigravity e Pi, con istruzioni, skill e server MCP condivisi. | Agenti |
| Modelli locali | Modelli linguistici sul PC, senza rete e senza account, per Pi e per una chat nel browser. | Modelli locali |

#### Tre idee da tenere a mente

**Tutto in una cartella** — Il kit scrive solo dentro la sua cartella (con poche eccezioni dichiarate). Per fare una copia di tutto basta copiare la cartella; per toglierlo basta cancellarla.

**Il kit pensa agli strumenti** — Quando un progetto o un agente ha bisogno di un programma che manca, il kit lo scarica dalla fonte ufficiale, lo verifica e lo mette al suo posto. Tu scegli, lui installa.

**Solo su questo PC** — Database e siti rispondono solo al computer su cui stai lavorando (indirizzo 127.0.0.1), con password generate dal kit. Nessuno dalla rete li vede, a meno che tu non lo chieda.

#### Come è fatta questa guida

La guida ha nove capitoli. I primi tre servono a tutti: come partire, come lavorare ai progetti, come usare database e siti. Poi agenti, strumenti e impostazioni. Il capitolo «Come funziona dentro» è per chi vuole capire i dettagli tecnici; l'ultimo raccoglie problemi comuni, domande frequenti e un glossario.

In ogni argomento trovi: un riquadro «In breve» all'inizio, i passaggi da seguire, consigli e avvertenze, e una parte ripiegata «Sotto il cofano» con i dettagli tecnici (file, cartelle, comandi). Il pulsante in alto a destra apre la pagina dell'app di cui l'argomento parla. La casella di ricerca a sinistra cerca nei titoli, nelle descrizioni e nelle parole chiave.

1. **[Primi passi](#1-primi-passi)** — Benvenuto in Atollo · Il primo avvio · Conoscere la finestra · Il primo progetto, passo per passo
2. **[Progetti](#2-progetti)** — Creare un progetto · La pagina Progetti e il «Pannello rapido» · Avviare un progetto: azioni e porte · Componenti e versioni di un progetto · Editor, terminale ed estensioni · Git di un progetto · Eliminare un progetto e i «Dati senza progetto»
3. **[Database, siti e terminali](#3-database-siti-e-terminali)** — I server di database · Il database di un progetto · Siti web e indirizzi .localhost · Terminali del kit
4. **[Agenti IA](#4-agenti-ia)** — Gli agenti: Claude Code, Codex, Antigravity, Pi · Istruzioni, skill e server MCP condivisi · Limiti e consumi degli agenti · Modelli locali: llama.cpp e i GGUF
5. **[Strumenti e linguaggi](#5-strumenti-e-linguaggi)** — La pagina Strumenti · Più versioni dello stesso strumento · Panoramiche: matrice, relazioni, estensioni, requisiti
6. **[Account e impostazioni](#6-account-e-impostazioni)** — L'account GitHub del kit · Le Impostazioni, una per una · Rete aziendale: proxy, certificati, rete locale · Le istruzioni dei framework
7. **[Portare il kit con sé](#7-portare-il-kit-con-sé)** — Rimuovere l'unità in sicurezza · Su un altro PC, un'altra lettera, un'altra cartella · Aggiornamenti e copie di sicurezza
8. **[Come funziona dentro](#8-come-funziona-dentro)** — Portabile e isolato: che cosa vuol dire · Le cartelle del kit su disco · Processi in background · Da dove arrivano gli strumenti · La riga di comando
9. **[Aiuto](#9-aiuto)** — Quando qualcosa non va · Domande frequenti · Glossario

Argomenti collegati: [1.2 Il primo avvio](#12-il-primo-avvio), [1.3 Conoscere la finestra](#13-conoscere-la-finestra), [1.4 Il primo progetto, passo per passo](#14-il-primo-progetto-passo-per-passo)

### 1.2 Il primo avvio

*Dove mettere il kit, la pagina «Benvenuto», quali componenti scegliere e che cosa succede mentre si scaricano.*

> **In breve**
>
> - Metti il programma di Atollo in una cartella vuota, dove vuoi tenere il kit, e aprilo.
> - La prima volta compare la pagina «Benvenuto»: scegli lingua e, se serve, proxy e certificati, poi agenti e linguaggi, e premi «Installa». Prima di «Installa» non si scarica niente.
> - I componenti si scaricano in sottofondo dalle fonti ufficiali: puoi già usare l'app mentre arrivano.
> - Niente è definitivo: tutto si aggiunge o si toglie più tardi dalla pagina Strumenti.

#### 1. Dove mettere il kit

Il kit sta dove sta il suo eseguibile: la cartella che lo contiene diventa la «cartella del kit» e lì nasceranno projects e data. Scegli quindi una cartella vuota e con un nome semplice, per esempio `D:\Atollo` su una chiavetta o un disco esterno, oppure `C:\Atollo` sul disco del PC.

- Su Windows l'eseguibile si chiama Atollo.exe, oppure Atollo-\<versione\>-win32-x64.exe come lo scarichi dalla pagina delle release. Su macOS è Atollo.app, su Linux un file AppImage.
- Spazio: la scelta proposta al primo avvio occupa circa 1,8 GB. Ogni linguaggio o versione in più aggiunge spazio, e i progetti con le loro dipendenze crescono nel tempo: su una chiavetta conviene avere almeno 16 GB liberi.
- Velocità: un SSD esterno è molto più veloce di una chiavetta economica. Il kit funziona su entrambi, ma installare pacchetti con npm o Composer su una chiavetta lenta richiede pazienza.

> [!WARNING]
> **Due cose da evitare nel percorso**
>
> - Su Windows, una cartella del percorso il cui nome comincia con una cifra, con la lettera x o con la lettera u (per esempio `D:\2024\Atollo` o `D:\utenti\Atollo`) impedisce all'editor di accorgersi dei file cambiati. Se succede, l'app lo segnala con un avviso: sposta il kit in un percorso diverso.
> - Su macOS e Linux un disco montato senza permesso di esecuzione (noexec) ferma l'app: il kit deve poter avviare i suoi programmi.

<details>
<summary>Quale file system scegliere</summary>

Su Windows il formato migliore è NTFS: è l'unico che permette al kit di proteggere le cartelle con le password (database, accessi) in modo che le legga solo il tuo utente, e di usare collegamenti al posto delle copie. Su exFAT e FAT32 il kit funziona, ma senza quella protezione; FAT32 inoltre non accetta file più grandi di 4 GB. Se usi la stessa chiavetta su Windows, macOS e Linux, exFAT è il formato che tutti leggono e scrivono: in quel caso tieni presente che i collegamenti diventano copie.

La pagina Impostazioni › «Il kit» mostra il disco su cui si trova il kit: file system e tipo (fisso, rimovibile, di rete).

</details>

#### 2. La pagina «Benvenuto»

Al primo avvio, al posto delle solite pagine, compare la scheda «Benvenuto», con il titolo «Benvenuto in Atollo». Spiega che agenti e linguaggi verranno scaricati dalle fonti ufficiali, verificati e messi nella cartella data del kit, senza installazione e senza diritti di amministratore.

In cima ci sono due gruppi di impostazioni, salvate da sole mentre le cambi (come nella pagina Impostazioni, dove le ritrovi): «Prima di iniziare», con «Lingua», «Tema» e «Aggiornamento automatico», e «Rete», con proxy, certificati aggiuntivi e rete locale. Compila «Rete» solo se la tua rete passa da un proxy o ispeziona il traffico HTTPS: dato che niente si scarica prima di «Installa», anche i primi download li useranno.

Sotto c'è l'elenco dei componenti, diviso in otto tipi (ognuno con la sua descrizione), con una casella di ricerca e un indice a sinistra. Ogni riga ha una casella da spuntare, il logo, il nome, una descrizione, la dimensione e delle etichette che dicono di che cosa ha bisogno o a che cosa serve.

| Tipo | Che cosa contiene | Cosa scegliere |
| --- | --- | --- |
| Sistema ed editor | L'editor, PowerShell 7 (la console del kit), GitHub CLI e Git. Sotto, i requisiti del PC. | Sono sempre installati, insieme agli altri, quando premi «Installa»: le caselle sono bloccate. |
| Agenti AI | Claude Code, Codex, Antigravity e Pi. | Facoltativi. Tieni quelli di cui hai un account; servono un abbonamento o una chiave API del fornitore. Pi non è spuntato: spuntalo se vuoi un agente sui modelli locali, senza account. |
| Modelli locali | llama.cpp, il server dei modelli linguistici sul PC. | Facoltativo e non spuntato. Si installa anche da solo al primo avvio del server, dalla pagina Modelli locali, dove si scaricano i modelli (molti GB). |
| Linguaggi | Tutti i linguaggi e i runtime: Python, Node.js, PHP, Java, Go, .NET, Rust, Ruby, il compilatore C/C++ e gli altri. Python, Node.js, PHP, Go e Java possono avere più versioni; gli altri restano all'ultima. | Quelli che usi. Altri si aggiungono quando servono. |
| Strumenti di sviluppo | uv, pnpm e yarn, CMake e Ninja, Android SDK, Typst, TinyTeX: una sola versione, sempre l'ultima. | Di solito niente: il kit li installa da solo quando un progetto o un agente li chiede. |
| Server web | Apache e Caddy, per i siti dei progetti e i Server Web condivisi. | Quello che usano i tuoi progetti; Apache è già nella scelta proposta. |
| Database | MySQL, MongoDB, SQLite; nel browser phpMyAdmin e mongo-express. | Solo se ti servono. MySQL si installa anche al primo avvio del server. |

La scelta proposta comprende Claude Code, Codex e Antigravity (non Pi né llama.cpp), Python, PHP con Apache e i componenti di «Sistema ed editor», Git compreso. In tutto circa 1,8 GB. Quando spunti un componente, il kit spunta anche quelli di cui ha bisogno (per esempio Kotlin porta con sé Java, phpMyAdmin porta PHP e MySQL).

1. In «Prima di iniziare» scegli la lingua e, se vuoi, il tema; se sei dietro un proxy aziendale, compila anche «Rete».
2. Togli la spunta a ciò che non ti interessa e aggiungi ciò che usi.
3. Premi «Installa» in basso. Non c'è una conferma: la scelta viene salvata e lo scaricamento parte subito.
4. L'app lascia la pagina «Benvenuto» e mostra le pagine normali. Lo scaricamento continua in sottofondo: lo vedi nella barra di stato in basso e nel pannello «Attività».

> [!TIP]
> **Consiglio**
>
> Al primo avvio non si scelgono le versioni: di ogni linguaggio si installa la più recente. Le altre versioni si aggiungono dopo, dalla pagina Strumenti.

> [!WARNING]
> **Fiducia nei progetti**
>
> Quando apri un agente in un progetto da Atollo, la cartella del progetto diventa attendibile per Claude Code e Codex, che possono quindi eseguirvi comandi senza chiedere ogni volta. Apri un agente solo nei progetti di cui ti fidi.

#### 3. Che cosa succede mentre scarica

1. **Scarica** — Dalla fonte ufficiale del produttore, in HTTPS.
2. **Verifica** — Confronta l'impronta del file con quella pubblicata; se non coincide lo scarta.
3. **Estrae** — In una cartella di lavoro temporanea.
4. **Sistema** — Mette il programma in `data\apps` e prova che risponda.

Ogni componente passa da questi quattro passi. Se un download non riesce (rete assente, proxy aziendale, sito del produttore irraggiungibile) il kit riprova tre volte; se ancora non va, la riga mostra l'errore e, quando possibile, un riquadro per scaricare il file con il browser e consegnarlo al kit.

I dettagli su fonti e verifiche sono in [8.4 Da dove arrivano gli strumenti](#84-da-dove-arrivano-gli-strumenti); se sei dietro un proxy aziendale leggi prima [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale).

<details>
<summary>Sotto il cofano</summary>

- La scelta dei componenti viene salvata in `data\app\components.txt`, una riga per componente (id=1 scelto, id=0 no).
- Al primo avvio non si scarica niente, nemmeno i componenti obbligatori, finché non premi «Installa»: così proxy e certificati scelti nella pagina valgono già per i primi download. Dagli avvii successivi il kit controlla che ci siano i componenti obbligatori e li reinstalla da solo se mancano, senza chiedere.
- Se il primo avvio viene interrotto, alla riapertura la pagina «Benvenuto» non ricompare se la scelta è già stata salvata: si riprende da Strumenti, dove le righe non scaricate mostrano lo stato «non esiste» o «errore».

</details>

Argomenti collegati: [1.3 Conoscere la finestra](#13-conoscere-la-finestra), [1.4 Il primo progetto, passo per passo](#14-il-primo-progetto-passo-per-passo), [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale)

### 1.3 Conoscere la finestra

*Barra delle attività, barra laterale, schede, barra di stato e pannello «Attività»: dove sono le cose e a che cosa servono.*

> **In breve**
>
> - La finestra imita quella di Visual Studio Code: chi lo conosce si orienta subito.
> - A sinistra la barra delle attività cambia pagina; accanto, la barra laterale mostra elenchi e alberi.
> - Al centro le schede; in basso la barra di stato e il pannello «Attività» con ciò che è in corso.
> - <kbd>Ctrl+P</kbd> apre un progetto per nome da qualsiasi pagina.

#### Le parti della finestra

```
+-------------------------------------------------------------+
| 1 · Barra del titolo                                        |
+------------+-----------------+------------------------------+
| 2 ·        | 3 · Barra laterale | 4 · Area delle schede        |
| Barra      |                 |                              |
| delle      |                 +------------------------------+
| attività   |                 | 5 · Pannello «Attività»      |
+------------+-----------------+------------------------------+
| 6 · Barra di stato                                          |
+-------------------------------------------------------------+
```

| Parte | Che cosa contiene |
| --- | --- |
| 1 · Barra del titolo | Il nome Atollo e la pagina aperta; a destra riduci, ingrandisci e chiudi. Trascinala per spostare la finestra, doppio clic per ingrandirla. |
| 2 · Barra delle attività | Le icone delle pagine. In alto quelle di lavoro: Progetti, Agenti, Database, Server Web, Terminali, Strumenti. In basso: Guida, Impostazioni, Rimuovi l'unità. Un numero sull'icona di Progetti dice quante cose sono in esecuzione: azioni dei progetti, versioni dei database e pool dei server web. |
| 3 · Barra laterale | Il titolo della pagina con le sue icone (tra cui il libro «Apri la guida di questa pagina»), spesso una casella di ricerca, e dei riquadri che si aprono e si chiudono con un clic sul titolo. |
| 4 · Area delle schede | Ciò che hai scelto nella barra laterale si apre in una scheda. La prima scheda di alcune pagine è fissa: il «Pannello rapido» in Progetti, «Tutti gli strumenti» in Strumenti. |
| 5 · Pannello «Attività» | Si apre dalla barra di stato. Elenca l'installazione in corso, le versioni dei database accese, i pool dei server web accesi, il server dei modelli locali e le azioni dei progetti, ognuna con il suo output; «Ferma tutto» le ferma tutte dopo una conferma. |
| 6 · Barra di stato | A sinistra la voce Atollo e il contatore di ciò che è in esecuzione e degli errori, che mostra o nasconde il pannello «Attività»; a destra i terminali aperti, lo stato dell'editor, il sistema (per esempio win32-x64) e, per ultima, la versione di Atollo: un clic cerca gli aggiornamenti, e accanto compare l'esito (un segno di spunta, la versione nuova o un avviso). |

#### Le pagine

**Progetti** — Il centro del lavoro: l'elenco dei progetti, il «Pannello rapido», la creazione di progetti nuovi e la pagina di ogni progetto. [Come funziona](#22-la-pagina-progetti-e-il-pannello-rapido)

**Agenti** — Gli agenti IA installati: accesso al tuo account, limiti del piano e consumi, cartella condivisa. [Come funziona](#41-gli-agenti-claude-code-codex-antigravity-pi)

**Modelli locali** — Modelli linguistici sul PC per Pi e la chat nel browser: il server llama.cpp, i download dal catalogo e da Hugging Face. [Come funziona](#44-modelli-locali-llamacpp-e-i-gguf)

**Database** — I server MySQL e MongoDB del kit: avvio, password, porte, versioni, phpMyAdmin e mongo-express. [Come funziona](#31-i-server-di-database)

**Server Web** — I server Caddy e Apache condivisi fra i progetti e i siti .localhost che servono. [Come funziona](#33-siti-web-e-indirizzi-localhost)

**Terminali** — Console con gli strumenti del kit, in qualsiasi cartella del PC. [Come funziona](#34-terminali-del-kit)

**Strumenti** — Linguaggi, agenti, database e programmi del kit: installare, aggiornare, togliere, più versioni. [Come funziona](#51-la-pagina-strumenti)

**Guida** — Questa guida. Ogni pagina ha anche un'icona a forma di libro che apre l'argomento giusto. [Come funziona](#11-benvenuto-in-atollo)

**Impostazioni** — Lingua, tema, rete, account GitHub e chi firma i commit, chiave di Gemini, aggiornamenti, privacy e istruzioni dei framework, salvati da soli. [Come funziona](#62-le-impostazioni-una-per-una)

**Rimuovi l'unità** — Chiude tutto ciò che il kit ha aperto, prima di staccare la chiavetta. [Come funziona](#71-rimuovere-lunità-in-sicurezza)

#### La barra di stato e il pannello «Attività»

| Elemento | Che cosa dice | Che cosa fa al clic |
| --- | --- | --- |
| Atollo (a sinistra) | Passandoci sopra mostra la cartella del kit. | — |
| ▶ numero · ⨯ numero | Quante cose sono in esecuzione (azioni dei progetti, ogni versione dei database accesa, anche più d'una dello stesso database, e i pool dei server web accesi) e quante azioni si sono fermate con un errore. Diventa rosso se c'è almeno un errore. | Apre o chiude il pannello «Attività». |
| Rotellina con testo | Un'installazione di componenti in corso e il passo che sta facendo. | — |
| Terminali aperti | Quante sessioni ha il pannello dei terminali dell'app. | Mostra o nasconde i terminali. |
| Editor attivo | Il server dell'editor è acceso (passandoci sopra, il suo indirizzo). | — |
| win32-x64, darwin-arm64… | Il sistema e il processore su cui gira il kit. | — |
| v0.x.y | La versione di Atollo. | — |

Nel pannello «Attività» ogni voce si seleziona per vederne i dettagli: per un'azione di progetto lo stato, l'indirizzo e l'output in diretta, con «Apri nel browser», «Ferma» o, se è finita, «Togli»; per un database il nome, l'indirizzo, eventuali errori, il file di log e «Ferma»; per un pool dei server web l'indirizzo, i VirtualHost collegati e «Ferma»; per i modelli locali l'indirizzo, i modelli in memoria e «Ferma». «Ferma tutto», in alto, chiede «Fermare le azioni dei progetti, i database, i modelli locali e i server web in esecuzione?» e poi ferma azioni, database, modelli locali e pool. Se non c'è niente compare «niente in corso». Il pannello si ridimensiona trascinandone il bordo; un doppio clic sul bordo lo riporta all'altezza iniziale.

#### Scorciatoie e abitudini utili

| Tasti | Che cosa fa |
| --- | --- |
| <kbd>Ctrl+P</kbd> | Apre «Apri un progetto» (⌘P su macOS): scrivi parte del nome, della piattaforma o del linguaggio. Invio apre la pagina del progetto, Maiusc+Invio avvia la sua azione predefinita, Esc chiude. |
| <kbd>Esc</kbd> | Chiude le finestre di conferma e lo zoom delle immagini, e annulla le domande nella pagina (per esempio quella di «Ferma tutto» o della rimozione di un terminale). |

- Passa il mouse sopra icone, pallini e etichette: quasi tutto ha un suggerimento che spiega che cosa è o che cosa fa.
- Lingua e tema si cambiano in Impostazioni › Generale › «Lingua» e «Tema»: di base il tema chiaro o scuro segue quello del sistema operativo, e vale anche per l'editor e i terminali.
- L'icona a forma di libro nella barra laterale di ogni pagina apre l'argomento della guida che la spiega.

Argomenti collegati: [2.2 La pagina Progetti e il «Pannello rapido»](#22-la-pagina-progetti-e-il-pannello-rapido), [8.3 Processi in background](#83-processi-in-background), [1.4 Il primo progetto, passo per passo](#14-il-primo-progetto-passo-per-passo)

### 1.4 Il primo progetto, passo per passo

*Un percorso completo in dieci minuti: creare un progetto, avviarlo, vederlo nel browser e aprirlo nell'editor.*

> **In breve**
>
> - Un esempio completo: un piccolo sito in TypeScript con Vite, dalla creazione al browser.
> - Servono solo il componente Node.js, che il kit installa da solo se manca, e una connessione a Internet.
> - Alla fine saprai creare un progetto, avviarlo, aprirlo nel browser e modificarlo nell'editor.

#### 1. Crea il progetto

1. Apri la pagina «Progetti» dalla barra delle attività.
2. Nel «Pannello rapido», sotto «Nuovo progetto», nel riquadro «Web», fai clic su «Vite» (sotto «Web con JavaScript e TypeScript»). Si apre la scheda «Nuovo progetto».
3. In «1. Base» la scheda «Vite» («Sito o applicazione web in TypeScript») è già scelta.
4. In «2. Aggiunte» non serve niente per questa prova.
5. In «3. Nome» scrivi primo-sito. Il nome diventa la cartella `projects\primo-sito`.
6. Guarda il «Riepilogo»: vedrai che il kit userà Node.js, eseguirà npm create vite nella cartella projects e poi npm install nel progetto, e salverà le azioni «Avvia» e «Build».
7. Premi «Crea il progetto». Nella parte «Avanzamento» scorrono i comandi e il loro output.
8. Quando compare «Progetto primo-sito creato.» premi «Apri il progetto».

> [!NOTE]
> **Da sapere**
>
> Se Node.js non era installato, il kit lo scarica prima di eseguire i comandi: lo vedi nelle prime righe dell'avanzamento («Installo gli strumenti del kit che servono…»).

#### 2. Avvialo e aprilo nel browser

1. La pagina del progetto si apre sulla sezione «Avvio». In basso, l'elenco «Azioni» mostra ciò che il progetto sa fare: le azioni riconosciute dal file package.json (npm run dev, npm run build, npm install) e quelle della ricetta (Avvia, Build).
2. Premi ▶ accanto a «npm run dev». È un'azione web: il kit le assegna una porta, la avvia e aspetta che risponda. Lo stato passa da «in avvio» a «in ascolto».
3. Premi «Apri nel browser»: si apre il sito di prova di Vite all'indirizzo http://localhost:\<porta\>/, dove la porta è un numero tra 7100 e 7999 scelto dal kit e tenuto uguale a ogni avvio.
4. Vuoi che il grande pulsante «Avvia» in alto avvii sempre questa azione? Selezionala e premi la stella «Predefinita».

#### 3. Modificalo nell'editor

1. Premi «Editor» in alto nella pagina del progetto. Il kit avvia l'editor (la prima volta può richiedere qualche secondo) e apre una finestra di Atollo con la cartella del progetto.
2. Apri src/main.ts, cambia un testo e salva con Ctrl+S.
3. Torna al browser: Vite aggiorna la pagina da solo.
4. Per fermare il sito torna su Atollo e premi «Ferma» (o ■ accanto all'azione). Chiudendo Atollo si ferma comunque tutto.

#### E adesso?

- Mettere il progetto su GitHub o clonarne uno: [6.1 L'account GitHub del kit](#61-laccount-github-del-kit).
- Dare al progetto un database: [3.2 Il database di un progetto](#32-il-database-di-un-progetto).
- Farti aiutare da un agente IA nel progetto: [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi).
- Provare altre basi (Laravel, Django, WordPress, Astro…): [2.1 Creare un progetto](#21-creare-un-progetto).

Argomenti collegati: [2.1 Creare un progetto](#21-creare-un-progetto), [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte), [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni)

## 2. Progetti

### 2.1 Creare un progetto

*Le tre strade: una cartella vuota, un repository Git, o una base di framework o CMS con le sue aggiunte. Più i tuoi repository GitHub.*

> **In breve**
>
> - Un progetto è semplicemente una cartella dentro projects: il kit la riconosce da sola.
> - Per crearne uno ci sono tre strade: «Progetto vuoto», «Da un indirizzo Git» e «Da una base», con le basi di framework e CMS in quattro gruppi: Web, CMS, App e Altro. Più l'elenco dei tuoi repository GitHub.
> - Con un framework, il kit installa gli strumenti che servono ed esegue i comandi ufficiali del framework.
> - Puoi anche copiare a mano una cartella in projects: compare nell'elenco dopo «Aggiorna».

![La procedura «Da una base»](../../ui/public/guide/it/project-new.webp)

*Base, aggiunte, nome, file .env e riepilogo, uno sotto l'altro. A sinistra l'albero delle tre strade.*

#### Dove si crea un progetto

Tutte le strade portano alla scheda «Nuovo progetto» della pagina Progetti, che si apre in uno di questi modi:

- dal «Pannello rapido», sotto «Nuovo progetto»: in alto a destra i collegamenti «Progetto vuoto» e «Da un indirizzo Git»; sotto, un riquadro per gruppo (Web, CMS, App, Altro) con le sue basi divise per categoria: un clic su una base apre la procedura con quella base già scelta;
- dalle icone in cima alla barra laterale: «Nuovo progetto» (cartella con il più) e «Nuovo progetto da una base» (bacchetta);
- dal riquadro «Nuovo progetto» della barra laterale, un albero: «Progetto vuoto», «Da un indirizzo Git», poi i rami Web, CMS, App e Altro con il numero delle basi. Sotto Web, App e Altro ci sono le categorie e le loro basi, sotto CMS direttamente le basi; un clic su un gruppo apre la procedura sulla sua prima base, un clic su una base la apre con quella base già scelta;
- da <kbd>Ctrl+P</kbd>, che in fondo all'elenco ha le tre voci «Nuovo progetto».

Un progetto creato o clonato compare subito anche nella pagina «Server Web».

#### Strada 1 · «Progetto vuoto»

Una cartella nuova e vuota in projects, da riempire come vuoi (anche copiandoci dei file).

1. Scrivi il nome della cartella nel campo «Vuoto».
2. Premi «Crea». Si apre la pagina del nuovo progetto.

Il nome non può contenere `\` / : \* ? " \< \> |, non può iniziare o finire con un punto e non può essere un nome riservato di Windows (come CON o NUL). Se esiste già una cartella con quel nome, il kit lo dice.

#### Strada 2 · «Da un indirizzo Git»

Copia (clona) un repository Git esistente in una cartella di projects.

1. Nel primo campo scrivi owner/repository per un repository GitHub (per esempio laravel/laravel), oppure l'indirizzo completo https:// di qualsiasi server Git.
2. Nel secondo campo, facoltativo, il nome della cartella; se lo lasci vuoto è l'ultima parte dell'indirizzo.
3. Premi «Clona».

Dopo la copia il kit guarda i file del progetto, installa i componenti che servono e sceglie le versioni che il progetto dichiara (per esempio la versione di Node scritta in .nvmrc o quella di PHP in composer.json), scaricandole se mancano.

> [!NOTE]
> **Da sapere**
>
> Sono accettati solo indirizzi https://, non ssh. Per un repository privato di GitHub serve l'accesso con l'account GitHub del kit; per i repository pubblici no. Se la copia non riesce, la cartella viene tolta e compare l'ultima riga dell'errore di Git.

#### Strada 3 · «Da una base»

Una procedura guidata che crea un progetto partendo da una base (un framework come Laravel, Django o Vite, un CMS come WordPress, un generatore di siti, un'app desktop o mobile…) e da eventuali aggiunte (Tailwind, React, un database, gli strumenti di prova…). Il kit installa gli strumenti che servono ed esegue i comandi ufficiali del framework, gli stessi che troveresti nella sua documentazione.

| Passo | Che cosa fai |
| --- | --- |
| 1. Base | Scegli un gruppo con le etichette in alto (Web con PHP, Web con JavaScript e TypeScript, Web con Python, Web con altri linguaggi, CMS, Siti statici e documentazione, App desktop, App mobile, Dati e documenti, Riga di comando e librerie) e poi la scheda della base. |
| 2. Aggiunte | Spunta le aggiunte che vuoi. Sono divise per tipo (Interfaccia, Stili, Prove, Database…): di ogni tipo se ne sceglie una sola, e sceglierne un'altra sostituisce la prima. |
| 3. Nome | Scrivi il nome del progetto. In alternativa usa un progetto esistente vuoto («vuoto») o con dei file ma senza Git («con file»): i file del framework si aggiungono e quelli che ci sono restano come sono. Un repository Git non si può usare. |
| 4. File .env | Le variabili che il kit scriverà nel file .env del progetto. {name} diventa il nome del progetto, {package} un identificativo come com.example.nome, {password} una password nuova. Puoi cambiare i valori, togliere righe o aggiungerne con «Aggiungi una variabile». |
| Riepilogo | Che cosa succederà: gli strumenti del kit, il database del progetto, i comandi nella cartella projects, quelli nella cartella del progetto e le azioni che il progetto avrà. |

1. Controlla il «Riepilogo» e premi «Crea il progetto».
2. Nella parte «Avanzamento» vedi «Installo gli strumenti del kit che servono…», poi «Comando 1 di N» con l'output di ogni comando.
3. Alla fine compare «Progetto … creato.» con «Apri il progetto». Se un comando fallisce, la procedura si ferma e mostra l'errore.

<!-- -->

1. **Strumenti** — Installa ciò che la base e le aggiunte chiedono (Node, PHP, Composer…).
2. **Creazione** — Esegue i comandi di creazione nella cartella projects.
3. **Database** — Se previsto, crea database e utente e scrive la connessione in .env.
4. **Rifinitura** — Esegue i passi nella cartella del progetto (installazioni, migrazioni).
5. **Azioni** — Salva le azioni della ricetta nel file panel.json del progetto.

> [!TIP]
> **Consiglio**
>
> Le ricette si possono modificare o se ne possono aggiungere di tue: [6.4 Le istruzioni dei framework](#64-le-istruzioni-dei-framework).

<details>
<summary>Sotto il cofano</summary>

- Si crea un progetto alla volta. «Annulla», durante l'avanzamento, ferma il comando in corso.
- Ogni riga della ricetta è un comando eseguito con la PowerShell del kit e con l'ambiente del kit, quindi con le versioni degli strumenti del kit.
- Se usi un progetto esistente con dei file, il framework viene generato in una cartella temporanea del sistema e poi spostato nel progetto: i file che ci sono già restano, e nell'avanzamento compare «Già nel progetto, lasciato com'è».
- Se il piano prevede MySQL o MongoDB, il kit avvia il server, crea database e utente del progetto e passa ai comandi le variabili ATOLLO_DB_HOST, ATOLLO_DB_PORT, ATOLLO_DB_NAME, ATOLLO_DB_USER, ATOLLO_DB_PASSWORD e ATOLLO_DB_URL. La password generata dal kit prende il posto di una vuota che arriva dalla base (per esempio DB_PASSWORD= nel suo .env.example).
- Alla fine, se c'è un file .env che Git prenderebbe, il kit aggiunge /.env al file .gitignore.
- Le azioni della ricetta vanno nel file panel.json del progetto con un identificativo che comincia con fw- (per esempio fw-avvia).
- Alcune basi richiedono componenti particolari: le app Android vogliono il componente Android SDK; Laravel con MongoDB vuole l'estensione mongodb di PHP, che il kit non installa.

</details>

#### I tuoi repository GitHub

Se hai collegato l'account GitHub, il riquadro «Dai tuoi repository GitHub» della barra laterale elenca i tuoi repository e quelli delle organizzazioni di cui fai parte, raggruppati per proprietario, con una casella «Cerca nei repository».

- «Clona» copia il repository in projects (nella cartella con il suo nome, oppure proprietario-nome se il nome è già preso). Un repository già clonato ha il nome in verde e il pulsante «Editor».
- «GitHub» apre il repository nel browser.
- Con un clic sulla riga si aprono i dettagli: privato o pubblico, linguaggio, stelle, ultimo push, dove è clonato, «Copia l'indirizzo per Git» e, per i repository che amministri, «Rinomina» ed «Elimina da GitHub».

> [!NOTE]
> **Da sapere**
>
> Come «Da un indirizzo Git», dopo la copia il kit installa i componenti che servono e sceglie le versioni che il progetto dichiara. Tutto sull'account in [6.1 L'account GitHub del kit](#61-laccount-github-del-kit).

Argomenti collegati: [1.4 Il primo progetto, passo per passo](#14-il-primo-progetto-passo-per-passo), [2.2 La pagina Progetti e il «Pannello rapido»](#22-la-pagina-progetti-e-il-pannello-rapido), [6.4 Le istruzioni dei framework](#64-le-istruzioni-dei-framework), [6.1 L'account GitHub del kit](#61-laccount-github-del-kit)

### 2.2 La pagina Progetti e il «Pannello rapido»

*L'elenco dei progetti, il «Pannello rapido» con preferiti e recenti, l'apertura veloce con Ctrl+P e le sette sezioni di un progetto.*

> **In breve**
>
> - Nella barra laterale trovi l'elenco dei progetti; con un clic se ne apre la pagina in una scheda.
> - Il «Pannello rapido» raccoglie ciò che è avviato, i preferiti, i recenti e ciò che va sistemato.
> - La pagina di un progetto ha in alto i pulsanti principali e sotto sette sezioni.

![Il «Pannello rapido»](../../ui/public/guide/it/projects-home.webp)

*Recenti, da sistemare e le tre strade per un progetto nuovo. A sinistra l'elenco dei progetti.*

#### La barra laterale

In cima, le icone: «Nuovo progetto», «Nuovo progetto da una base», «Apri un progetto (Ctrl+P)», «Aggiorna», «Apri la cartella» (apre projects) e il libro della guida. Sotto, quattro riquadri:

| Riquadro | Che cosa contiene |
| --- | --- |
| Progetti | Tutti i progetti in ordine alfabetico, con il numero in alto. Le cartelle che iniziano con un punto e i collegamenti non contano. Oltre sei progetti compare la casella «Cerca nei progetti». |
| Dati senza progetto | Compare solo se il kit ha dati di progetti che non sono più in projects. Vedi [2.7 Eliminare un progetto e i «Dati senza progetto»](#27-eliminare-un-progetto-e-i-dati-senza-progetto). |
| Nuovo progetto | L'albero delle strade per creare un progetto: «Progetto vuoto», «Da un indirizzo Git» e le basi nei gruppi Web, CMS, App e Altro. |
| Dai tuoi repository GitHub | I repository del tuo account, da clonare. |

Un clic sul titolo di un riquadro lo apre o lo chiude. Da solo, «Progetti» occupa tutta l'altezza; se è aperto anche «Nuovo progetto» o «Dai tuoi repository GitHub», «Progetti» prende solo l'altezza delle sue righe (e scorre solo se non ci stanno) e il resto va agli altri riquadri aperti.

Ogni riga di progetto ha un'icona che dice che tipo di progetto è: un globo per un sito web, una finestra per un'app con finestra, un terminale per un programma a riga di comando. Un pallino verde vuol dire che qualcosa è avviato; un triangolo giallo che mancano dei componenti. Passando il mouse sulla riga compaiono i pulsanti ▶ «Avvia» (o ■ «Ferma»), «Editor», «Terminale» e uno per ogni agente installato, con il suo logo.

#### Il «Pannello rapido»

È la prima scheda della pagina, sempre presente. Mostra soltanto le parti che hanno qualcosa da mostrare:

| Parte | Che cosa mostra |
| --- | --- |
| In esecuzione | Le azioni avviate in questa sessione, con l'indirizzo e i pulsanti «Apri nel browser», «Vedi l'output», «Ferma». |
| Preferiti | I progetti che hai segnato con la stella («Aggiungi … ai preferiti»), come schede con i loro linguaggi e pulsanti. |
| Recenti | I sei progetti usati o modificati più di recente, escluso chi è già tra i preferiti. |
| Da sistemare | Le azioni fermate con un errore («Vedi l'output») e i progetti a cui mancano componenti («Installa», che apre la sezione Componenti del progetto). |
| Nuovo progetto | Accanto al titolo «Progetto vuoto» e «Da un indirizzo Git»; sotto, un riquadro per gruppo, Web, CMS, App e Altro, con le basi come pulsanti (un clic apre la procedura su quella base); in fondo i collegamenti ai repository GitHub, alla cartella dei progetti e alla guida. |

#### Aprire un progetto per nome

Da qualsiasi pagina premi <kbd>Ctrl+P</kbd> (⌘P su macOS): compare «Apri un progetto». Scrivi parte del nome, della piattaforma o di un linguaggio; i progetti sono in ordine di uso recente. <kbd>Invio</kbd> apre la pagina, <kbd>Maiusc+Invio</kbd> avvia subito l'azione predefinita.

#### La pagina di un progetto

In alto ci sono il nome, il tipo e la piattaforma riconosciuta (per esempio «Laravel · sito web»), i loghi dei linguaggi, il ramo Git e, se mancano componenti, un avviso «Mancano: …». Poi i pulsanti:

| Pulsante | Che cosa fa |
| --- | --- |
| Avvia / Ferma | Avvia l'azione predefinita del progetto, o la ferma se è in corso. |
| Editor | Apre il progetto nell'editor, in una finestra di Atollo. |
| Terminale | Apre una console del kit nella cartella del progetto, con le sue versioni. |
| Loghi degli agenti | Un pulsante per ogni agente installato: il clic sul logo apre quell'agente nel progetto. Da quel momento Claude Code e Codex considerano attendibile la cartella del progetto. |
| Apri la cartella | Apre la cartella del progetto in Esplora file (o Finder). |
| Aggiorna | Rilegge il progetto: utile dopo aver cambiato a mano panel.json o i file del progetto. |

Sotto, le sette sezioni:

| Sezione | A che cosa serve | Argomento |
| --- | --- | --- |
| Avvio | Le azioni del progetto, il loro stato, la porta e l'output. | [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte) |
| Sito web | Servire una cartella del progetto come sito, con file statici, PHP, Caddy o Apache. | [3.3 Siti web e indirizzi .localhost](#33-siti-web-e-indirizzi-localhost) |
| Componenti | Gli strumenti che servono, come è stato riconosciuto il progetto, le versioni. | [2.4 Componenti e versioni di un progetto](#24-componenti-e-versioni-di-un-progetto) |
| Database | Database e utente del progetto, la connessione nel file .env, le copie. | [3.2 Il database di un progetto](#32-il-database-di-un-progetto) |
| Estensioni | Le estensioni dell'editor attive in questo progetto. | [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni) |
| Editor | Impostazioni dell'editor e server MCP del progetto. | [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni) |
| Git | Remoto, ramo e sincronizzazione, in sola lettura. | [2.6 Git di un progetto](#26-git-di-un-progetto) |

> [!NOTE]
> **Come il kit riconosce un progetto**
>
> Il kit guarda i file della cartella: package.json vuol dire Node.js (e con dentro vite o next, un sito web), composer.json PHP, artisan Laravel, pyproject.toml Python, Cargo.toml Rust, go.mod Go e così via. Da lì ricava il tipo, la piattaforma, i linguaggi, i componenti che servono e le azioni. La sezione «Componenti» › «Come è stato riconosciuto» dice da quali file è partito. Il riconoscimento si può correggere con il file panel.json.

Argomenti collegati: [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte), [2.1 Creare un progetto](#21-creare-un-progetto), [1.3 Conoscere la finestra](#13-conoscere-la-finestra)

### 2.3 Avviare un progetto: azioni e porte

*Che cos'è un'azione, come si avvia e si ferma, l'output, la porta, l'azione predefinita e come aggiungerne di tue nel file panel.json.*

> **In breve**
>
> - Un'azione è un comando del progetto con un nome: «npm run dev», «php artisan serve», il sito web…
> - Le azioni web ricevono una porta dal kit, che aspetta che rispondano e ti dà l'indirizzo da aprire.
> - La stella «Predefinita» sceglie l'azione del pulsante «Avvia».
> - Le azioni si riconoscono da sole; le tue si aggiungono con «Nuova azione» e finiscono nel file panel.json del progetto.

![La pagina di un progetto, sezione «Avvio»](../../ui/public/guide/it/project-run.webp)

*In alto i pulsanti Avvia, Editor, Terminale e Agente; sotto, l'azione scelta con stato, comando e porta, e l'elenco delle azioni.*

#### La sezione «Avvio»

In alto c'è il dettaglio dell'azione scelta (all'inizio quella predefinita), sotto l'elenco «Azioni». Un clic su una riga la sceglie e ne mostra l'output; ▶ e ■ sulla riga la avviano e la fermano.

| Voce | Che cosa dice |
| --- | --- |
| Stato | fermo, in avvio, in ascolto (azione web pronta), in esecuzione, terminato, fermato, errore (con il codice di uscita). |
| Comando | Il comando eseguito, oppure il server che serve il sito (file server del kit, Apache o Caddy). |
| Cartella | Dove viene eseguito il comando. |
| Porta | Solo per le azioni web: «automatica» o un numero scelto da te. |
| Indirizzo | Mentre l'azione è in ascolto, l'indirizzo da aprire (un collegamento). |
| Origine | panel.json, se l'azione è scritta a mano nel progetto. |

Sotto il dettaglio c'è il riquadro «Output di …» con la casella «Segui», che tiene in vista le ultime righe. Il kit conserva le ultime 3000 righe di ogni azione; le sue note compaiono come righe che iniziano con \[Atollo\]. I pulsanti in alto: la stella «Predefinita», «Apri nel browser» (mentre un'azione web è in ascolto) e «Togli l'output» (quando l'azione è finita).

#### Che cosa succede quando premi «Avvia»

1. **Rilegge** — Riconosce di nuovo il progetto: panel.json potrebbe essere cambiato.
2. **Installa** — Se all'azione manca un componente, lo installa prima.
3. **Prepara** — Ambiente del kit, versioni del progetto, porta, variabili dell'azione.
4. **Avvia** — Esegue il comando nella cartella dell'azione.
5. **Aspetta** — Per un'azione web, finché la porta risponde: allora è «in ascolto».

- Il browser non si apre da solo: usa «Apri nel browser» (qui, nel «Pannello rapido» o nel pannello «Attività»). L'indirizzo è di solito http://localhost:\<porta\>/.
- Se il programma ignora la porta assegnata e ne stampa un'altra nell'output, il kit segue quella.
- «Ferma» chiude l'intero albero di processi dell'azione. Chiudendo Atollo si fermano tutte le azioni.
- Le azioni avviate compaiono anche nel pannello «Attività» e nel contatore della barra di stato; nel pannello «Ferma tutto», dopo una conferma, le ferma insieme ai database e ai server web in esecuzione.

#### La porta

Ogni azione web ha una porta tutta sua, scelta dal kit tra 7100 e 7999 la prima volta e poi tenuta uguale a ogni avvio: così l'indirizzo non cambia. Se ti serve un numero preciso, scrivilo nel campo «Porta» e premi Invio (o esci dal campo); svuotandolo torni alla porta automatica.

- La porta deve essere tra 1024 e 65535.
- Le porte da 7000 a 7099 sono dell'editor: una di queste viene rifiutata con «Le porte da 7000 a 7099 sono dell'editor».
- Una porta già usata da un'altra azione viene rifiutata con «Porta già assegnata a …», seguito da progetto → azione.
- Viene rifiutata anche una porta che ha già un pool dei server web o una versione di un database, e il messaggio dice di chi è: «La porta … è già del server …: scegline un'altra.» oppure «La porta … è già del database …: scegline un'altra.» (per esempio MySQL 8.4).

#### L'azione predefinita

È quella che parte con il pulsante «Avvia» della pagina, con ▶ nella riga del progetto e con Maiusc+Invio in «Apri un progetto». Per cambiarla scegli l'azione nell'elenco e premi la stella «Predefinita». Quando servi il progetto come sito web, il sito diventa da solo l'azione predefinita.

#### Aggiungere, modificare e togliere azioni

1. Nella sezione «Avvio» premi «Nuova azione», accanto al titolo «Azioni».
2. Compila «Nome», «Tipo» («Sito o server web», «App con finestra» o «Comando») e «Comando»; se serve, «Cartella» (relativa al progetto), «Indirizzo» (solo per un sito) e «Variabili», una NOME=valore per riga. Nel comando, {port} diventa la porta dell'azione.
3. Premi «Salva». L'azione compare nell'elenco; l'identificativo si ricava dal nome.

- «Modifica», sull'azione scelta, riapre lo stesso modulo. Su un'azione riconosciuta salva una tua versione con lo stesso identificativo, che prende il suo posto.
- «Togli» c'è solo per le azioni tue e chiede conferma; se esiste un'azione riconosciuta con lo stesso identificativo, torna quella.
- Il sito web si modifica nella sezione «Sito web», non qui.

#### Il file panel.json

Le azioni tue, la stella e il sito web stanno nel file panel.json nella cartella del progetto. Puoi anche scriverlo a mano e premere «Aggiorna». Il file sta nel progetto, quindi va in Git e vale per chiunque usi il progetto con Atollo.

**Esempio di panel.json** `projects\gestionale\panel.json`

```
{
  "default": "dev",
  "actions": [
    {
      "id": "dev",
      "label": "Avvia tutto",
      "type": "web",
      "command": "npm run dev -- --port {port}",
      "folder": "frontend",
      "env": { "API_URL": "http://localhost:8000" }
    },
    {
      "id": "test",
      "label": "Prove",
      "type": "command",
      "command": "php artisan test"
    }
  ]
}
```

| Chiave | Significato |
| --- | --- |
| `id` | Identificativo dell'azione: minuscole, cifre e - _ : . (al massimo 40 caratteri). |
| `label` | Il nome che vedi nell'elenco. |
| `type` | web (un server con porta), desktop (un'app con finestra) o command (un comando che fa qualcosa e finisce). |
| `command` | Il comando. {port} viene sostituito con la porta dell'azione. |
| `folder` | La sottocartella in cui eseguirlo (facoltativa). |
| `url` | L'indirizzo da aprire, se non è http://localhost:{port}/ (facoltativo). |
| `env` | Variabili d'ambiente in più solo per questa azione (facoltativo). |
| `default` | In cima al file: l'id dell'azione predefinita. |
| `replace` | In cima al file: true per togliere tutte le azioni riconosciute e tenere solo le tue. |

> [!NOTE]
> **Da sapere**
>
> Un'azione con lo stesso id di una riconosciuta la sostituisce. Se il file non è un JSON valido, il kit lo ignora e lo dice in «Componenti» › «Come è stato riconosciuto»; correggilo prima di usare stella, sito web o «Nuova azione», che scrivono nello stesso file.

<details>
<summary>Sotto il cofano</summary>

- Il comando viene eseguito con cmd.exe su Windows e /bin/sh su macOS e Linux, nella cartella dell'azione, con l'ambiente del kit e le versioni del progetto. I token degli agenti (CLAUDE_CODE_OAUTH_TOKEN, GEMINI_API_KEY, LLAMA_API_KEY) non ci sono: arrivano solo agli agenti avviati da Atollo.
- Le azioni web ricevono le variabili PORT (e ROCKET_PORT) con la porta; tutte ricevono BROWSER=none, PYTHONUNBUFFERED=1 e ATOLLO_LANG. Se il progetto ha un database MySQL del kit e il suo .env non dice un'altra porta, ricevono anche DB_PORT.
- Le porte assegnate e quelle scelte da te sono in `data\projects\<progetto>\ports.json`, fuori dal progetto.
- Ogni azione gira in un suo gruppo di processi (un Job Object su Windows): «Ferma» chiude anche i processi figli, per esempio il server avviato da npm.
- Nei file vecchi sono ancora accettate le chiavi in italiano (azioni, etichetta, comando, cartella, predefinita) e {porta} al posto di {port}.

</details>

Argomenti collegati: [3.3 Siti web e indirizzi .localhost](#33-siti-web-e-indirizzi-localhost), [2.2 La pagina Progetti e il «Pannello rapido»](#22-la-pagina-progetti-e-il-pannello-rapido), [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va)

### 2.4 Componenti e versioni di un progetto

*Gli strumenti che servono a un progetto, la versione di ogni linguaggio scelta solo per lui e come il kit la riconosce dai file del progetto.*

> **In breve**
>
> - La sezione «Componenti» dice quali strumenti del kit servono al progetto e li installa con un clic.
> - In «Versioni» scegli, solo per questo progetto, la versione di ogni linguaggio: per esempio Node 20 qui e Node 24 altrove.
> - Un progetto nuovo tiene le versioni più recenti che c'erano quando è stato creato, finché non le cambi.
> - La scelta vale per azioni, terminale, agenti, editor e server web del progetto.

![La sezione «Componenti»](../../ui/public/guide/it/project-versions.webp)

*I componenti che servono al progetto, come è stato riconosciuto e le versioni dei suoi linguaggi.*

#### Componenti che servono

La parte «Componenti» elenca gli strumenti del kit di cui il progetto ha bisogno, ciascuno «installato» o «da installare». Se manca qualcosa, «Installa ciò che manca» li installa e aspetta che abbiano finito. La parte «Come è stato riconosciuto» spiega da quali file il kit ha capito il tipo di progetto, con eventuali avvisi (per esempio «Dipendenze non installate: esegui prima npm install»).

> [!NOTE]
> **Da sapere**
>
> «Installa ciò che manca» installa i componenti del kit (Node.js, PHP, Python…), non le dipendenze del progetto: per quelle esegui npm install, composer install o simili, con un'azione o nel terminale del progetto.

#### Scegliere le versioni del progetto

La parte «Versioni» ha due gruppi: «Linguaggi attivi nel progetto», aperto, e «Linguaggi non presenti nel progetto», chiuso, per impostare altri linguaggi da usare nel terminale o negli script.

1. Apri il menu accanto al linguaggio.
2. La prima voce, «La più recente (…)», lascia decidere al kit: userà sempre la versione più recente presente su disco.
3. Sotto «Scaricate» ci sono le versioni già nel kit; sotto «Da scaricare: 8.x» quelle che la fonte offre e il kit non ha ancora.
4. Scegli una versione: se non è scaricata parte subito il download («Download di … in corso…»). Finché non è pronta il progetto usa la più recente («… non è scaricata: per ora si usa la …», con «Scarica» o «Riprova»).
5. Il pulsante con le frecce, «Cerca altre versioni di …», chiede di nuovo alla fonte l'elenco delle versioni.

Sotto il menu il kit può mostrare dei suggerimenti: «Richiesto da package.json: \>=20» se il progetto dichiara una versione, oppure «Dai container del progetto (Compose o Dockerfile): php 8.3» con il pulsante «Usa 8.3».

> [!TIP]
> **Quando vale la nuova versione**
>
> Per le azioni, il terminale del progetto e gli agenti dal prossimo avvio; per i terminali dell'editor da quelli aperti dopo la modifica; per il sito web servito da Caddy o Apache dal prossimo avvio del sito (il server sceglie il PHP del progetto).

#### Da dove vengono le versioni di partenza

- Un progetto creato dall'app (vuoto o con un framework) fissa le versioni più recenti presenti in quel momento: un aggiornamento futuro del kit non cambia sotto i piedi un progetto che funziona.
- Un progetto clonato «Da un indirizzo Git» usa le versioni che il suo codice chiede, se il kit le riconosce: .nvmrc, .node-version, .python-version, pyproject.toml, composer.json, .php-version, go.mod, global.json, .ruby-version, pom.xml e altri.
- Il riepilogo di chi chiede che cosa è nella vista «Relazioni» della pagina Strumenti: [5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti).

<details>
<summary>Sotto il cofano</summary>

- La scelta viene salvata in `data\projects\<progetto>\dev-settings.json`, alla voce versions; la prima voce del menu la toglie.
- Il kit scrive anche versions.json e versions.txt nella stessa cartella: li leggono il profilo della PowerShell e quello di Git Bash per mettere nel PATH le versioni giuste quando un terminale si apre in quel progetto.
- Oltre al PATH, il kit imposta JAVA_HOME, DOTNET_ROOT e CP_PHP sulle versioni scelte.
- La versione del server di database del progetto si sceglie nella sezione «Database», non qui.

</details>

Argomenti collegati: [5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento), [3.2 Il database di un progetto](#32-il-database-di-un-progetto), [5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti)

### 2.5 Editor, terminale ed estensioni

*Aprire il progetto nell'editor o in una console del kit, scegliere le estensioni, le impostazioni dell'editor e i server MCP del progetto.*

> **In breve**
>
> - «Editor» apre il progetto in VSCodium, dentro una finestra di Atollo, con tutti gli strumenti del kit.
> - «Terminale» apre una console del kit nella cartella del progetto.
> - Le estensioni si scelgono per progetto; quelle installate valgono per tutto l'editor.
> - Nella sezione «Editor» imposti rientro, formattazione, cartelle nascoste e server MCP del progetto.

#### L'editor

L'editor del kit è VSCodium, la versione libera di Visual Studio Code. Il pulsante «Editor» (nella pagina del progetto, nella riga dell'elenco o nel «Pannello rapido») avvia il server dell'editor la prima volta e apre una finestra di Atollo sul progetto. Nella barra di stato compare «Editor attivo».

- I terminali dell'editor hanno già l'ambiente del kit. Su Windows i profili sono «Atollo» (PowerShell, predefinito), «Atollo - Git Bash» e «Atollo - cmd»; su macOS e Linux «Atollo», «Atollo - Bash» e, su macOS, «Atollo - zsh».
- Commit, push, confronto delle modifiche: tutto ciò che fa Git nell'editor funziona con l'account GitHub del kit.
- Il menu «Account» dell'editor e le sue funzioni «Pubblica su GitHub» e «Clona da GitHub» non vedono l'account del kit: usa Git, la pagina Progetti o GitHub CLI.

> [!NOTE]
> **Da sapere**
>
> Se con l'editor aperto cambi nelle Impostazioni la lingua, la rete (per esempio il proxy) o i temporanei dei programmi, l'app propone «Riavvia l'editor», perché i terminali dell'editor usino i nuovi valori.

#### Il terminale del progetto

«Terminale» apre una console del kit nella cartella del progetto: una scheda del pannello dei terminali dentro Atollo, su ogni sistema, con la PowerShell 7 del kit. Gli strumenti sono quelli del kit, alle versioni scelte per il progetto. Il prompt mostra Atollo e la cartella.

Per console in altre cartelle o con strumenti scelti da te c'è la pagina Terminali: [3.4 Terminali del kit](#34-terminali-del-kit).

#### La sezione «Estensioni»

Atollo abbina a ogni linguaggio le sue estensioni dell'editor, prese da Open VSX (il catalogo libero di estensioni). Un progetto riceve quelle dei suoi linguaggi: la sezione le mostra in gruppi (uno per linguaggio, poi «Strumenti» e «Aggiunte da te»), con lo stato «installata», «da installare» o «disattivata».

1. Togli la spunta a un'estensione per disattivarla in questo progetto, o rimettila.
2. Per aggiungerne una che non c'è, scrivi il suo identificativo editore.nome (per esempio ms-python.python) nel campo «Aggiungi un'estensione» e premi «Aggiungi».

> [!NOTE]
> **Da sapere**
>
> Le estensioni installate valgono in tutto l'editor; quelle nuove si installano quando l'editor è aperto. In cima, il riepilogo dice quante sono attive e quante da installare.

#### La sezione «Editor»

Due riquadri.

| Impostazione | Valori |
| --- | --- |
| Formatta al salvataggio | come l'editor, sì, no |
| Rientro | come l'editor, 2 spazi, 4 spazi, tabulazione |
| Cartelle nascoste nell'editor | Una per riga, per esempio \*\*/vendor o \*\*/node_modules: non compaiono nell'elenco dei file. |

«Salva» le scrive nel file `.vscode\settings.json` del progetto; le altre impostazioni del file restano. Se il file contiene commenti o non è JSON semplice, Atollo non lo riscrive e ti chiede di modificarlo nell'editor.

Il secondo riquadro, «Server MCP del progetto», aggiunge server MCP che valgono solo in questo progetto, oltre a quelli del kit. Come funzionano e come si aggiungono: [4.2 Istruzioni, skill e server MCP condivisi](#42-istruzioni-skill-e-server-mcp-condivisi).

Argomenti collegati: [3.4 Terminali del kit](#34-terminali-del-kit), [4.2 Istruzioni, skill e server MCP condivisi](#42-istruzioni-skill-e-server-mcp-condivisi), [5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti)

### 2.6 Git di un progetto

*Che cosa mostra la sezione «Git», dove si fanno commit e push e perché il kit legge lo stato senza eseguire Git.*

> **In breve**
>
> - La sezione «Git» mostra lo stato del repository: remoto, ramo, collegamento al remoto, sincronizzazione.
> - È in sola lettura: commit, push e l'elenco delle modifiche si fanno nell'editor, in un terminale o con un agente.
> - Git usa l'account GitHub del kit senza chiedere password.

#### Che cosa mostra

| Voce | Che cosa dice |
| --- | --- |
| Remoto | L'indirizzo del repository remoto (origin), o «nessuno». |
| Ramo | Il ramo corrente; «nessun ramo» se sei su un commit preciso. |
| Remoto del ramo | Il ramo remoto collegato, o «ramo non collegato al remoto». |
| Sincronizzato | «sì, allo stesso commit», «no: ci sono commit da inviare o da ricevere», oppure «non si sa». Si riferisce all'ultimo aggiornamento dal remoto (fetch). |

«Apri sul web» apre il repository nel browser (per indirizzi https e per GitHub via ssh). Se la cartella non è un repository, la sezione lo dice.

#### Dove si lavora con Git

- Nell'editor: il pannello «Controllo del codice sorgente» per commit, push, pull, rami e confronti.
- Nel terminale del progetto o dell'editor: i comandi git e gh, già autenticati.
- Con un agente: Claude Code, Codex, Antigravity e Pi possono fare commit, creare rami e aprire pull request.

> [!WARNING]
> **Prima del primo commit**
>
> Git vuole un nome e un'e-mail per i commit. Se il kit ha accesso a GitHub li prende dall'account (con l'indirizzo noreply di GitHub); altrimenti scrivili in Impostazioni › «Account GitHub» › «Chi firma i commit». Vengono salvati nel file .gitconfig del kit e ti seguono su ogni PC.

<details>
<summary>Sotto il cofano</summary>

Atollo legge lo stato direttamente dai file della cartella .git (HEAD, config, riferimenti), senza eseguire Git nel progetto: così non può far partire hook o programmi del repository solo perché hai aperto la pagina.

</details>

Argomenti collegati: [6.1 L'account GitHub del kit](#61-laccount-github-del-kit), [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni), [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi)

### 2.7 Eliminare un progetto e i «Dati senza progetto»

*Che cosa si elimina e che cosa resta, e come ricollegare o pulire i dati di un progetto rinominato o cancellato fuori dall'app.*

> **In breve**
>
> - Il kit tiene alcuni dati di ogni progetto fuori dal progetto, in `data\projects`: impostazioni, porte, password dei database, copie.
> - Dall'app si elimina solo un progetto con la cartella vuota; per gli altri cancelli la cartella tu.
> - Quando la cartella sparisce, i suoi dati diventano «Dati senza progetto»: li ricolleghi o li elimini.
> - I dati senza progetto non vengono mai eliminati da soli.

#### Dove sono i dati di un progetto

| Dentro il progetto (va in Git) | Fuori, in `data\projects\<progetto>` (resta nel kit) |
| --- | --- |
| Il tuo codice, panel.json, .env, `.vscode\settings.json`, .mcp.json | Versioni scelte, estensioni, preferiti (dev-settings.json), porte (ports.json), configurazioni di Apache e Caddy, password e nomi dei database (private), copie dei database (`private\backups`) |

#### Eliminare un progetto

Se la cartella del progetto è vuota, la sezione «Avvio» mostra «Elimina la cartella vuota». Per un progetto con dei file, il kit non elimina niente al posto tuo: cancella o sposta la cartella in Esplora file, poi premi «Aggiorna»; i suoi dati compaiono in «Dati senza progetto».

La finestra «Eliminare …?» elenca che cosa sparirà (il progetto sparisce subito anche dalla pagina «Server Web»):

- la cartella vuota e le impostazioni del kit per quel progetto;
- il database e il suo utente su MySQL o MongoDB (un server fermo viene avviato apposta);
- le copie dei database, con «Apri la cartella delle copie» per salvarle prima altrove.

Se l'eliminazione di un database non riesce, non viene eliminato nient'altro; compare allora anche «Elimina senza i database», che lascia database e utenti sui server.

> [!WARNING]
> **Attenzione**
>
> Le copie dei database vengono eliminate insieme al progetto. Se ti servono, copiale prima fuori dal kit.

#### I «Dati senza progetto»

Se rinomini o cancelli la cartella di un progetto fuori dall'app, il kit si ritrova dei dati senza progetto. Il riquadro «Dati senza progetto (N)» li elenca, con che cosa contengono: impostazioni, porte, database, copie.

| Pulsante | Che cosa fa |
| --- | --- |
| Collega a… | Sposta impostazioni, porte, database e copie a un progetto che scegli nel menu «Progetto». Database e utente mantengono il vecchio nome, così il file .env continua a funzionare. |
| Elimina | Elimina quei dati, database compresi, con la stessa finestra di conferma dell'eliminazione di un progetto. |

> [!NOTE]
> **Quando «Collega a…» non si può usare**
>
> Il progetto di destinazione può avere al massimo le sue impostazioni (versioni, preferito, ultimo agente): quelle dei dati collegati prendono il loro posto. Se ha già porte scelte, server, un database o delle copie, il collegamento viene rifiutato, per non mescolare i dati di due progetti.

> [!TIP]
> **Consiglio**
>
> Per rinominare un progetto senza perdere niente: rinomina la cartella in Esplora file, torna su Atollo, premi «Aggiorna» e usa «Collega a…» sui suoi vecchi dati, scegliendo il progetto con il nome nuovo.

Argomenti collegati: [3.2 Il database di un progetto](#32-il-database-di-un-progetto), [8.2 Le cartelle del kit su disco](#82-le-cartelle-del-kit-su-disco), [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza)

## 3. Database, siti e terminali

### 3.1 I server di database

*MySQL e MongoDB del kit: avviarli, le password, le porte, più versioni insieme, phpMyAdmin, mongo-express e l'esploratore dei dati.*

> **In breve**
>
> - Il kit ha un server MySQL e uno MongoDB, condivisi da tutti i progetti e raggiungibili solo da questo PC.
> - «Avvia» installa il server se manca, lo accende e genera la password di amministratore.
> - Ogni progetto ha poi il suo database e il suo utente, che si creano dalla pagina del progetto.
> - MySQL si esplora e si modifica con phpMyAdmin, MongoDB con mongo-express, entrambi nel browser.

![La pagina Database, server MySQL](../../ui/public/guide/it/databases.webp)

*Stato e indirizzo, connessione, versione, porta, «Avvia con Atollo» e le altre versioni installate.*

#### La pagina Database

Nella barra laterale, il riquadro «Server» elenca MySQL e MongoDB con la porta, lo stato e il pulsante «Avvia» o «Ferma». Un clic su un server ne apre la pagina. In cima alla barra: «Aggiorna», «Apri la cartella dei dati» (`data\db`) e il libro della guida.

| Stato | Significato |
| --- | --- |
| Fermo (grigio) | Il server è spento. |
| Avvio in corso… (giallo) | Il server sta partendo; il kit aspetta che risponda (fino a 90 secondi). |
| Avviato (verde) | Il server risponde su 127.0.0.1 alla sua porta. |
| Errore (rosso) | Non è partito: la pagina mostra l'ultimo errore e il file di log. |
| Non installato: si installa al primo avvio | Il componente non c'è ancora; «Avvia» lo scarica prima. |

#### La pagina di un server

In alto lo stato, l'indirizzo 127.0.0.1:\<porta\>, la versione e i pulsanti «Avvia» o «Ferma»; per MySQL anche «Apri phpMyAdmin», per MongoDB «Apri mongo-express».

| Parte | Che cosa contiene |
| --- | --- |
| Connessione | «Stringa»: l'indirizzo per collegarsi come amministratore (senza password). «Password di root» (MySQL) o «Password di admin» (MongoDB): nascosta, con «Mostra» e «Copia». Prima del primo avvio: «Generata al primo avvio.» |
| Impostazioni | «Versione» (se ne hai più di una; si cambia a server fermo), «Porta» (tra 1024 e 65535, vale dal prossimo avvio) e «Avvia con Atollo» (il server parte da solo all'apertura dell'app). |
| Altre versioni | Compare se hai installato più versioni dello stesso server: ognuna gira sulla sua porta con i suoi dati (vedi sotto). |
| File | «Dati»: la cartella dei dati di quella versione. «Log»: il file con l'output del server. |

> [!NOTE]
> **Solo da questo PC, sempre con password**
>
> I server rispondono soltanto all'indirizzo 127.0.0.1, cioè a questo computer: nessuno dalla rete può collegarsi, nemmeno con l'opzione «Rete locale» delle Impostazioni. Le password di amministratore sono generate dal kit (32 caratteri casuali) al primo avvio. Nella console del kit il comando mysql entra già come root.

#### Esplorare i dati

| Server | Come vedi i dati |
| --- | --- |
| MySQL | «Apri phpMyAdmin». La prima volta il kit installa phpMyAdmin (e PHP se manca), accende MySQL se è spento e apre phpMyAdmin nel browser, già collegato come root. Per un'altra versione c'è «phpMyAdmin su …» nella parte «Altre versioni». |
| MongoDB | «Apri mongo-express». La prima volta il kit installa mongo-express (e Node.js se manca), accende MongoDB se è spento e apre mongo-express nel browser, già collegato come admin: database, collezioni, documenti, indici, importazione ed esportazione. Per un'altra versione c'è «mongo-express su …». |

<details>
<summary>Sotto il cofano: phpMyAdmin e mongo-express</summary>

phpMyAdmin gira con il server PHP integrato del kit su un indirizzo di loopback casuale (127.x.y.z, non 127.0.0.1) e su una porta scelta dal sistema, con un gettone nuovo a ogni avvio: senza il gettone risponde 403. Se ne apre uno alla volta; si ferma con la versione di MySQL a cui è collegato o con l'app. La sua lingua è quella di Atollo.

mongo-express è una build di Atollo: compilata in CI dal codice di mongo-express a un commit fissato, pubblicata nelle release di Atollo e controllata col suo SHA-256 prima dell'installazione. Gira col Node.js del kit, allo stesso modo di phpMyAdmin: indirizzo di loopback suo, porta che nessun'altra parte di Atollo usa, gettone nuovo a ogni avvio, 403 senza. La password di admin di MongoDB gli arriva solo nel suo ambiente, mai sulla riga di comando o nel browser. L'interfaccia di mongo-express è in inglese.

</details>

#### Più versioni dello stesso server

Puoi installare più versioni di MySQL o di MongoDB (dalla pagina Strumenti). Il server usa quella scelta in «Versione», di solito la più recente. Le altre compaiono nella parte «Altre versioni»: ciascuna ha la sua porta (3307, 3308… per MySQL, assegnate al primo avvio) e la sua cartella dei dati, e si avvia insieme al server principale se un progetto la usa.

- Al primo avvio di una versione più recente il kit copia i dati della precedente, che restano com'erano: puoi tornare indietro.
- Un progetto sceglie la versione del server nella sua sezione «Database»; la porta gli arriva come DB_PORT.
- Ogni versione tiene la sua porta: cambiare la «Versione» del server non sposta nessuna porta, e le connessioni dei progetti restano giuste.
- Una porta che ha già un'altra versione, un pool dei server web o un'azione di un progetto, o una fra 7000 e 7099 (dell'editor), viene rifiutata: il messaggio dice di chi è.
- Due avvii insieme della stessa versione (dalla pagina e da un progetto, per esempio) accendono un solo server, sulla stessa porta.

<details>
<summary>Sotto il cofano</summary>

| Che cosa | Dove |
| --- | --- |
| Dati di ogni versione | `data\db\mysql\8.4 · data\db\mongodb\8.0` |
| Porte, avvio automatico, versione scelta | `data\app\services.json` |
| Password di root e admin | `data\app\db\secrets.json` |
| Accesso di mysql nella console | `data\app\db\mysql.cnf` |
| Log | `data\app\logs\mysql.log · mongodb.log` |

La cartella `data\app\db` è leggibile solo dal tuo utente sui dischi NTFS. MySQL usa un file my.cnf scritto dal kit a ogni avvio (solo 127.0.0.1, niente log binario, nomi delle tabelle in minuscolo); MongoDB parte sempre con l'autenticazione. Il comando mysql nella console entra come root solo sulla versione principale.

</details>

Argomenti collegati: [3.2 Il database di un progetto](#32-il-database-di-un-progetto), [5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento), [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va)

### 3.2 Il database di un progetto

*Un database e un utente solo per il progetto, la connessione scritta nel file .env, le copie di sicurezza e il ripristino.*

> **In breve**
>
> - Ogni progetto ha un database e un utente tutti suoi, con una password generata dal kit.
> - «Scrivi in .env» mette la connessione nel file .env del progetto, dopo averti mostrato che cosa cambia.
> - «Esporta» salva una copia del database; «Ripristina…» la rimette.
> - Tutto si fa da Progetti › il progetto › sezione «Database».

![La sezione «Database» di un progetto](../../ui/public/guide/it/project-db.webp)

*Database, utente, password e indirizzo del progetto, «Scrivi in .env» e le copie, con «Esporta» e «Ripristina…».*

#### Creare database e utente

La sezione ha una parte per MySQL e una per MongoDB (con «usato dal progetto» se il kit vede che il progetto lo usa). Se il progetto usa SQLite, una nota spiega che il database è un file del progetto e non serve nessun server.

1. Premi «Crea database e utente» nella parte del server che ti serve.
2. Il kit installa e avvia il server se serve, poi crea un database e un utente con il nome del progetto (minuscole, cifre e trattino basso: gestionale-fatture diventa gestionale_fatture) e una password di 32 caratteri.
3. La parte mostra «Database», «Utente», «Password» e «Stringa» (nascoste, con «Mostra» e «Copia») e «Indirizzo».

> [!NOTE]
> **Da sapere**
>
> L'utente del progetto può fare tutto, ma solo sul suo database: un progetto non vede i dati degli altri. Se il nome è riservato (come mysql o admin) il kit aggiunge _db.

#### Scrivere la connessione nel file .env

1. Premi «Scrivi in .env».
2. Compare «Righe di .env che cambiano»: per ogni chiave il valore vecchio barrato e quello nuovo (password nascoste).
3. Premi «Scrivi» per confermare o «Annulla».

Il kit sceglie le chiavi in base al tuo file .env:

| Se il .env… | MySQL scrive | MongoDB scrive |
| --- | --- | --- |
| contiene già DB_CONNECTION= (stile Laravel) | `DB_CONNECTION, DB_HOST, DB_PORT, DB_DATABASE, DB_USERNAME, DB_PASSWORD` | `DB_CONNECTION=mongodb, DB_DATABASE, MONGODB_URI` |
| non lo contiene | `DATABASE_URL` | `MONGODB_URI` |

**Esempio: progetto Laravel con MySQL** `.env`

```
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=gestionale
DB_USERNAME=gestionale
DB_PASSWORD=3f9c…(32 caratteri generati dal kit)
```

**Esempio: altro progetto con MySQL e MongoDB** `.env`

```
DATABASE_URL=mysql://portale:3f9c…@127.0.0.1:3306/portale
MONGODB_URI=mongodb://portale:a71e…@127.0.0.1:27017/portale?authSource=portale
```

- Se il file .env non esiste viene creato copiando .env.example. Le altre righe e i commenti restano.
- Se la connessione è già quella giusta: «Il file .env ha già questa connessione.»

> [!WARNING]
> **Attenzione**
>
> Il file .env contiene la password: non deve finire nel repository. Se .env non è nel file .gitignore, il kit lo segnala: aggiungilo.

#### Copie e ripristino

1. Nella parte «Copie» premi «Esporta»: compare «Copia salvata: …» e la copia si aggiunge all'elenco.
2. Per tornare a una copia premi «Ripristina…» accanto al file e conferma: «I dati attuali vengono sostituiti.»
3. Prima di ripristinare, il kit copia ciò che il database contiene ora, in \<database\>-prima-del-ripristino-\<data e ora\>: se la copia non riesce, il ripristino non si fa. Così un ripristino del file sbagliato si annulla ripristinando quella copia.
4. «Apri la cartella delle copie» apre la cartella: per ripristinare una copia arrivata da fuori (da un collega, da un altro kit) mettila lì e comparirà nell'elenco.

> [!TIP]
> **Consiglio**
>
> Le copie stanno nel kit, in `data\projects\<progetto>\private\backups`. Per sicurezza tienine una anche fuori dal kit: se perdi la chiavetta, perdi anche le copie.

Delle copie di sicurezza («prima-del-ripristino» e «prima-del-cambio-di-versione») il kit tiene le ultime 3 di ogni tipo ed elimina le più vecchie. Le copie fatte con «Esporta» non vengono mai eliminate.

#### Cambiare la versione del server

Se hai più versioni di MySQL o MongoDB, il menu «Versione del server» sceglie su quale gira il database del progetto. Cambiandola, database e utente si creano anche sulla nuova versione, vuoti: i dati si portano con una copia (Esporta sulla vecchia, Ripristina sulla nuova). Se la versione scelta non è installata, compare «Installa …». Prima del cambio il kit copia il database sulla versione che lascia, in \<database\>-prima-del-cambio-di-versione-\<data e ora\>: se la copia non riesce, la versione non cambia (se quella versione non è più installata, non c'è niente da copiare e il cambio si fa).

<details>
<summary>Sotto il cofano</summary>

- Nomi, password e versioni del database del progetto: `data\projects\<progetto>\private\db.json`, leggibile solo dal tuo utente su NTFS.
- MySQL: utente con caching_sha2_password su localhost e 127.0.0.1, tutti i permessi sul solo database del progetto. MongoDB: ruolo dbOwner sul database, authSource uguale al nome del database.
- Le copie MySQL sono file .sql fatti con mysqldump (--single-transaction, con routine, trigger ed eventi); quelle MongoDB file .archive.gz fatti con mongodump. Il ripristino MySQL gira con l'utente del progetto e senza comandi del client; quello MongoDB tocca solo il database del progetto.
- Le copie di sicurezza MySQL usano mysqldump --force: un oggetto rotto (una vista su una tabella eliminata) non ferma la copia del resto. Prima di un ripristino MySQL l'amministratore toglie viste, trigger, routine ed eventi del solo database del progetto (quelli di root l'utente del progetto non potrebbe sostituirli); la copia li ricrea come utente del progetto.
- Eliminando il progetto dall'app, il kit elimina anche database e utente su ogni versione in cui li aveva creati.

</details>

Argomenti collegati: [3.1 I server di database](#31-i-server-di-database), [2.7 Eliminare un progetto e i «Dati senza progetto»](#27-eliminare-un-progetto-e-i-dati-senza-progetto), [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza)

### 3.3 Siti web e indirizzi .localhost

*Servire una cartella come sito: file statici, PHP, Caddy o Apache. I server condivisi della pagina «Server Web», i domini .localhost e gli alias.*

> **In breve**
>
> - Nella sezione «Sito web» di un progetto scegli una cartella e come servirla: file statici, PHP, Caddy o Apache.
> - Con Caddy e Apache il sito entra in un server condiviso con gli altri progetti delle stesse versioni, su un indirizzo come http://mio-sito.localhost:8081.
> - La pagina «Server Web» mostra questi server condivisi (pool) e permette di crearne di tuoi (profili).
> - I siti rispondono solo a questo PC, salvo che tu non attivi la rete locale nelle Impostazioni.

![La sezione «Sito web» di un progetto](../../ui/public/guide/it/project-site.webp)

*La cartella da servire, il modo (file statici, PHP, Caddy, Apache) e, per Caddy e Apache, profilo, domini e alias.*

![La pagina Server Web](../../ui/public/guide/it/servers.webp)

*I pool e i profili a sinistra; a destra i siti collegati, i dettagli e le opzioni di avvio del pool scelto.*

#### Servire un progetto come sito

1. Apri il progetto e la sezione «Sito web».
2. In «Servi come sito web la cartella» scrivi la cartella da pubblicare: . per tutto il progetto, oppure per esempio public.
3. In «con» scegli come servirla (vedi la tabella).
4. Per Caddy e Apache scegli anche il «Profilo server».
5. Premi «Salva». Il sito diventa l'azione predefinita del progetto: si avvia con «Avvia».

| Scelta | Quando usarla |
| --- | --- |
| file statici | HTML, CSS, JavaScript già pronti. È la scelta iniziale. Il kit usa il suo piccolo server di file, che risponde anche come \<progetto\>.localhost. |
| PHP (server integrato) | Un sito PHP semplice, servito dal server integrato di PHP. |
| Caddy | Un sito PHP o statico servito da Caddy. Caddy non legge i file .htaccess: le regole di riscrittura le genera il kit dal tipo di progetto. |
| Apache | Un sito che ha bisogno di Apache e dei suoi file .htaccess (molti CMS PHP). |

«Togli il sito» toglie la configurazione. Per cambiarla, ferma prima il sito: la modifica vale dal prossimo avvio.

#### Il profilo server: condiviso o tutto suo

| Profilo | Che cosa succede |
| --- | --- |
| Automatico (pool per versione) | La scelta iniziale. Il sito entra nel server condiviso delle sue versioni (per esempio Apache 2.4 con PHP 8.4), insieme agli altri progetti che usano le stesse: un'unica porta, un nome per ogni sito. |
| Server tutto suo (porta del progetto) | Un Caddy o un Apache solo per questo progetto, sulla porta del progetto. La configurazione è in `data\projects\<progetto>\apache` o `\caddy`, e le tue direttive vanno nella sottocartella conf.d (file .conf per Apache, .caddy per Caddy). |
| Un profilo tuo | Uno dei profili creati nella pagina Server Web, per esempio con una versione di PHP precisa. |

1. **Avvia** — Premi «Avvia» nel progetto.
2. **Pool** — Il kit accende il pool delle sue versioni, o lo ricarica se è già acceso.
3. **Nome** — Il sito riceve il nome \<progetto\>.localhost.
4. **Indirizzo** — Lo apri su http://\<progetto\>.localhost:\<porta del pool\>.

- «Ferma» sgancia il sito dal pool. Un pool acceso dai progetti si spegne con l'ultimo progetto, a meno che tu non l'abbia avviato dalla pagina Server Web o che il suo profilo dica «Mantieni attivo anche senza progetti».
- Se il pool automatico non riesce a partire, il sito usa un server tutto suo e l'output lo dice: «Il server condiviso … non è partito (…): il sito usa un server tutto suo.»
- Nel pool il sito tiene le tue direttive, come con un server tutto suo: Apache include i file .conf di `data\projects\<progetto>\apache\conf.d`, Caddy importa i .caddy di `data\projects\<progetto>\caddy\conf.d`.

#### Domini .localhost e alias

Con Caddy e Apache puoi dare al sito altri nomi e collegare cartelle di altri posti. I nomi che finiscono in .localhost portano sempre a questo PC, senza toccare il file hosts di Windows.

| Campo | Esempio | Effetto |
| --- | --- | --- |
| Domini .localhost aggiuntivi | `api.negozio.localhost, admin.negozio.localhost` | Il sito risponde anche a questi nomi. Solo nomi che finiscono in .localhost. |
| Alias di percorso portabili | `/media → storage/uploads` | L'indirizzo /media serve la cartella storage/uploads del progetto. Con @altro-progetto/public puoi servire la cartella di un altro progetto. |

«Aggiungi alias» aggiunge una riga «Percorso URL» → «Cartella relativa»; «Rimuovi» la toglie. Le cartelle devono essere relative (niente percorsi assoluti né ..), così il sito continua a funzionare se il kit cambia lettera o PC.

#### La pagina «Server Web»

Il riquadro «Pool attivi e profili» elenca i server condivisi con la porta, lo stato e il numero di siti. Un clic ne apre la pagina:

| Parte | Che cosa contiene |
| --- | --- |
| VirtualHost collegati | I siti del pool: progetto, cartella, indirizzo cliccabile, nomi e alias. «Sganciato: il progetto è fermo» se il progetto è stato fermato. |
| Dettagli del pool | Motore, versione del server, versione di PHP, porta, tipo (pool automatico o profilo manuale), cartella della configurazione. |
| Avvio e arresto | Per un profilo: «Mantieni attivo anche senza progetti» e «Avvia con Atollo». Un pool automatico parte con il primo progetto che lo usa e si ferma con l'ultimo. |

I pulsanti in alto: «Avvia» o «Ferma», «Ricarica configurazione» e, per i profili, «Elimina profilo», che chiede prima conferma. Un pool ancora acceso ma non più legato a progetti o profili mostra un avviso: fermalo. Un progetto creato, clonato o eliminato compare o sparisce subito dalla pagina.

1. Per creare un profilo premi «Nuovo profilo server» in cima alla barra laterale.
2. Compila «Nome del profilo», «Server web» (Caddy o Apache httpd), «Versione server», «Versione PHP» e, se vuoi, «Porta» (altrimenti il kit ne sceglie una libera tra 8081 e 8999). Una porta che ha già un altro pool, una versione di un database o un'azione di un progetto, o una fra 7000 e 7099 (dell'editor), viene rifiutata: il messaggio dice di chi è.
3. Scegli se tenerlo acceso anche senza progetti e se avviarlo con Atollo, poi premi «Crea profilo».
4. Nella sezione «Sito web» dei progetti, scegli il profilo nel menu «Profilo server».

#### Farsi vedere dagli altri PC della rete

Di norma i siti rispondono solo a questo PC. Per provarli da un telefono o da un altro computer della stessa rete, attiva «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete» in Impostazioni › Rete: [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale). Gli altri dispositivi usano l'indirizzo IP di questo PC e la porta, anche con il server integrato di PHP; i nomi .localhost funzionano solo qui. Database e phpMyAdmin restano comunque chiusi alla rete.

<details>
<summary>Sotto il cofano</summary>

- La configurazione del sito è nel file panel.json del progetto (azione site); i profili in `data\app\servers.json`.
- La configurazione dei pool viene rigenerata a ogni avvio e ricarica in `data\servers\<pool>` (httpd.conf o Caddyfile, una cartella vhosts con un file per sito, i log), mai nel progetto.
- I pool automatici hanno un nome come apache-2.4_php-8.4 o caddy-2.11 e una porta tra 8081 e 8999 ricordata fra un avvio e l'altro, scelta fuori dalle porte che hanno già gli altri pool, i database e le azioni dei progetti.
- Solo HTTP, niente HTTPS. Il PHP gira come FastCGI (php-cgi con Caddy su Windows, php-fpm su macOS e Linux).
- Ricarica: Caddy si ricarica senza fermarsi; Apache, o un cambio di porta o di PHP, fa ripartire il pool.

</details>

Argomenti collegati: [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte), [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale), [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va)

### 3.4 Terminali del kit

*Console con gli strumenti del kit in qualsiasi cartella del PC, e terminali tuoi con strumenti e versioni scelti.*

> **In breve**
>
> - Un terminale del kit è una console con solo l'ambiente del kit: niente profilo né programmi del PC.
> - Si apre in qualsiasi cartella del PC, non solo nei progetti.
> - Il terminale «Tutti gli strumenti» è sempre pronto; puoi crearne di tuoi con strumenti e versioni scelti.

![La pagina Terminali](../../ui/public/guide/it/terminals.webp)

*Il terminale «Tutti gli strumenti» e quelli tuoi; a destra cartella di partenza, strumenti e versioni.*

#### La pagina Terminali

Il riquadro «Terminali» elenca il terminale predefinito «Tutti gli strumenti» («Ogni strumento alla versione più recente») e quelli che hai creato tu. «Apri» su una riga apre la console nella sua cartella di partenza.

- Si apre una scheda del pannello dei terminali dentro Atollo, su ogni sistema, con la PowerShell 7 del kit (su macOS e Linux, se non c'è, bash). Mai il terminale del PC: la scheda si può spostare in una finestra di Atollo tutta sua.
- Su Windows il terminale usa il ConPTY del kit (conpty.dll e OpenConsole.exe di Microsoft, la console di Windows Terminal), una parte dell'app installata con i componenti obbligatori: se manca, l'app lo scarica prima di aprire il terminale (avanzamento in «Attività»).
- All'apertura compare «Atollo · \<nome del terminale\>» e l'elenco degli strumenti trovati; il prompt mostra Atollo e la cartella corrente.

#### Creare un terminale tuo

Utile quando ti serve sempre lo stesso insieme: per esempio un terminale «PHP 7.4 legacy» con solo PHP 7.4 e Composer, aperto nella cartella di un vecchio progetto fuori dal kit.

1. Premi «Nuovo terminale» in cima alla barra laterale, oppure «Duplica» su un terminale esistente.
2. In «Generale» scrivi il «Nome» e la «Cartella di partenza» (qualsiasi cartella del PC: «Scegli…» per sfogliare, «Usa projects» per la cartella dei progetti).
3. In «Strumenti e versioni» spunta gli strumenti (gruppi Agenti, Linguaggi, Database; «Tutti» e «Nessuno» per fare in fretta) e per ciascuno la versione, o «La più recente».
4. Premi «Crea terminale» (o «Salva» per uno esistente), poi «Apri».

- Gli strumenti non scelti restano fuori dal PATH. Sono sempre presenti PowerShell 7, GitHub CLI, i comandi del kit e quelli del sistema.
- «Apri in…» apre il terminale in un'altra cartella solo per quella volta.
- Scegliere uno strumento di database mette nel PATH il suo client (per esempio mysql), ma non avvia il server: quello si avvia dalla pagina Database.
- Il terminale predefinito non si modifica: duplicalo per crearne uno tuo. «Elimina» chiede conferma (Esc chiude la domanda) e toglie solo il terminale, non gli strumenti.
- Se una versione scelta non è più installata, il terminale usa la più recente e lo dice, per esempio «PHP 8.2 → 8.4»; uno strumento tolto del tutto resta fuori.

#### Il pannello dei terminali

Il pannello dei terminali sta in basso, sopra il pannello «Attività». Ogni sessione è una scheda («app · pwsh», «app · claude»…); «Chiudi il terminale» chiude quella scelta (se dentro gira ancora un programma, l'app chiede conferma) e «Nascondi i terminali» nasconde il pannello. Nella barra di stato un elemento conta i terminali aperti e li mostra o nasconde.

«Sposta in una nuova finestra» stacca la scheda scelta: il terminale continua in una finestra sua, che si può spostare su un altro schermo o ingrandire, con la stessa sessione, la sua uscita e lo stesso carattere (Geist Mono). «Riporta nel pannello» lo rimette tra le schede; chiudere la finestra fa lo stesso finché il programma gira, e non lo ferma mai. Per fermarlo, «Chiudi il terminale», nella finestra o nel pannello.

Su Windows e Linux Ctrl+C copia il testo selezionato (senza selezione arriva al programma), Ctrl+Maiusc+C copia, Ctrl+V e Ctrl+Maiusc+V incollano; su macOS ⌘C e ⌘V.

<details>
<summary>Sotto il cofano</summary>

- I terminali creati da te sono in `data\app\terminals.json` (nome, cartella, strumenti e versioni).
- La console parte con pwsh -NoProfile e il profilo del kit (`data\kit\profile.ps1`): i tuoi profili e moduli di PowerShell del PC non vengono caricati.
- In un terminale della pagina Terminali la variabile ATOLLO_TERMINAL dice il suo nome: entrando nella cartella di un progetto non cambia versioni, tiene quelle scelte per il terminale. Il «Terminale» di un progetto invece usa le versioni del progetto.
- All'apertura non si scarica niente (salvo la PowerShell del kit se manca): conta ciò che è già su disco.

</details>

Argomenti collegati: [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni), [8.1 Portabile e isolato: che cosa vuol dire](#81-portabile-e-isolato-che-cosa-vuol-dire), [5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento)

## 4. Agenti IA

### 4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi

*Che cosa sono, come si installano, come si aprono in un progetto, l'accesso al proprio account e l'uscita.*

> **In breve**
>
> - Un agente IA è un assistente che lavora nella cartella del progetto: legge il codice, lo modifica, esegue comandi.
> - Il kit ne offre quattro: Claude Code (Anthropic), Codex (OpenAI), Antigravity (Google) e Pi (Earendil). Si installano da Strumenti.
> - Pi funziona anche senza account, sui modelli locali del kit, che girano sul tuo PC.
> - Si aprono sempre dentro un progetto, dalla pagina Progetti, con il pulsante che porta il logo dell'agente.
> - L'accesso al tuo account si fa al primo avvio dell'agente e resta nel kit, anche su un altro PC.

![La pagina Agenti](../../ui/public/guide/it/agents.webp)

*Gli agenti installati con accesso e limite più alto; a destra limiti e consumi, accesso e cartella condivisa.*

#### Che cosa sono e che cosa serve

Un agente è un programma a riga di comando che dialoga con un modello di intelligenza artificiale del fornitore. Gli chiedi in italiano che cosa fare («aggiungi una pagina di contatto», «perché questo test fallisce?») e lui legge i file, propone modifiche, esegue comandi e ti mostra i risultati. Per usarlo serve un account del fornitore con un piano che lo comprenda, oppure una chiave API; Pi può usare invece un modello locale, senza account.

| Agente | Fornitore | Accesso |
| --- | --- | --- |
| Claude Code | Anthropic | Account Claude (abbonamento); su macOS anche un token di claude setup-token. |
| Codex | OpenAI | Account ChatGPT oppure chiave API salvata da Codex con codex login --with-api-key. |
| Antigravity | Google | Account Google oppure chiave API di Gemini, da scrivere in Impostazioni › Agenti. |
| Pi | Earendil (Mario Zechner) | Nessuno con i modelli locali del kit; per un provider in cloud, /login dentro Pi. |

Gli agenti si installano dalla pagina Strumenti, tipo «Agenti AI» (il primo avvio propone già Claude Code, Codex e Antigravity; Pi non è tra quelli proposti, si spunta a mano). Seguono sempre l'ultima versione e portano con sé ciò che serve; Git, che usano tutti, è già tra i componenti sempre installati.

> [!NOTE]
> **Pi**
>
> Pi è un agente da terminale che lavora con molti fornitori. Nel kit parte da solo su un modello locale: quello scelto nella pagina «Modelli locali», altrimenti il primo del kit che sa usare gli strumenti, e prima di aprirlo il kit avvia il server dei modelli. Se in Pi scegli un provider in cloud con /login, resta su quello; i modelli locali restano nel suo elenco (/model). Su Windows Pi usa il Git Bash del kit come shell. Vedi [4.4 Modelli locali: llama.cpp e i GGUF](#44-modelli-locali-llamacpp-e-i-gguf).

#### Aprire un agente in un progetto

La pagina Agenti non avvia gli agenti: un agente lavora sempre nella cartella di un progetto, quindi si apre dalla pagina Progetti.

1. Apri il progetto.
2. In alto, accanto a «Editor» e «Terminale», c'è un pulsante per ogni agente installato, con il suo logo («Apri Claude Code», «Apri Codex», «Apri Antigravity», «Apri Pi»). Fai clic su quello dell'agente che vuoi usare.
3. L'agente parte subito, nella cartella del progetto, in una scheda del pannello dei terminali dentro Atollo (su ogni sistema).

Gli stessi pulsanti sono nella riga del progetto (barra laterale o «Pannello rapido»): un clic sul logo apre quell'agente nel progetto. Un agente non installato non ha il suo pulsante; se non ne è installato nessuno, non ci sono pulsanti degli agenti.

<details>
<summary>Sotto il cofano</summary>

- L'ultimo agente aperto viene annotato nel file dev-settings.json del progetto.
- Prima dell'avvio il kit installa ciò che manca (l'agente, le sue dipendenze, i programmi dei server MCP come Node.js o uv), applica le versioni del progetto e allinea istruzioni, skill e server MCP. Quando l'agente termina, li allinea di nuovo.
- Antigravity parte attraverso un piccolo programma del kit (agy) che rimette il tuo accesso nel sistema prima dell'avvio e lo riporta nel kit alla fine.

</details>

#### La pagina Agenti

A sinistra il riquadro «Installati» elenca gli agenti con la versione, lo stato dell'accesso, «in esecuzione» quando sono aperti e il limite più alto raggiunto (per esempio «5 ore 42%»). A destra, per l'agente scelto, tre parti:

| Parte | Che cosa contiene |
| --- | --- |
| Limiti e consumi | [4.3 Limiti e consumi degli agenti](#43-limiti-e-consumi-degli-agenti) |
| Accesso | Lo stato dell'accesso, «Esci» e, su macOS per Claude Code, «Collega con un token». |
| Condiviso fra gli agenti | [4.2 Istruzioni, skill e server MCP condivisi](#42-istruzioni-skill-e-server-mcp-condivisi) |

In cima alla barra laterale: «Aggiorna i consumi», «Apri la cartella condivisa» (`data\agents`) e il libro della guida. Se non c'è nessun agente: «Nessun agente installato: aggiungili da Strumenti.»

#### L'accesso al tuo account

| Testo | Significato |
| --- | --- |
| accesso non fatto: si fa al primo avvio dell'agente | Apri l'agente in un progetto e segui le sue istruzioni (di solito una pagina nel browser). |
| accesso fatto | L'agente ha un accesso con il tuo account, salvato nel kit. |
| accesso con chiave API | Codex con la chiave salvata da codex login --with-api-key, Antigravity con la chiave di Gemini delle Impostazioni, o Pi con una chiave API salvata nel suo auth.json: niente limiti di piano, paghi a consumo. |
| accesso con un token | Solo Claude Code su macOS, collegato con un token (vedi sotto). |
| modelli locali del kit, senza accesso | Solo Pi: nessun provider in cloud, usa un modello scaricato nella pagina «Modelli locali». Non c'è niente da cui uscire. |

L'accesso resta nella cartella del kit: portando il kit su un altro PC non devi rifarlo. «Esci» toglie l'accesso salvato nel kit dopo una domanda («Togliere l'accesso salvato nel kit?», Esc annulla), con l'agente chiuso: mentre è aperto il pulsante dice «Chiudi prima l'agente». Con una chiave API «Esci» non c'è.

> [!WARNING]
> **Se perdi la chiavetta**
>
> Gli accessi degli agenti e di GitHub sono salvati nel kit, in chiaro. «Esci» non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account di Anthropic, OpenAI, Google e GitHub (e del provider che usi con Pi).

> [!NOTE]
> **Claude Code su macOS**
>
> Su macOS Claude Code salverebbe l'accesso nel Portachiavi, che resta su quel Mac. «Collega con un token» apre claude setup-token nel terminale dell'app: completi l'accesso nel browser e il token (valido un anno) viene salvato nel kit, così funziona su ogni Mac. Se non viene riconosciuto, incollalo nel campo «Token di Claude Code» e premi «Salva il token». Il token arriva, come CLAUDE_CODE_OAUTH_TOKEN, solo a Claude Code avviato da Atollo: non alle azioni dei progetti, agli script, ai terminali né all'editor.

<details>
<summary>Sotto il cofano</summary>

| Agente | Dove sta l'accesso |
| --- | --- |
| Claude Code | `data\home\.claude\.credentials.json` |
| Codex | `data\home\.codex\auth.json` |
| Antigravity | `data\antigravity\keyring.json` |
| Pi (provider in cloud; «Esci» lo toglie) | `data\home\.pi\agent\auth.json` |
| Claude Code, token su macOS | `data\agents\claude\oauth-token` |

Antigravity salva il suo accesso solo nel portachiavi del sistema (Gestione credenziali su Windows, Portachiavi su macOS, Secret Service su Linux). Il kit ce lo mette prima dell'avvio e lo riporta in keyring.json alla fine, rimettendo a posto le eventuali credenziali di Antigravity del PC (conservate, cifrate, in keyring-before.json). Su Windows l'accesso nel sistema vive solo per la sessione di Windows.

</details>

Argomenti collegati: [4.2 Istruzioni, skill e server MCP condivisi](#42-istruzioni-skill-e-server-mcp-condivisi), [4.3 Limiti e consumi degli agenti](#43-limiti-e-consumi-degli-agenti), [4.4 Modelli locali: llama.cpp e i GGUF](#44-modelli-locali-llamacpp-e-i-gguf), [6.2 Le Impostazioni, una per una](#62-le-impostazioni-una-per-una)

### 4.2 Istruzioni, skill e server MCP condivisi

*Un'unica cartella per istruire gli agenti insieme: AGENTS.md, le skill, i server MCP del kit e quelli di un progetto.*

> **In breve**
>
> - Istruzioni, skill e server MCP si scrivono una volta sola, nella cartella `data\agents`, e valgono per tutti gli agenti.
> - Il kit li allinea all'avvio dell'app, prima e dopo ogni agente e quando un agente termina.
> - Puoi modificare anche i file di un agente: la modifica passa agli altri.
> - I server MCP di un solo progetto si aggiungono nella sezione «Editor» del progetto.

#### Che cosa si condivide

| Che cosa | A che cosa serve | Il file da modificare |
| --- | --- | --- |
| Istruzioni globali | Regole e preferenze che valgono in ogni progetto («rispondi in italiano», «usa pnpm», il tuo stile di codice). | `data\agents\AGENTS.md` |
| Skill | Procedure riutilizzabili che l'agente carica quando servono, una cartella per skill con un file SKILL.md. | `data\agents\skills\<nome>\SKILL.md` |
| Server MCP | Programmi o servizi che danno all'agente nuovi strumenti (leggere una documentazione, interrogare un servizio, usare il browser). | `data\agents\mcp.json` |

1. Nella pagina Agenti premi «Apri la cartella condivisa».
2. Modifica AGENTS.md con un editor di testo, aggiungi cartelle in skills o server in mcp.json.
3. La modifica arriva agli agenti al loro prossimo avvio (o subito, se riavvii l'app).

**Esempio di mcp.json** `data\agents\mcp.json`

```
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
    },
    "docs": {
      "type": "http",
      "url": "https://example.com/mcp"
    }
  }
}
```

> [!TIP]
> **Consiglio**
>
> I programmi dei server MCP si installano da soli all'avvio di un agente: npx e node portano Node.js, uvx e uv portano uv, python porta Python.

#### Come avviene l'allineamento

- Istruzioni: AGENTS.md viene copiato come CLAUDE.md per Claude Code e come AGENTS.md per Codex, Antigravity e Pi. Se modifichi uno di quei file, la modifica torna in AGENTS.md e passa agli altri. Se cambiano entrambe le parti, vince la più recente e l'altra viene conservata nella cartella dei conflitti.
- Skill: ogni agente vede la cartella skills attraverso un collegamento (copie sui dischi FAT32 ed exFAT). Una skill creata dentro un agente viene spostata in `data\agents\skills` e diventa di tutti; una skill cancellata in un agente viene messa da parte e si può recuperare.
- Server MCP: mcp.json viene tradotto nel formato di ciascun agente. Con Claude Code vale in entrambe le direzioni: un server aggiunto con claude mcp add -s user torna in mcp.json. Codex, Antigravity e Pi ricevono i server del kit e lasciano stare quelli che hai aggiunto a mano nelle loro configurazioni. Pi non accetta i server SSE: il kit li lascia fuori dal suo mcp.json, con un avviso.

> [!NOTE]
> **La regola del kit**
>
> A ogni allineamento il kit dà agli agenti una regola comune: nei progetti i file CLAUDE.md, AGENTS.md e GEMINI.md sono equivalenti, e le skill di un progetto stanno nella sua cartella .agents/skills. Per Pi la regola sta in un blocco marcato di APPEND_SYSTEM.md, riscritto a ogni allineamento: il testo che ci scrivi tu, fuori dal blocco, resta.

#### Server MCP di un solo progetto

1. Apri il progetto, sezione «Editor», riquadro «Server MCP del progetto».
2. Premi «Aggiungi un server».
3. Scrivi il «Nome» (lettere, cifre, - e _) e scegli il «Tipo»: «programma» (con «Programma», per esempio npx, e gli «Argomenti, uno per riga») oppure «indirizzo (http)» (con l'«Indirizzo»).
4. Premi «Aggiungi». «Togli» su una riga lo rimuove.

Questi server valgono solo in quel progetto, oltre a quelli del kit. Claude Code li legge dal file .mcp.json del progetto (e chiede conferma al primo uso); Codex da `.codex\config.toml`, che Atollo tiene allineato; Antigravity e Pi usano solo i server del kit.

<details>
<summary>Sotto il cofano</summary>

| Originale | Copie per gli agenti |
| --- | --- |
| `data\agents\AGENTS.md` | `data\home\.claude\CLAUDE.md · data\home\.codex\AGENTS.md · data\home\.gemini\AGENTS.md · data\home\.pi\agent\AGENTS.md` |
| `data\agents\skills` | `data\home\.claude\skills · data\home\.codex\skills · data\home\.gemini\antigravity-cli\skills · data\home\.pi\agent\skills` |
| `data\agents\mcp.json` | `data\home\.claude\.claude.json · data\home\.codex\config.toml · data\home\.gemini\config\mcp_config.json · data\home\.pi\agent\mcp.json` |

- Lo stato dell'allineamento, i conflitti e le skill cancellate sono in `data\agents\.sync` (conflicts, deleted).
- Gli avvisi dell'allineamento finiscono nel log dell'app, `data\app\logs\app.log`.
- L'allineamento si può lanciare anche dalla riga di comando: `Atollo.exe sync [--project <cartella>] [--verbose]`.
- Claude Code, Codex e Pi considerano attendibile la cartella di un progetto solo dopo che vi hai aperto un agente da Atollo: da lì possono eseguire comandi senza chiedere la fiducia ogni volta. Le altre cartelle di projects non sono attendibili in anticipo.
- In Codex le variabili ${VAR} non vengono espanse: il kit le traduce dove Codex ha una chiave apposita, altrimenti le copia così come sono con un commento.
- Pi tiene tutto in `data\home\.pi\agent` (la variabile PI_CODING_AGENT_DIR). Oltre a istruzioni, skill e server MCP, il kit vi scrive models.json (il provider «atollo»: i modelli locali su disco, con la porta e la chiave del server, solo per il proprietario) e settings.json (su Windows il Git Bash del kit come shell, la fiducia nei progetti «ask» se non l'hai impostata, il modello locale come predefinito finché non scegli un altro provider). I tuoi provider e le tue impostazioni restano.

</details>

Argomenti collegati: [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi), [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni), [8.5 La riga di comando](#85-la-riga-di-comando)

### 4.3 Limiti e consumi degli agenti

*Quanto del piano hai usato, quando si azzera, e i token e le sessioni di oggi e degli ultimi sette giorni.*

> **In breve**
>
> - «Limiti del piano» mostra quanta parte del tuo piano hai usato in ogni finestra e quando si azzera.
> - «Consumi» mostra token e sessioni di oggi e degli ultimi sette giorni, con un grafico per giorno.
> - I limiti si leggono dai server del fornitore ogni due minuti; «Aggiorna i consumi» li rilegge subito.

#### Limiti del piano

Per ogni finestra di limite c'è una casella con la percentuale usata, una barra e «si azzera …». Le finestre sono «5 ore» e «Settimana» per Claude Code, «… ore» e «Settimana» per Codex, e quote per modello per Antigravity. Sopra, il piano, la fonte dei dati e quando sono stati letti. Pi non ha finestre di limite: i modelli locali non ne hanno, e il kit non interroga i provider in cloud che Pi può usare.

| Colore | Quando |
| --- | --- |
| Verde | sotto il 70% |
| Arancione | dal 70% |
| Rosso | dal 90% |

Il valore più alto compare anche nella riga dell'agente nella barra laterale. Per Claude Code i crediti extra a pagamento sono mostrati a parte («Crediti extra: … di …»).

| Messaggio | Che cosa fare |
| --- | --- |
| accesso non fatto: apri … per vedere limiti e consumi | Apri l'agente in un progetto e fai l'accesso. |
| accesso scaduto: apri Claude Code per rinnovarlo | Apri l'agente: rinnova l'accesso da solo. |
| chiave API: nessun limite di piano | Con una chiave API non ci sono finestre di limite. |
| Modelli locali: nessun limite di piano | Pi sui modelli locali del kit: girano sul tuo PC, senza piano. I consumi qui sotto ci sono comunque. |
| I limiti dei provider di Pi non sono letti dal kit | Pi con un provider in cloud: i limiti si vedono nel sito del provider o in Pi. |
| nessun dato ancora: i limiti arrivano con la prossima risposta di Codex | Usa Codex una volta: i limiti arrivano con la risposta. |
| Il server chiede di aspettare: nuova lettura … | Il fornitore ha limitato le letture: il kit riprova da solo più tardi. |

#### Consumi

- «Token · Oggi» con i token di oggi e degli ultimi 7 giorni; passandoci sopra, quanti sono nuovi, quanti presi dalla cache e quanti in uscita.
- «Sessioni · Oggi» con le sessioni di oggi e degli ultimi 7 giorni.
- Un grafico a barre per i 7 giorni, l'ultima attività e i tre modelli più usati.
- Antigravity non salva i token: per lui si contano solo le sessioni.
- Per Pi i token si leggono dalle sue sessioni, anche quelle sui modelli locali.

> [!NOTE]
> **Da sapere**
>
> I consumi si calcolano dai file delle sessioni salvati nel kit, quindi comprendono le sessioni aperte da questo kit su qualsiasi PC. Non ci sono costi in denaro: solo token e sessioni.

<details>
<summary>Sotto il cofano</summary>

- Fonti dei limiti: per Claude Code il server di Anthropic, per Codex quello di OpenAI (o, in mancanza, l'ultima risposta di Codex), per Antigravity quello di Google. Il kit usa l'accesso salvato, solo per leggere: non lo rinnova e non lo manda altrove.
- Letture al massimo ogni 2 minuti, ogni 20 secondi con «Aggiorna i consumi»; dopo una risposta «troppe richieste» aspetta quanto chiesto dal server (da 2 a 30 minuti).
- File delle sessioni: `data\home\.claude\projects`, `data\home\.codex\sessions`, `data\home\.gemini` (conversazioni di Antigravity), `data\home\.pi\agent\sessions` (Pi).

</details>

Argomenti collegati: [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi), [4.2 Istruzioni, skill e server MCP condivisi](#42-istruzioni-skill-e-server-mcp-condivisi), [4.4 Modelli locali: llama.cpp e i GGUF](#44-modelli-locali-llamacpp-e-i-gguf)

### 4.4 Modelli locali: llama.cpp e i GGUF

*Modelli linguistici sul PC, senza rete e senza account: il server, la chat nel browser, il modello di Pi, scaricare dal catalogo o da Hugging Face.*

> **In breve**
>
> - Un modello locale è un modello linguistico che gira sul tuo PC: niente rete, niente account, niente piano. Va sulla CPU o sulla scheda video (Vulkan su Windows e Linux, Metal su macOS).
> - Il server è llama.cpp, i modelli sono file GGUF: si scaricano dalla pagina «Modelli locali», dal catalogo di Atollo o da Hugging Face.
> - Li usano Pi e una chat nel browser. Il server ascolta solo su questo PC, con una chiave.
> - Un download sospeso o interrotto riprende da dove si era fermato, anche dopo aver chiuso Atollo.

#### Che cosa sono e che cosa serve

Un modello GGUF è un unico file (a volte diviso in parti) con i pesi del modello già compressi: la «quantizzazione» (Q4_K_M, Q8_0, MXFP4…) dice quanto, e più è bassa più il file è piccolo e meno è preciso. Il server llama.cpp carica il modello nella memoria della scheda video, se ce n'è abbastanza, altrimenti lo divide fra scheda video e RAM o usa solo la CPU. La scelta è automatica: llama.cpp si installa la prima volta che avvii il server, una sola versione per sistema, verificata con il suo SHA-256.

I modelli occupano molti GB e chiedono memoria: un modello da 7-9 miliardi di parametri sta in circa 8 GB, uno da 30 miliardi ne vuole più di 20. Sono meno capaci dei modelli in cloud di Claude Code o Codex, ma restano sul tuo PC.

#### La pagina «Modelli locali»

Si apre dalla barra delle attività (l'icona del chip). A sinistra, «Nel kit»: il server llama.cpp con il suo stato e la porta, e i modelli scaricati («modello di Pi», «in memoria», «scaricamento sospeso»). In alto, «Avvia» o «Ferma», «Apri la chat» e «Copia la chiave». Sotto, le sezioni:

| Sezione | Che cosa contiene |
| --- | --- |
| Server | L'indirizzo (http://127.0.0.1:\<porta\>), i modelli in memoria e «Avvia con Atollo», per accendere il server all'apertura dell'app. «Avvia» resta spento finché non c'è almeno un modello scaricato. |
| Modello di Pi | «Pi parte con»: un modello scaricato, oppure «il primo modello del kit». Con un provider in cloud scelto in Pi (/login), Pi resta su quello; i modelli locali restano nel suo elenco (/model). |
| Catalogo di Atollo | I modelli scelti per il kit, ognuno con quantizzazione, dimensione, licenza, la memoria che chiede, se entra in questo PC e se usa gli strumenti. «Scarica», poi «Sospendi» o «Riprendi»; il cestino lo cancella. |
| Altri modelli nel kit | I modelli presi da Hugging Face e i file .gguf messi a mano nella cartella dei modelli. |
| Da Hugging Face | Un repository qualsiasi (proprietario/nome), la scelta della quantizzazione e il token per i modelli ad accesso controllato. |
| Questo PC | La RAM, le schede video con la loro memoria (si vedono dopo l'installazione di llama.cpp), il file system del disco del kit con lo spazio libero, e la cartella dei modelli. |

#### Scegliere e scaricare un modello

Accanto a ogni modello la pagina dice se entra nel PC, confrontando la memoria che chiede con la più grande fra la RAM e la memoria della scheda video:

| Testo | Significato |
| --- | --- |
| entra | Chiede al massimo l'80% di quella memoria: c'è margine. |
| al limite | Ci sta, ma lascia poco al resto del sistema: può essere lento. |
| troppo grande per questo PC | Chiede più memoria di quella che il PC ha. |

«usa gli strumenti (adatto agli agenti)» vuol dire che il modello sa chiamare i comandi e leggere i file quando lavora con Pi; «solo chat» vuol dire che con gli agenti non li usa bene (scrive la chiamata come testo), ma va bene per chiacchierare nel browser.

| Modello del catalogo | Memoria | Uso |
| --- | --- | --- |
| Qwen2.5-Coder 7B | circa 7 GB | solo chat; in due parti, entra anche su FAT32 |
| Qwen3.5 9B | circa 8 GB | usa gli strumenti |
| gpt-oss 20B | circa 15 GB | usa gli strumenti |
| Devstral Small 2 24B | circa 18 GB | usa gli strumenti |
| Qwen3-Coder 30B-A3B | circa 22 GB | usa gli strumenti |

1. Nel «Catalogo di Atollo» scegli un modello che «entra» e «usa gli strumenti», se lo vuoi per Pi.
2. Premi «Scarica». La barra mostra percentuale e GB; il download continua anche se cambi pagina.
3. «Sospendi» lo ferma e tiene la parte già scaricata; «Riprendi» riparte da lì, anche dopo aver chiuso Atollo o dopo un errore di rete.
4. Alla fine il modello compare in «Nel kit» con «nel kit»; se il server è acceso, si riavvia per vederlo.

> [!NOTE]
> **Controlli**
>
> Ogni file del catalogo ha il suo SHA-256: il kit lo calcola mentre i byte arrivano e cancella un file che non corrisponde. Prima di scrivere un byte, rifiuta un modello più grande dello spazio libero e, su un disco FAT32, un file oltre i 4 GB, dicendo che cosa fare.

> [!WARNING]
> **Dischi FAT32**
>
> FAT32 non tiene file oltre i 4 GB: la pagina lo avvisa e scrive «non entra su FAT32» accanto ai modelli troppo grandi. Qwen2.5-Coder 7B, in due parti più piccole, entra comunque. Per tutti gli altri formatta il disco in exFAT o NTFS.

#### Da Hugging Face

1. In «Repository di Hugging Face» scrivi proprietario/nome del repository, per esempio unsloth/Qwen3-8B-GGUF.
2. Premi «Cerca i file GGUF»: il kit legge l'ultimo commit e raggruppa i file per quantizzazione, con dimensione, numero di parti, se entra nel PC e la licenza.
3. Premi «Scarica» sulla quantizzazione che vuoi: il kit scarica tutte le sue parti da quel commit, o nessuna. Q4_K_M è di solito un buon compromesso.

Alcuni modelli sono ad accesso controllato: si accettano le condizioni sulla pagina del modello e serve un token di Hugging Face. Scrivilo in «Token di Hugging Face» e premi «Salva il token»; «Togli il token» lo cancella. Il token è salvato nel kit, leggibile solo dal tuo utente, e va solo a huggingface.co: non segue il reindirizzamento verso la CDN da cui arrivano i file.

> [!TIP]
> **Consiglio**
>
> Un file .gguf copiato a mano in una cartella di `data\models\gguf` è un modello anche lui: compare in «Altri modelli nel kit» con il nome del file.

#### La chat nel browser

1. Premi «Avvia» (o lascia fare a «Avvia con Atollo»). La prima volta il kit installa llama.cpp.
2. Premi «Copia la chiave»: la chiave del server va negli appunti.
3. Premi «Apri la chat»: si apre la chat di llama.cpp nel browser. Nelle sue impostazioni incolla la chiave, poi scegli il modello e scrivi.

Pi non ha bisogno di questi passi: il kit gli scrive porta e chiave nella sua configurazione e accende il server prima di aprirlo, quando usa un modello locale.

#### Sicurezza e risorse

- Il server ascolta solo su 127.0.0.1: nessun altro PC della rete lo raggiunge, anche con la rete locale attiva nelle Impostazioni.
- Ogni richiesta vuole la chiave del server, creata al primo avvio e data al server come file, mai sulla riga di comando (dove altri programmi del PC la vedrebbero). Senza chiave il server risponde 401.
- Un modello si carica quando Pi o la chat lo chiede, uno alla volta, e si libera dopo dieci minuti senza richieste: la memoria torna al PC.
- Il server compare nel pannello «Attività» (con «Ferma» e i modelli in memoria) e si ferma con «Ferma tutto», con «Rimuovi l'unità» e alla chiusura dell'app.
- Mentre cancelli un modello il server si ferma: su Windows un file aperto non si può cancellare.

<details>
<summary>Sotto il cofano</summary>

| File o cartella | Che cosa contiene |
| --- | --- |
| `data\models\gguf\<proprietario>--<nome>\` | I modelli, una cartella per repository, una sola copia per Windows, macOS e Linux. atollo.json dice che cosa ha scaricato il kit (file, commit, se è completo); un download a metà lascia un file .part. |
| `data\app\secrets\llama-key` | La chiave del server, solo per il proprietario. |
| `data\app\secrets\hf-token` | Il token di Hugging Face, solo per il proprietario. |
| `data\app\llm.json` | La porta del server (9931 o la prima libera dopo), «Avvia con Atollo» e il modello di Pi. |
| `data\app\llm\presets.ini` | I modelli che il server offre, con i percorsi assoluti: si riscrive a ogni avvio, perché il kit può essersi spostato. |
| `data\app\logs\llm.log` | L'uscita del server, da leggere se non parte. |

- llama.cpp è fissato a una build (b11669) con lo SHA-256 dell'archivio di ogni sistema: la build Vulkan su Windows e Linux (che contiene anche i motori per la CPU), quella Metal su macOS.
- I modelli del catalogo sono fissati a un commit di Hugging Face, ogni file con SHA-256 e dimensione; quelli presi da Hugging Face si scaricano dall'ultimo commit al momento della ricerca.
- Dalla riga di comando: `models` elenca i modelli, `models pull <modello>` ne scarica uno (un id del catalogo o proprietario/nome:quantizzazione, riprende se interrotto), `models rm <id>` lo cancella, `llm start|stop|status` comanda il server.

</details>

Argomenti collegati: [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi), [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [8.5 La riga di comando](#85-la-riga-di-comando), [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza)

## 5. Strumenti e linguaggi

### 5.1 La pagina Strumenti

*La tabella di tutti gli strumenti divisa negli otto tipi, i cinque stati, installare e aggiornare subito o scegliere e poi «Applica».*

> **In breve**
>
> - La pagina Strumenti elenca tutti i componenti che il kit sa installare, divisi in otto tipi come nel sito: sistema ed editor, agenti AI, modelli locali, linguaggi, strumenti di sviluppo, server web, database, visualizzazione database.
> - Le icone delle righe agiscono subito: «Installa» e «Aggiorna» partono appena le premi.
> - Le caselle invece preparano una scelta, che si conferma con «Applica»: così si installano e tolgono più cose insieme.
> - Ogni riga mostra lo stato su disco con un'icona: non esiste, pre-installato, installato, da aggiornare, errore.

![La tabella «Tutti gli strumenti»](../../ui/public/guide/it/tools.webp)

*Filtri in alto, una riga per componente con stato, versioni e azioni; in fondo la barra con «Aggiorna tutto» e «Applica».*

#### Com'è fatta la pagina

| Parte | Che cosa contiene |
| --- | --- |
| Cerca negli strumenti | Cerca nel nome, nella descrizione e nell'identificativo. |
| Riquadro «Componenti» | Un albero con gli otto tipi e i loro componenti, con quanti sono installati. Un clic apre la scheda del componente. |
| Riquadro «Panoramica» | Le viste d'insieme: «Tutti gli strumenti», «Matrice», «Estensioni dell'editor», «Relazioni», «Requisiti del PC» ([5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti)). |
| Scheda «Tutti gli strumenti» | La prima scheda, sempre aperta: la tabella di tutti i componenti. |
| Barra «Applica» | In fondo, sempre visibile: le modifiche in attesa, «Aggiorna tutto», «Applica». |

| Tipo | Che cosa contiene |
| --- | --- |
| Sistema ed editor | Editor, PowerShell 7, GitHub CLI, Git: sempre installati. |
| Agenti AI | Claude Code, Codex, Antigravity, Pi: facoltativi, sempre all'ultima versione. |
| Modelli locali | llama.cpp, il server dei modelli linguistici sul PC: li usano Pi e la chat nel browser, senza rete. Facoltativo, a una versione fissata; i modelli si scaricano dalla pagina Modelli locali ([4.4 Modelli locali: llama.cpp e i GGUF](#44-modelli-locali-llamacpp-e-i-gguf)). |
| Linguaggi | Tutti i linguaggi e i runtime: Python, Node.js, PHP, Go, Java, Rust, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, C/C++, Ruby, R. Tutti possono avere più versioni una accanto all'altra, tranne Rust e C/C++, che restano all'ultima versione. |
| Strumenti di sviluppo | uv, pnpm/yarn, CMake/Ninja, Android SDK, Typst, TinyTeX: una sola versione, sempre l'ultima (solo Typst può tenerne più d'una); quelli che servono si installano da soli. |
| Server web | Apache e Caddy, per i siti dei progetti e i Server Web condivisi. |
| Database | I server: MySQL e MongoDB (più versioni), SQLite. |
| Visualizzazione database | I database nel browser: phpMyAdmin per MySQL, mongo-express per MongoDB. Si aprono dalla pagina Database, che li installa al primo uso. |

#### La tabella «Tutti gli strumenti»

Sopra la tabella ci sono i filtri: la ricerca, «Piattaforma», «Tipo» («Tutti i tipi» o uno degli otto) e «Stato» (Tutti, Installati, Non installati, Da applicare, Usati dai progetti), più i pulsanti «Espandi» e «Verifica». Le colonne: nome, versione, versioni su disco, dimensione, stato, progetti che lo usano, che cosa richiede e le azioni.

Le righe sono divise per tipo. Ogni tipo ha una riga di titolo: la freccia per chiuderlo o riaprirlo, il nome, quanti componenti contiene («3», oppure «2 di 5» quando un filtro ne nasconde qualcuno), la sua descrizione e una casella «Scegli tutti: …» che sceglie tutti i suoi componenti che si possono scegliere; la casella nell'intestazione, «Scegli tutti quelli mostrati», fa lo stesso con tutte le righe visibili. Un clic sull'intestazione di una colonna ordina le righe dentro ogni tipo (all'inizio per nome).

| Stato | Icona | Significato |
| --- | --- | --- |
| non esiste | cerchio vuoto | Non c'è niente su disco. Il suggerimento dice perché (non scelto, non disponibile su quel sistema…). |
| pre-installato | pacchetto verde | Gli archivi sono già nel kit e l'installazione si completa al primo avvio su quel sistema. |
| installato | cerchio verde con la spunta | Pronto all'uso. |
| da aggiornare | triangolo giallo | Va reinstallato perché il kit è stato spostato e quel componente non lo segue (succede solo su macOS e Linux). Non vuol dire che esista una versione nuova. |
| errore | cerchio rosso con la X | L'ultima installazione non è riuscita, oppure su disco manca una parte. Il suggerimento dice che cosa. |

Durante il lavoro compaiono altri quattro stati, solo sul sistema mostrato: «in corso» (con la percentuale del download), «in coda», «da installare» e «da rimuovere» (scelte non ancora applicate).

> [!TIP]
> **Consiglio**
>
> Passa il mouse sull'icona di stato: il suggerimento dice componente, sistema, stato e il dettaglio (per esempio «download 42%» o «in uso: chiudi i programmi del kit e riprova»).

#### Agire subito o scegliere e applicare

| Comando | Dove | Che cosa fa |
| --- | --- | --- |
| Installa (nuvola) | Riga di un componente non installato | Lo installa subito, all'ultima versione. |
| Installa una versione (+) | Riga di un linguaggio non installato | Lo installa subito nella versione che scegli dal menu. |
| Aggiorna | Riga di un componente installato | Scarica subito la versione nuova, se c'è. |
| Aggiungi una versione (+) | Riga di un linguaggio installato | Scarica un'altra versione accanto a quelle che hai. |
| Casella | Ogni riga tranne quelle sempre installate; «Scegli tutti: …» nella riga del tipo | Segna il componente da installare o da togliere, in attesa di «Applica». |
| Applica | Barra in fondo | Mostra il riepilogo e, con «Conferma», installa e toglie ciò che hai segnato. |
| Aggiorna tutto | Barra in fondo | Porta ogni componente installato all'ultima versione e ogni versione all'ultima correzione. |

1. Spunta i componenti da aggiungere e togli la spunta a quelli da rimuovere. Le righe cambiate si colorano.
2. In fondo la barra dice per esempio «2 da installare (180 MB), 1 da rimuovere».
3. Premi «Applica»: compare «Applicare le modifiche?» con l'elenco di ciò che si installa e si toglie. «Indietro» o Esc chiudono la domanda senza cambiare niente.
4. Premi «Conferma». Prima si tolgono, poi si installano; lo stato scorre nella barra («Installazione 1/3: …»). «Annulla» ferma alla fine del passo in corso.

> [!WARNING]
> **Attenzione**
>
> «Aggiorna tutto» applica anche le scelte in attesa, senza la finestra di conferma. Se hai caselle cambiate che non vuoi applicare, rimettile com'erano prima.

- Togliere un componente è possibile solo con la casella e «Applica», mai con un'icona: è un'operazione che vuole conferma.
- Le dipendenze seguono in entrambi i sensi: scegliere Kotlin sceglie Java, togliere Java toglie Kotlin e Android. Le etichette delle righe lo spiegano («Richiede Java: si installa insieme», «Serve a Kotlin: resta installato finché serve»).
- Un componente richiesto da un progetto ma non scelto ha un triangolo e l'etichetta «Serve ai progetti: … Non è scelto: sceglilo perché funzionino.»
- Un componente cancellato a mano dal disco viene tolto dalla scelta all'apertura della pagina, con un avviso.

#### La scheda di un componente

Un clic sul nome apre la scheda. In alto, come la pagina di un'estensione dell'editor: il logo grande, il nome, tipo, stato e dimensione, la descrizione e i pulsanti «Installa»/«Mantieni» e «Rimuovi»/«Non installare» (scelte da applicare), più «Sito ufficiale». Sotto, una barra con tre sezioni, una alla volta:

| Sezione | Che cosa contiene |
| --- | --- |
| Versioni | Solo per i componenti con più versioni (linguaggi, server web, database): le versioni su disco e quelle da scaricare ([5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento)). |
| Dettagli | Tipo, versione, dimensione, che cosa richiede, a chi serve, sito. |
| Progetti | I progetti che lo usano e, per i linguaggi, la versione scelta da ciascuno. |

#### «Piattaforma», «Espandi» e «Verifica»

- «Piattaforma» sceglie per quale sistema guardare (e preparare) i componenti: Windows x64 o ARM64, macOS Apple Silicon o Intel, Linux x64 o ARM64. Lo stesso kit su una chiavetta si apre su tutti e sei: da qui puoi preparare i programmi per un altro sistema prima di portarci la chiavetta.
- «Espandi» aggiunge una colonna di stato per ognuno dei sei sistemi; «Riduci» torna a quella del sistema scelto.
- «Verifica» chiede a ogni componente installato di rispondere con la sua versione, e aggiunge un segno ✓ o ⚠ nella cella dello stato.

<details>
<summary>Sotto il cofano</summary>

- La scelta è in `data\app\components.txt`, una per tutti i sistemi.
- I programmi sono in `data\apps\<sistema>\<componente>`; la versione installata in `data\app\versions\<sistema>\<componente>.txt`.
- Un componente tolto viene tolto anche dalle cartelle degli altri sistemi che lo hanno su disco.
- Lo stato «da aggiornare» corrisponde a un file \<componente\>.reinstall, scritto quando il kit cambia cartella e un ambiente conda (R, Ruby, MongoDB e SQLite su Linux, make) non può seguirlo.

</details>

Argomenti collegati: [5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento), [5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti), [8.4 Da dove arrivano gli strumenti](#84-da-dove-arrivano-gli-strumenti)

### 5.2 Più versioni dello stesso strumento

*Tenere PHP 8.3 e 8.4 o Node 20 e 24 una accanto all'altra: aggiungere, rimuovere, versioni bloccate e quale usa il kit.*

> **In breve**
>
> - I linguaggi e i server possono avere più versioni installate insieme, una per versione minore (8.3, 8.4…).
> - Il kit usa sempre la più recente su disco; ogni progetto può sceglierne un'altra nella sua pagina.
> - Una versione usata da un progetto è «bloccata»: non si può togliere finché il progetto la usa.
> - «Aggiorna tutto» porta ogni versione all'ultima correzione (8.4.1 → 8.4.3) senza toglierne nessuna.

#### Le versioni di un componente

Nella sezione «Versioni» della scheda (o sotto la riga, nella tabella) c'è una riga per ogni versione su disco, per esempio «8.4 (8.4.3)», con delle etichette:

| Etichetta | Significato |
| --- | --- |
| più recente | È la versione più nuova offerta dalla fonte. |
| bloccata (lucchetto) | Dei progetti la usano: il menu «Usata da» dice quali. Non si può rimuovere. |
| non per … | Un altro componente non è compatibile con questa versione (per esempio un'estensione che non la supporta). |
| cestino «Rimuovi …» | Toglie quella versione. Compare solo se ce ne sono almeno due: l'ultima si toglie con la casella del componente. |

1. Per aggiungere una versione premi + «Aggiungi una versione di …»: il menu «Versioni da scaricare» le raggruppa (8.x, 7.x…).
2. Scegli la versione: il download parte e la riga compare in grigio finché non è pronta.
3. Se l'elenco sembra vecchio, «Cerca altre versioni di …» lo chiede di nuovo alla fonte.

> [!NOTE]
> **Quale versione usa il kit**
>
> Non c'è una versione «predefinita» da scegliere: il kit usa la più recente che ha su disco. Un progetto nuovo fissa quella che c'è alla sua creazione; per cambiarla vai nella pagina del progetto, «Componenti» › «Versioni» ([2.4 Componenti e versioni di un progetto](#24-componenti-e-versioni-di-un-progetto)).

#### Casi particolari

- Composer segue PHP: c'è una copia per ogni versione di PHP, compatibile con quella; si installa, si aggiorna e si toglie con PHP.
- Maven e Gradle seguono Java e compaiono come etichette sulla riga della versione.
- MySQL e MongoDB: ogni versione ha i suoi dati e la sua porta, che resta sua anche quando cambi la versione del server; vedi la pagina Database.
- Python e PHP: una nuova installazione prende la linea più recente; le altre linee si scaricano da questo menu e ogni progetto sceglie la sua. Le Impostazioni non hanno più una scelta di versione (i valori pythonVersion e phpVersion di un vecchio settings.json vengono ignorati).
- Rust, solo su Windows: nella sua pagina, sezione «Dettagli», «Compilatore di Rust» sceglie GNU (predefinito, tutto nel kit) oppure MSVC, che richiede Visual Studio Build Tools nel PC. Vale dalla prossima installazione o aggiornamento di Rust. Su macOS e Linux Rust ha un solo compilatore e la voce non compare.

<details>
<summary>Sotto il cofano</summary>

- Ogni versione vive nella sua cartella: `data\apps\<sistema>\<componente>\<versione minore>`, con la versione completa nel file .atollo-version.
- Gli elenchi delle versioni offerte sono in `data\app\version-lines.json`, aggiornati quando hanno più di 24 ore.
- Su Windows PHP offre anche le ultime build delle versioni non più supportate (8.1, 8.0, 7.4…), utili per progetti vecchi; su macOS e Linux solo quelle con una build fissata dal kit.

</details>

Argomenti collegati: [2.4 Componenti e versioni di un progetto](#24-componenti-e-versioni-di-un-progetto), [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [3.1 I server di database](#31-i-server-di-database)

### 5.3 Panoramiche: matrice, relazioni, estensioni, requisiti

*Le viste d'insieme della pagina Strumenti: i sei sistemi, chi richiede chi, le estensioni per linguaggio e ciò che serve al PC.*

> **In breve**
>
> - «Matrice»: dove può stare ogni componente, sui sei sistemi che il kit supporta.
> - «Relazioni»: chi richiede chi e con quali versioni, e le versioni chieste dai progetti.
> - «Estensioni dell'editor»: quali estensioni riceve ogni linguaggio.
> - «Requisiti del PC»: i componenti di sistema che servono al PC.

#### Matrice

Una tabella con un componente per riga, divisa negli stessi otto tipi della tabella «Tutti gli strumenti», e i sei sistemi in colonna. Ogni casella dice se il componente può stare su quel sistema: ✓ disponibile, ≈ in emulazione (la versione x64 su un PC ARM64), ↓ pre-installato, ◷ si completa al primo avvio, ✕ non disponibile; un punto verde vuol dire installato. Un clic su una casella mostra il motivo, la versione e il pulsante «Mostra questo sistema».

> [!NOTE]
> **Da sapere**
>
> La Matrice dice che cosa è possibile; la tabella «Tutti gli strumenti» dice che cosa c'è davvero su disco.

#### Relazioni

Due tabelle, ricalcolate quando cambia qualcosa (o con «Ricalcola le relazioni»):

| Tabella | Che cosa dice |
| --- | --- |
| Matrice delle relazioni | Per ogni strumento che cosa richiede, con quali versioni, e se va bene («sì», «no: il kit usa …», «manca …»). |
| Versioni chieste dai progetti | Per ogni progetto la versione che i suoi file chiedono, da quale file, e lo stato («usa …», «da scaricare: …»), con i pulsanti «Usa …» e «Scarica e usa …». |

#### Estensioni dell'editor

Una tabella in sola lettura: in riga le estensioni, in colonna i linguaggi e gli strumenti. ● vuol dire che l'estensione è attiva di default nei progetti con quel linguaggio, ○ che è facoltativa; ✓ che è installata. I linguaggi dei tuoi progetti sono in evidenza. Le estensioni si scelgono per progetto: [2.5 Editor, terminale ed estensioni](#25-editor-terminale-ed-estensioni).

#### Requisiti del PC

Alcuni componenti hanno bisogno di qualcosa del sistema: Visual C++ 2015-2026 per PHP, Apache e MongoDB su Windows, WebView2 per la finestra dell'app, l'emulazione x64 di Windows 11 sui PC ARM64. Ogni riga dice per quali componenti serve e se è «presente», se «manca» o se «non serve».

<details>
<summary>Sotto il cofano</summary>

- Le regole delle relazioni sono nel catalogo del kit (tauri/catalog/relations.json); il risultato in `data\app\relations.json`.
- File dei progetti che il kit legge per le versioni: `package.json, .nvmrc, .python-version, pyproject.toml, composer.json, go.mod, global.json, pom.xml, Gemfile, Cargo.toml` e altri.
- Il catalogo delle estensioni (da Open VSX) è tauri/catalog/extensions.json.

</details>

Argomenti collegati: [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [2.4 Componenti e versioni di un progetto](#24-componenti-e-versioni-di-un-progetto), [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella)

## 6. Account e impostazioni

### 6.1 L'account GitHub del kit

*Perché collegarlo, come si accede, che cosa sblocca (repository, Git, GitHub CLI) e che cosa funziona anche senza.*

> **In breve**
>
> - Un solo accesso a GitHub vale per tutto il kit: la pagina Progetti, Git (anche nell'editor, nelle console e negli agenti) e GitHub CLI.
> - Si fa in Impostazioni › «Account GitHub» con «Accedi a GitHub» e un codice da scrivere nel browser.
> - Senza account il kit funziona lo stesso: cloni i repository pubblici e lavori in locale.
> - L'accesso resta nel kit: su un altro PC sei già collegato.
> - Con l'accesso, nome ed e-mail dei commit si riempiono dall'account, con l'indirizzo noreply di GitHub; li cambi in «Chi firma i commit».

#### Che cosa sblocca

| Con l'account | Anche senza |
| --- | --- |
| L'elenco dei tuoi repository in Progetti, con clona, rinomina ed elimina | Clonare repository pubblici con «Da un indirizzo Git» |
| Clonare repository privati | Creare progetti, avviarli, database, siti, agenti, strumenti |
| git push e git pull verso GitHub senza password | Usare Git in locale (commit, rami) |
| I comandi gh nelle console, nell'editor e negli agenti | — |
| Più richieste all'API di GitHub per i download (5.000 l'ora invece di 60) | Scaricare i componenti (con il limite più basso e qualche ripiego) |

#### Accedere

1. Apri Impostazioni › «Account GitHub».
2. Premi «Accedi a GitHub». Dopo «Accesso in preparazione…» compare un codice di otto caratteri (XXXX-XXXX) e si apre github.com nel browser.
3. Premi «Copia il codice», incollalo nella pagina di GitHub e autorizza GitHub CLI. Se la pagina non si è aperta, «Apri la pagina di GitHub».
4. Torna su Atollo: compare «Accesso fatto come @tuo-nome».

«Esci» toglie l'accesso dal kit dopo la conferma «Uscire da GitHub? Anche Git, l'editor e gli agenti del kit perdono l'accesso.»; «Annulla» o Esc chiudono la domanda.

> [!WARNING]
> **Attenzione**
>
> Il token di GitHub è salvato nel kit, in chiaro, perché ti segua su ogni PC. «Esci» non revoca la sessione su github.com: se perdi la chiavetta, revoca l'autorizzazione di GitHub CLI dalle impostazioni del tuo account GitHub (Applications).

#### Il permesso di eliminare i repository

L'accesso normale non comprende il permesso di eliminare repository. La prima volta che premi «Elimina da GitHub» su un tuo repository, il riquadro dice che manca il permesso e mostra «Concedi il permesso»: il kit rifà l'accesso chiedendo anche quello. Il codice da scrivere compare in Impostazioni › «Account GitHub». Poi premi di nuovo «Elimina da GitHub».

> [!WARNING]
> **Attenzione**
>
> Eliminare un repository da GitHub non si può annullare: spariscono anche issue, pull request, wiki e release. La copia nel kit, se c'è, resta.

#### Il tuo nome nei commit

In Impostazioni › «Account GitHub», sotto l'account, «Chi firma i commit» ha due campi, «Nome» ed «E-mail»: sono il nome e l'e-mail dei commit fatti con il Git del kit. Ognuno si salva quando il campo perde il focus.

- Se sono vuoti tutti e due e il kit ha accesso a GitHub, Atollo li riempie da solo, all'accesso e a ogni avvio: il nome dell'account (o il login) e l'indirizzo privato di GitHub, id+login@users.noreply.github.com. Così la tua e-mail vera resta privata.
- Puoi scrivere altro nome ed e-mail quando vuoi. Se svuoti tutti e due, vengono tolti; al prossimo avvio, se il kit ha ancora accesso a GitHub, tornano quelli dell'account.
- Un valore che non va bene mostra «Nome non valido.» o «Indirizzo e-mail non valido.». Se Git non è ancora nel kit compare «Git non è ancora nel kit: si installa con GitHub CLI.».

Sono i valori user.name e user.email di Git: in un terminale del kit funziona anche git config --global.

**Nome ed email per i commit**

```
git config --global user.name "Nome Cognome"
git config --global user.email "tu@esempio.it"
```

<details>
<summary>Sotto il cofano</summary>

- L'accesso esegue gh auth login --web --insecure-storage --scopes workflow: oltre ai permessi standard di GitHub CLI (repo, read:org, gist) chiede workflow; delete_repo solo con «Concedi il permesso».
- Il token è in `data\gh\hosts.yml` (GH_CONFIG_DIR punta lì), mai nella Gestione credenziali di Windows.
- Git usa lo stesso account attraverso un credential helper (gh auth git-credential) scritto nel file `data\home\.gitconfig` per github.com e gist.github.com, senza percorsi assoluti: sopravvive allo spostamento del kit. A ogni avvio il kit lo allinea allo stato di gh; un helper scritto da te per quegli indirizzi resta com'è.
- «Chi firma i commit» legge e scrive git config --global user.name e user.email, cioè il file `data\home\.gitconfig` del kit. Per riempirli dall'account basta una chiamata a gh api user.
- Git si fida solo dei repository nella cartella del kit: il .gitconfig del kit ha safe.directory = \<cartella del kit\>/\* (per esempio D:/Atollo/\*), riscritto a ogni avvio perché segua il kit quando cambia cartella; i valori aggiunti da te restano. Prima era safe.directory = \*, cioè ogni repository del PC, anche di altri utenti.
- Per un repository fuori dal kit che appartiene a un altro utente, Git avvisa di nuovo «dubious ownership». Se ti fidi, in un terminale del kit: git config --global --add safe.directory \<cartella\>.
- Per clonare un repository privato il token viene passato a Git solo per quella operazione, mai sulla riga di comando né nel file `.git\config`.
- Per i download il kit usa prima GITHUB_TOKEN o GH_TOKEN del PC, se ci sono, poi l'account del kit. Le risposte dell'API sono conservate in `data\cache\github`.
- Il menu «Account» dell'editor non vede l'account del kit; push e pull nell'editor funzionano comunque, attraverso Git.

</details>

Argomenti collegati: [2.1 Creare un progetto](#21-creare-un-progetto), [2.6 Git di un progetto](#26-git-di-un-progetto), [8.4 Da dove arrivano gli strumenti](#84-da-dove-arrivano-gli-strumenti)

### 6.2 Le Impostazioni, una per una

*Salvataggio automatico e le sette sezioni: lingua e tema, file temporanei, rete, chiave di Gemini, aggiornamento automatico, chi firma i commit, privacy e le informazioni sul kit.*

> **In breve**
>
> - Le Impostazioni hanno sette sezioni nella barra laterale (Generale, Rete, Agenti, Aggiornamenti, Account GitHub, Privacy, Il kit) e, sotto, le istruzioni dei framework.
> - Non c'è un pulsante «Salva»: ogni modifica si salva da sola mezzo secondo dopo, e la barra delle schede mostra «Salvataggio…» e poi «Salvato.».
> - Valgono subito per l'app, le console e gli agenti; i terminali dell'editor dopo «Riavvia l'editor».

![La pagina Impostazioni](../../ui/public/guide/it/settings.webp)

*Le sezioni a sinistra, sotto le istruzioni dei framework; lo stato del salvataggio sulla barra delle schede.*

#### Come si salvano le modifiche

- Ogni modifica si salva e si applica da sola, circa mezzo secondo dopo l'ultima battuta: sulla barra delle schede compare «Salvataggio…», poi «Salvato.».
- Se un valore non va bene (per esempio un file PEM che non esiste), compare il suo errore e il valore resta sullo schermo: il kit riprova alla modifica successiva.
- Anche le istruzioni dei framework si salvano da sole, poco meno di un secondo dopo l'ultima modifica.
- Come nell'editor, un'impostazione cambiata rispetto al valore di base ha una barra colorata sul bordo sinistro.
- Se l'editor è aperto e cambi lingua, rete o temporanei dei programmi, compare «Riavvia l'editor perché i suoi terminali usino le nuove impostazioni.»: le altre impostazioni non lo richiedono. Il tema arriva subito anche all'editor e ai suoi terminali.

#### Generale

| Impostazione | Che cosa fa |
| --- | --- |
| Lingua | La lingua di Atollo: quella del sistema («Lingua del sistema»), italiano, inglese, francese o tedesco. Cambia subito. |
| Tema | Chiaro o scuro: «Come il sistema» (di base), «Chiaro» o «Scuro». Vale subito per tutte le finestre: Atollo, l'editor integrato e i suoi terminali, senza riavviare l'editor. |
| Temporanei dei programmi | Dove i programmi del kit mettono i loro file temporanei. «Automatico» usa la cartella temporanea del sistema se il kit è su una chiavetta, un disco di rete o un disco FAT32/exFAT, altrimenti `data\tmp`. «Nel kit» sempre `data\tmp`; «Nel sistema» sempre la cartella del sistema (i file restano sul PC finché chiudi l'app). |
| Aggiornamento automatico | «Scarica da solo le nuove versioni di Atollo (sostituiscono l'exe alla chiusura)»: la nuova versione sostituisce quella attuale quando chiudi l'app. Attivo di base. Vale subito: attivarlo avvia un controllo all'istante; spegnerlo prima del controllo dei 20 secondi dopo l'avvio ferma quel controllo. |
| Pacchetti per reinstallare senza rete | Solo su macOS e Linux: conserva nel kit i pacchetti dei componenti che vanno reinstallati quando il kit cambia cartella (R, Ruby, MongoDB, SQLite, make), così non serve la rete. «Libera» li cancella. |

> [!TIP]
> **Perché i temporanei fuori dal kit su una chiavetta**
>
> I programmi scrivono e cancellano continuamente file temporanei: su una chiavetta questo rallenta tutto e la consuma. Sul disco del PC vanno molto più veloci, e il kit li cancella quando chiudi l'app.

> [!NOTE]
> **Versioni e compilatore di Rust**
>
> Le Impostazioni non scelgono più le versioni di Python e PHP: una nuova installazione prende la linea più recente, le altre linee si scaricano dal menu delle versioni di ogni strumento in Strumenti e ogni progetto sceglie la sua ([5.2 Più versioni dello stesso strumento](#52-più-versioni-dello-stesso-strumento)). I valori pythonVersion e phpVersion di un vecchio settings.json vengono ignorati. Il «Compilatore di Rust» è nella pagina di Rust in Strumenti, sezione «Dettagli».

#### Rete

Proxy, certificati aggiuntivi e rete locale: [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale).

#### Agenti

«Chiave API di Gemini (Antigravity senza account Google)»: incolla la chiave e passa ad altro (un clic fuori dal campo o Tab). La chiave si salva quando il campo perde il fuoco, poi il campo si svuota e mostra «salvata: scrivi per sostituirla»: la chiave non viene più mostrata. «Togli la chiave» la cancella subito. Antigravity mostrerà «accesso con chiave API». La chiave è salvata a parte, in `data\app\secrets\gemini-api-key`, leggibile solo dal tuo utente dove il disco ha i permessi, e passata come GEMINI_API_KEY solo ad Antigravity avviato da Atollo: non alle azioni dei progetti, agli script, ai terminali né all'editor.

#### Aggiornamenti

Lo stato degli aggiornamenti di Atollo e il pulsante «Cerca aggiornamenti»: [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza).

#### Account GitHub

L'accesso a GitHub per tutto il kit: [6.1 L'account GitHub del kit](#61-laccount-github-del-kit).

Sotto l'account, «Chi firma i commit» ha i campi «Nome» ed «E-mail» dei commit fatti con il Git del kit; ognuno si salva quando il campo perde il focus.

#### Privacy

«Cosa fa Atollo con i tuoi dati», in breve: nessuna telemetria e nessun account; impostazioni, chiavi e accessi restano nella cartella del kit. Atollo contatta GitHub per i suoi aggiornamenti (se l'aggiornamento automatico è attivo), le fonti ufficiali di linguaggi e strumenti quando li installi o aggiorni e, una volta al giorno, per leggere le versioni disponibili (anche questo si spegne con ATOLLO_NO_UPDATE), e Open VSX per le estensioni dell'editor. Con la pagina Agenti aperta, ogni 2 minuti legge i limiti del piano di ogni agente collegato, mandandone il token solo al suo fornitore; con l'account GitHub collegato ne usa il token con api.github.com. Agenti, GitHub e registri dei pacchetti parlano direttamente con i loro servizi, con le loro regole.

«Leggi l'informativa sulla privacy» apre nel browser l'informativa completa (docs/PRIVACY.md su GitHub), nella lingua dell'app.

#### Il kit

Informazioni in sola lettura: cartella del kit, cartella dei progetti, cartella dei dati e dei programmi, disco del kit (file system e tipo: fisso, rimovibile, di rete), cartella temporanea dei programmi, piattaforma, versione di WebView2 e di Atollo. Se il disco ha qualcosa da segnalare compare un avviso.

«Salute del kit» controlla ciò che del PC e del disco può fermare il kit: il disco (FAT32, exFAT, noexec), lo spazio libero, i caratteri a rischio nel percorso del kit, i file resi privati da un altro account e, su Linux, le librerie di sistema. Ogni riga è verde, gialla o rossa, con sotto che cosa fare; «Controlla di nuovo» rifà i controlli ([9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va)).

Sotto, «Segnalare un problema» ha due pulsanti. «Esporta diagnostica» crea in `data\app\diagnostics` uno zip con versione, sistema, disco, ambiente, verifica dei programmi, componenti, porte e la parte finale dei log, con chiavi, token e password del kit sostituiti da \*\*\*, la cartella del tuo profilo come ~ e i nomi tuo e del PC tolti; «Apri la cartella» lo mostra. Lo zip resta sul PC: guardalo e allegalo tu. «Segnala un problema» apre nel browser una segnalazione nuova su GitHub con versione e sistema già scritti; nulla parte finché non la crei.

#### Framework

Le istruzioni di «Nuovo progetto»: [6.4 Le istruzioni dei framework](#64-le-istruzioni-dei-framework).

<details>
<summary>Sotto il cofano</summary>

- Le impostazioni sono in `data\app\settings.json`. Ogni salvataggio rilegge il file, controlla i valori, lo scrive in modo sicuro (prima una copia, poi la sostituzione) e ricostruisce l'ambiente del kit. I salvataggi passano uno alla volta, sempre con i valori più recenti.
- Il tema è la voce theme di settings.json. Atollo lo passa alle finestre (Tauri set_theme): le loro pagine lo vedono come prefers-color-scheme. L'editor integrato lo segue grazie a window.autoDetectColorScheme, e i terminali con lui.
- Se il file non si può leggere, l'app usa i valori predefiniti e lo riscrive alla prima modifica: «Le impostazioni in … non sono leggibili: uso quelle predefinite. Il file verrà riscritto alla prima modifica.» Le voci scritte da una versione più recente di Atollo, che questa non conosce, restano nel file.
- La casella di ricerca delle Impostazioni cerca anche tra le basi e le aggiunte dei framework.

</details>

Argomenti collegati: [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale), [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza), [6.4 Le istruzioni dei framework](#64-le-istruzioni-dei-framework), [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va)

### 6.3 Rete aziendale: proxy, certificati, rete locale

*Quando servono, come compilarli e a che cosa si applicano. Più come far vedere i tuoi siti agli altri PC della rete.*

> **In breve**
>
> - Servono solo se la rete passa da un proxy o ispeziona il traffico HTTPS, come in molte aziende e scuole.
> - Valgono per i download del kit, per gli agenti e per gli strumenti nelle console.
> - «Rete locale» fa vedere i siti dei progetti agli altri PC della rete.

#### Capire se ti servono

Se a casa tutto si scarica e in ufficio i download falliscono con errori di connessione o di certificato, probabilmente la rete aziendale usa un proxy, oppure ispeziona il traffico HTTPS con un suo certificato. I valori giusti te li dà l'assistenza informatica; spesso sono gli stessi impostati nel browser aziendale.

#### I campi

| Campo | Esempio | Note |
| --- | --- | --- |
| Proxy HTTPS | `http://proxy.azienda.it:8080` | Il più importante: lo usano anche i download di Atollo. |
| Proxy HTTP | `http://proxy.azienda.it:8080` | Per le connessioni non cifrate. |
| Senza proxy per | `localhost,127.0.0.1,.azienda.it` | Indirizzi da raggiungere direttamente, separati da virgola. |
| Certificati aggiuntivi (file PEM) | `E:\certificati\azienda.pem` | Il certificato dell'azienda, per i programmi come Node.js che non usano quelli del sistema. |

1. Apri Impostazioni › «Rete» (al primo avvio, il gruppo «Rete» della pagina «Benvenuto in Atollo»: niente si scarica prima di «Installa») e compila i campi che ti servono.
2. Non serve premere nulla: i valori si salvano da soli mentre scrivi. Se l'editor è aperto, premi «Riavvia l'editor» nell'avviso che compare.
3. Torna in Strumenti e riprova il download che non riusciva.

- Il proxy deve iniziare con http:// o https:// e non avere spazi; se serve l'utente: http://utente:password@host:porta.
- Il file PEM deve esistere: il kit controlla il percorso a ogni salvataggio. Finché il valore non va bene compare il suo errore e il campo non viene salvato; il kit riprova alla modifica successiva.

#### Rete locale

La casella «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete» apre i siti dei progetti alla rete: utile per provare un sito dal telefono. Dagli altri dispositivi si usa l'indirizzo IP di questo PC con la porta del sito (per esempio http://192.168.1.20:8000). Database e phpMyAdmin restano raggiungibili solo da questo PC.

- Il server PHP integrato (php -S) e il server di file dei siti statici rispondono a chi li chiama con l'indirizzo IP del PC; con qualsiasi altro nome la richiesta resta rifiutata.
- Su questo PC un sito statico risponde anche a http://\<progetto\>.localhost:\<porta\>, oltre che a localhost e 127.0.0.1, con la casella accesa o spenta.

> [!WARNING]
> **Attenzione**
>
> Chiunque sia sulla stessa rete potrà vedere i tuoi siti di prova. Attivala su reti di cui ti fidi e spegnila quando hai finito. Windows può chiedere il permesso del firewall la prima volta.

<details>
<summary>Sotto il cofano</summary>

- I valori finiscono nelle variabili HTTPS_PROXY, HTTP_PROXY, NO_PROXY e NODE_EXTRA_CA_CERTS di tutti i programmi del kit. Con un file PEM, `data\app\ca-bundle.pem` riunisce i certificati del sistema e i suoi, per gli strumenti che leggono un solo file (SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO). Un file PEM dentro il kit è salvato relativo al kit: lo segue se cambia disco o cartella.
- Anche i download di Atollo usano queste impostazioni: il «Proxy HTTPS» (se vuoto il «Proxy HTTP», altrimenti le variabili del PC o il proxy di sistema), gli host di «Senza proxy per» raggiunti direttamente, e i certificati del sistema più quelli del file PEM. Se il certificato aziendale è già installato nel PC, il file PEM non serve. Un file senza certificati PEM viene rifiutato.
- I file PAC (configurazione automatica del proxy) non sono supportati: chiedi l'indirizzo del proxy.
- «Rete locale» imposta ATOLLO_ALLOW_LAN=1: i comandi dei progetti ascoltano su tutte le interfacce (127.0.0.1 nel comando diventa 0.0.0.0), e così il server di file e i pool di Caddy e Apache. Il router di php -S e il server di file accettano allora come Host anche un indirizzo IP; a ogni altro nome che non sia localhost, \*.localhost, 127.0.0.1 o \[::1\] rispondono 421.

</details>

Argomenti collegati: [8.4 Da dove arrivano gli strumenti](#84-da-dove-arrivano-gli-strumenti), [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va), [3.3 Siti web e indirizzi .localhost](#33-siti-web-e-indirizzi-localhost)

### 6.4 Le istruzioni dei framework

*Modificare le ricette di «Nuovo progetto»: basi, aggiunte, comandi, variabili .env, azioni. E provare una combinazione prima di usarla.*

> **In breve**
>
> - Le basi e le aggiunte di «Nuovo progetto» sono ricette: elenchi di comandi che puoi leggere e modificare.
> - Le trovi in Impostazioni, nel riquadro «Framework».
> - Puoi cambiare una ricetta ufficiale (e ripristinarla) o crearne di tue.
> - «Prova una combinazione» mostra che cosa farebbe una base con delle aggiunte, prima di usarla.

#### Com'è fatta una ricetta

1. **Strumenti** — I componenti del kit che servono (Node.js, PHP…).
2. **Crea** — I comandi che creano il progetto, eseguiti nella cartella projects.
3. **Poi** — I comandi da eseguire nella cartella del progetto.
4. **Contorno** — Database, variabili .env, azioni del progetto.

Ogni riga è un comando eseguito con la PowerShell e gli strumenti del kit. {name} diventa il nome del progetto (tra virgolette), {package} com.example. seguito dal nome in minuscolo.

#### Il riquadro «Framework»

L'albero ha le «Basi», divise per gruppo, e sotto ogni base le aggiunte che vanno con lei; poi le «Aggiunte» per tipo; in fondo «Prova una combinazione». In cima, le icone «Aggiungi una base» e «Aggiungi un'aggiunta».

| Campo | Che cosa scrivere |
| --- | --- |
| Nome, Descrizione, Pagina ufficiale | Come compare nella procedura e il collegamento alla documentazione. |
| Strumenti del kit | I componenti da installare prima («Aggiungi uno strumento del kit»). |
| Crea il progetto (nella cartella projects) | Per esempio: npm create vite@latest {name} -- --template vanilla-ts |
| Poi, nella cartella del progetto | Per esempio: npm install |
| Database del kit per il progetto | Nessuno, MySQL o MongoDB. La password che la procedura genera per il database vince su una vuota della base (per esempio DB_PASSWORD= in .env.example). |
| Variabili del file .env (CHIAVE=valore) | Per esempio: APP_NAME={name} |
| Azioni del progetto (Nome: comando) | Per esempio: Avvia: npm run dev |

Un'aggiunta ha in più il «Gruppo» (due aggiunte dello stesso gruppo non vanno insieme), «Usala anche con» per le basi compatibili e, per ogni base, i comandi da aggiungere o «Crea il progetto al posto della base».

1. Scegli una base o un'aggiunta nell'albero e modifica i campi.
2. Le istruzioni si salvano da sole poco meno di un secondo dopo l'ultima modifica: la barra delle schede mostra «Salvataggio…», poi «Salvato.». Una ricetta ufficiale cambiata mostra «Modificata».
3. Per tornare alla versione originale usa «Ripristina le istruzioni ufficiali»; una ricetta tua si toglie con «Togli».

#### Prova una combinazione

Scegli una base e delle aggiunte: la scheda mostra il piano completo che la procedura eseguirebbe (strumenti, comandi, database, .env, azioni), con le istruzioni come sono adesso, anche se l'ultima modifica non è ancora salvata. Non esegue niente.

<details>
<summary>Sotto il cofano</summary>

- Le ricette ufficiali sono nel catalogo del kit (tauri/catalog/frameworks.json): 63 basi e 18 aggiunte.
- Le tue modifiche sono in `data\app\frameworks.json`, che contiene solo le differenze dalle ricette ufficiali.
- Se quel file non si può leggere, il kit usa le ricette ufficiali e non lo tocca: «Le istruzioni in … non sono leggibili: uso quelle ufficiali e non modifico il file.»

</details>

Argomenti collegati: [2.1 Creare un progetto](#21-creare-un-progetto), [6.2 Le Impostazioni, una per una](#62-le-impostazioni-una-per-una)

## 7. Portare il kit con sé

### 7.1 Rimuovere l'unità in sicurezza

*Chiudere tutto quello che il kit ha aperto prima di staccare la chiavetta o il disco, senza perdere dati.*

> **In breve**
>
> - Prima di staccare la chiavetta o il disco del kit, usa la pagina «Rimuovi l'unità».
> - Chiude con ordine tutto ciò che il kit ha aperto: progetti, server, editor, console, database.
> - Poi chiudi Atollo e rimuovi l'unità con la «Rimozione sicura dell'hardware» di Windows.

![La pagina «Rimuovi l'unità»](../../ui/public/guide/it/eject.webp)

*Che cosa è aperto adesso e il pulsante che chiude tutto prima di staccare l'unità.*

#### Perché serve

Un database spento di colpo può perdere gli ultimi dati scritti, e un programma che ha un file aperto sulla chiavetta impedisce a Windows di rimuoverla. La pagina chiude tutto nel modo giusto: i database con il loro comando di arresto, i server e i programmi uno per uno. E salva nel kit l'accesso di Antigravity, così sul PC non resta niente.

#### Passo per passo

1. Salva il lavoro aperto nell'editor e negli altri programmi.
2. Apri «Rimuovi l'unità» (l'ultima icona in basso nella barra delle attività). La parte «Aperti adesso» elenca che cosa è aperto: installazione in corso, editor, console, progetti avviati, database avviati, il server dei modelli locali e i programmi del kit avviati fuori dall'app, con nome e PID.
3. Premi «Chiudi tutto e prepara la rimozione».
4. Quando compare «Programmi del kit chiusi e accesso di Antigravity salvato nel kit.», premi «Chiudi Atollo». Se qualcosa è rimasto aperto, è elencato sotto «Ancora aperti (chiudili a mano)».
5. Rimuovi l'unità dal sistema: su Windows con l'icona «Rimozione sicura dell'hardware»; su macOS espellendola dal Finder.

> [!WARNING]
> **Attenzione**
>
> I file non salvati nei programmi elencati andranno persi. La pagina non espelle l'unità da sola: l'ultimo passo lo fai tu dal sistema.

#### E se chiudo semplicemente la finestra?

Se c'è qualcosa di aperto, Atollo chiede «Chiudere Atollo?» ed elenca che cosa si chiuderà. «Chiudi tutto» ferma azioni e server, spegne i database con ordine ed esce. È sicuro anche questo; la pagina «Rimuovi l'unità» in più chiude i programmi del kit avviati fuori dall'app (per esempio da una console) e ti dice se resta qualcosa.

Lo stesso vale per «Chiudi Atollo» nel menu dell'icona di Atollo, nell'area di notifica di Windows, nella barra dei menu di macOS o nell'area di stato di Linux. Quel menu ha anche «Mostra Atollo» e «Progetti», con i progetti più recenti: un clic apre la pagina del progetto.

<details>
<summary>Sotto il cofano</summary>

L'ordine con cui la pagina chiude le cose:

1. **Installazione** — Si ferma alla fine del passo in corso.
2. **Progetti** — Le azioni avviate.
3. **Server web** — I pool di Caddy e Apache.
4. **Editor** — Le finestre; il server dell'editor si ferma subito dopo i database.
5. **Database** — Arresto ordinato di MySQL e MongoDB, ogni versione avviata.
6. **Il resto** — Console e ogni processo con il programma dentro il kit.
7. **Antigravity** — L'accesso torna nel kit, il sistema torna com'era.

Il kit non trova i programmi del PC che hanno aperto un file della chiavetta (per esempio un documento aperto in un'app esterna): chiudili tu.

</details>

Argomenti collegati: [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella), [8.3 Processi in background](#83-processi-in-background), [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza)

### 7.2 Su un altro PC, un'altra lettera, un'altra cartella

*Che cosa succede quando il kit cambia posto, e come si comporta sullo stesso disco aperto da Windows, macOS e Linux.*

> **In breve**
>
> - Puoi spostare o copiare la cartella del kit dove vuoi: un'altra lettera, un'altra cartella, un altro PC.
> - Al primo avvio nel nuovo posto il kit se ne accorge e sistema da solo i percorsi.
> - La stessa chiavetta si apre su Windows, macOS e Linux: progetti e dati sono in comune, i programmi sono separati per sistema.

#### Cambiare lettera, cartella o PC

Su un altro PC la chiavetta potrebbe diventare E: invece di D:. Non devi fare niente: all'avvio il kit confronta la cartella attuale con quella dell'ultima volta e, se è cambiata, mostra «Il kit è stato spostato: prima era in …, ora in …» e aggiorna i percorsi.

- L'ambiente dei programmi (PATH e variabili) si ricostruisce a ogni avvio dalla cartella attuale.
- I percorsi scritti nei file degli agenti e dell'editor vengono riscritti con il nuovo percorso.
- Le configurazioni dei server web si rigenerano a ogni avvio e usano percorsi relativi.
- Nel .gitconfig del kit, safe.directory (le cartelle di cui Git si fida) viene riscritto a ogni avvio con la cartella attuale del kit, per esempio D:/Atollo/\*. Il kit toglie solo il valore che aveva scritto prima (lo ricorda in `data\app\safe-directory.txt`): le cartelle che hai aggiunto tu restano.
- Il file PEM dei certificati (Impostazioni › Rete) e la cartella di un terminale, se sono dentro il kit, sono salvati relativi al kit: seguono lo spostamento.
- Accessi, impostazioni, progetti e database sono nella cartella: viaggiano con lei.

> [!NOTE]
> **Da sapere**
>
> Si può aprire un kit alla volta per cartella: avviare di nuovo Atollo dalla stessa cartella porta in primo piano la finestra già aperta. Due kit diversi (due chiavette) possono invece girare insieme.

#### Lo stesso kit su Windows, macOS e Linux

I programmi sono diversi per ogni sistema, e il kit li tiene in cartelle separate (`data\apps\win32-x64`, `data\apps\darwin-arm64`, `data\apps\linux-x64…`). Tutto il resto è in comune: progetti, impostazioni, accessi, istruzioni degli agenti, database.

1. Formatta la chiavetta in exFAT, che tutti e tre i sistemi leggono e scrivono.
2. Metti nella cartella del kit l'eseguibile di ciascun sistema (Atollo.exe, Atollo.app, il file AppImage).
3. Su ogni sistema, apri Atollo: alla prima apertura installa i componenti scelti per quel sistema. In alternativa, dalla pagina Strumenti scegli quel sistema in «Piattaforma» e prepara i programmi in anticipo da un altro PC.

> [!WARNING]
> **Attenzione**
>
> Un database usato da più sistemi va sempre spento prima di cambiare PC (usa «Rimuovi l'unità»). Su macOS e Linux alcuni componenti (R, Ruby, MongoDB su Linux, SQLite su Linux ARM64, make) vanno reinstallati quando il kit cambia cartella: compaiono come «da aggiornare» e il kit li reinstalla da solo, senza rete se hai attivato «Pacchetti per reinstallare senza rete».

<details>
<summary>Sotto il cofano</summary>

- L'ultima cartella è in `data\app\root.txt` (root-darwin.txt e root-linux.txt su macOS e Linux); la precedente in old-root.txt finché i file degli agenti non sono stati aggiornati.
- Su Windows il confronto delle cartelle ignora maiuscole e minuscole; su macOS e Linux no.
- Le versioni installate di ogni sistema sono in `data\app\versions\<sistema>`.
- macOS: l'app non è autenticata da Apple. Se macOS la apre da una posizione temporanea («App Translocation»), Atollo si ferma e mostra il comando xattr da eseguire per togliere la quarantena. Linux: servono glibc 2.35 o più recente e WebKitGTK 4.1; se mancano, Atollo mostra il comando per installarli.

</details>

Argomenti collegati: [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza), [8.1 Portabile e isolato: che cosa vuol dire](#81-portabile-e-isolato-che-cosa-vuol-dire), [5.3 Panoramiche: matrice, relazioni, estensioni, requisiti](#53-panoramiche-matrice-relazioni-estensioni-requisiti)

### 7.3 Aggiornamenti e copie di sicurezza

*Aggiornare Atollo e i suoi strumenti, e che cosa copiare per avere un backup completo del tuo lavoro.*

> **In breve**
>
> - Atollo si aggiorna da solo: scarica la nuova versione e la mette al posto della vecchia quando chiudi l'app.
> - Gli strumenti si aggiornano dalla pagina Strumenti, con «Aggiorna \<nome\>» su una riga o «Aggiorna tutto».
> - Per una copia di sicurezza completa basta copiare projects e data (senza i programmi, che si riscaricano).

#### Aggiornare Atollo

Con «Aggiornamento automatico» attivo (Impostazioni › Generale), venti secondi dopo l'avvio il kit controlla se c'è una nuova versione, la scarica, ne verifica l'impronta e la prepara. Quando chiudi l'app, la nuova versione prende il posto della vecchia; al successivo avvio la vecchia copia viene cancellata. L'interruttore vale subito: attivarlo avvia un controllo all'istante, spegnerlo prima dei venti secondi ferma quel controllo.

In Impostazioni › «Aggiornamenti» vedi lo stato («Atollo … è la versione più recente.», «La versione … è pronta: sostituisce questa alla chiusura di Atollo.») e il pulsante «Cerca aggiornamenti», che controlla e scarica subito anche con l'aggiornamento automatico spento. Lo stesso fa un clic sulla versione, l'ultima voce della barra di stato: accanto compare l'esito, e passandoci sopra lo leggi per intero.

> [!NOTE]
> **Da sapere**
>
> Su macOS e Linux, se usi il programma semplice (dall'archivio tar.gz) invece di Atollo.app o dell'AppImage, il kit non può sostituirlo: ti dice che la nuova versione è disponibile e la scarichi dalla pagina della release.

#### Aggiornare gli strumenti

- «Aggiorna \<nome\>» (l'icona con le due frecce) sulla riga di un componente scarica subito la sua versione nuova, se c'è.
- «Aggiorna tutto», in fondo alla pagina Strumenti, aggiorna ogni componente installato e porta ogni versione all'ultima correzione (per esempio PHP 8.4.1 → 8.4.3), senza aggiungere né togliere versioni.
- Gli agenti seguono sempre l'ultima versione e non si aggiornano da soli mentre lavori: lo fa il kit con «Aggiorna \<nome\>» o «Aggiorna tutto».

> [!TIP]
> **Consiglio**
>
> Aggiorna quando non hai progetti avviati: i programmi in uso non si possono sostituire, e la riga mostrerebbe «in uso: chiudi i programmi del kit e riprova».

#### Copie di sicurezza

| Da copiare | Perché |
| --- | --- |
| `projects\` | I tuoi progetti. |
| `data\` | Impostazioni, accessi, istruzioni degli agenti, dati dei database, copie dei database, dati dei progetti. |

Dentro data puoi lasciare fuori `data\apps` (i programmi) e `data\cache` (file scaricati che si possono riscaricare): sono la parte più grande e il kit li ricostruisce. Anche `data\models` (i modelli locali, molti GB) si può riscaricare dalla pagina Modelli locali, tranne i file .gguf messi a mano. Per copiare i dati dei database in modo coerente, spegni prima i server (o usa «Rimuovi l'unità»).

> [!WARNING]
> **Attenzione**
>
> La cartella data contiene accessi e password. Tieni le copie in un posto sicuro, e cifra la chiavetta se la porti in giro (per esempio con BitLocker To Go su Windows).

Per un singolo database c'è la copia dalla pagina del progetto: [3.2 Il database di un progetto](#32-il-database-di-un-progetto).

<details>
<summary>Sotto il cofano</summary>

- Le nuove versioni arrivano dalle release app-v… del repository GitHub nabre/atollo; l'impronta SHA-256 viene da GitHub o dal file .sha256 accanto al file.
- Su Windows la nuova versione attende in `data\app\update`; la copia sostituita resta come Atollo.old.exe fino al prossimo avvio.
- La variabile ATOLLO_NO_UPDATE, impostata nel PC, spegne il controllo automatico e anche la lettura giornaliera delle versioni offerte dalle fonti.

</details>

Argomenti collegati: [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza), [8.2 Le cartelle del kit su disco](#82-le-cartelle-del-kit-su-disco)

## 8. Come funziona dentro

### 8.1 Portabile e isolato: che cosa vuol dire

*Perché il kit non si installa, non chiede diritti di amministratore e non tocca il PC che lo ospita. E le eccezioni.*

> **In breve**
>
> - Atollo non si installa, non scrive nel registro e non chiede diritti di amministratore.
> - I programmi del kit partono in un ambiente costruito da zero, che punta solo a cartelle del kit.
> - Così non vedono i programmi del PC e il PC non vede loro: niente conflitti, niente tracce.
> - Poche eccezioni sono dichiarate: la cartella temporanea del sistema e, per qualche secondo, il portachiavi.

#### L'idea, con un esempio

Sul PC di un collega c'è già Node.js 18 installato, con i suoi pacchetti globali e le sue impostazioni di npm. Se apri una console del kit e scrivi node --version, rispondi Node 24 del kit, non il 18 del PC: la console del kit non sa nemmeno che l'altro esiste. E quando installi un pacchetto globale, finisce nella cartella del kit, non nel profilo del collega.

Questo succede perché ogni programma avviato dal kit riceve un ambiente costruito da zero: il kit prende dal PC solo una manciata di variabili di sistema (nome del computer, cartelle di Windows, lingua…) e imposta tutte le altre in modo che puntino dentro data.

#### Che cosa viene reindirizzato

| Gruppo | Variabili principali | Dove portano |
| --- | --- | --- |
| Cartella utente | `HOME, USERPROFILE, APPDATA, LOCALAPPDATA` | `data\home` |
| File temporanei | `TEMP, TMP (TMPDIR su macOS e Linux)` | `data\tmp\system` o la cartella temporanea del sistema, secondo le Impostazioni |
| Agenti | `CLAUDE_CONFIG_DIR, CODEX_HOME, PI_CODING_AGENT_DIR` | `data\home\.claude, data\home\.codex, data\home\.pi\agent` |
| GitHub CLI | `GH_CONFIG_DIR` | `data\gh` |
| Cache dei pacchetti | `NPM_CONFIG_CACHE, PIP_CACHE_DIR, UV_CACHE_DIR, COMPOSER_CACHE_DIR, GOCACHE, NUGET_PACKAGES` | `data\cache\…` |
| Pacchetti globali | `NPM_CONFIG_PREFIX, PYTHONUSERBASE, COMPOSER_HOME, GOPATH, GRADLE_USER_HOME, CARGO_HOME` | `data\npm-global, data\python-user, data\composer, data\go, data\gradle…` |
| Linguaggi | `JAVA_HOME, DOTNET_ROOT, PHPRC, RUSTUP_HOME…` | Le cartelle dei programmi del kit, alle versioni in uso |
| Certificati | `SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO, NODE_EXTRA_CA_CERTS` | Solo con un file PEM in Impostazioni › Rete: `data\app\ca-bundle.pem` (i certificati del sistema più quelli del file); Node.js riceve il file PEM |
| Niente telemetria e aggiornamenti automatici | `DOTNET_CLI_TELEMETRY_OPTOUT, POWERSHELL_TELEMETRY_OPTOUT, GOTELEMETRY=off, DISABLE_UPDATES` | Spenti: gli strumenti non mandano dati e non si aggiornano da soli |
| PATH | `PATH` | Prima le cartelle del kit, poi solo le cartelle di sistema (System32 su Windows, /usr/bin e simili su macOS e Linux) |

Per vedere l'ambiente completo apri una console nella cartella del kit e usa il comando `.\Atollo.exe env | Out-Host` (le chiavi e le password sono mascherate con \*\*\*).

#### Che cosa resta sul PC

Il kit non scrive nel registro di Windows, non cambia il PATH del sistema, non crea servizi né operazioni pianificate. Le eccezioni, dichiarate:

- La cartella temporanea del sistema, in `Atollo\<id del kit>`: area di lavoro dei download, file consegnati a mano, temporanei dei programmi. Viene pulita alla chiusura e ai successivi avvii.
- Mentre Antigravity è aperto, il suo accesso sta nel portachiavi del sistema (su Windows solo per la sessione); alla chiusura torna nel kit.
- Una cartella vuota che la PowerShell crea sempre in `%LOCALAPPDATA%\Microsoft\PowerShell`.
- Se esegui gh auth login a mano in una console, GitHub CLI può salvare l'accesso nella Gestione credenziali di Windows: accedi invece dalle Impostazioni.
- Su macOS: i dati della finestra in ~/Library/WebKit e, se non usi il token, l'accesso di Claude Code nel Portachiavi.

> [!NOTE]
> **Diritti di amministratore**
>
> Atollo parte sempre come utente normale. Le uniche cose che chiedono un amministratore sono esterne al kit e facoltative: Visual Studio Build Tools per il compilatore MSVC di Rust, gli Xcode Command Line Tools su macOS per alcuni programmi C/C++.

<details>
<summary>Sotto il cofano</summary>

- L'ambiente parte vuoto e copia dal PC circa 40 variabili di sistema (ComSpec, SystemRoot, PROCESSOR_ARCHITECTURE, LANG, DISPLAY…) più le variabili del proxy; il kit ne imposta un centinaio. Atollo env su Windows stampa circa 130 righe.
- Ogni programma figlio parte con l'ambiente svuotato e poi riempito con quello del kit: niente filtra dal PC.
- Anche il processo dell'app reindirizza la propria cartella utente e i temporanei, prima di avviare qualsiasi cosa.
- Le variabili che contengono KEY, TOKEN, SECRET o PASSWORD sono mascherate nel comando env e nel log.

</details>

Argomenti collegati: [8.2 Le cartelle del kit su disco](#82-le-cartelle-del-kit-su-disco), [8.3 Processi in background](#83-processi-in-background), [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella)

### 8.2 Le cartelle del kit su disco

*projects e data: che cosa c'è in ogni cartella, quali file sono tuoi, quali del kit e quali sono protetti.*

> **In breve**
>
> - Tutto sta nella cartella del kit: l'eseguibile, projects (tuo) e data (del kit).
> - In data i programmi sono separati per sistema; il resto è in comune fra Windows, macOS e Linux.
> - Le cartelle con password sono leggibili solo dal tuo utente (sui dischi NTFS).

#### Le cartelle

- **Cartella progetti** — `projects\`
- **Agenti condivisi** — `data\agents\`
- **Dati database** — `data\db\`
- **Backup database** — `data\projects\<progetto>\private\backups\`

| Cartella o file | Che cosa contiene |
| --- | --- |
| `<cartella del kit>\` | La cartella che contiene l'eseguibile, per esempio `D:\Atollo`. |
| &emsp;`Atollo.exe` | L'unico programma: è l'app con la finestra e anche la riga di comando. Su macOS Atollo.app, su Linux l'AppImage. |
| &emsp;`projects\` | I tuoi progetti, una cartella ciascuno. È tuo: il kit non ci scrive senza che tu lo chieda. |
| &emsp;`data\` | Tutto ciò che è del kit. |
| &emsp;&emsp;`app\` | Lo stato dell'app: settings.json (Impostazioni), `secrets\` (la chiave di Gemini, quella del server dei modelli locali e il token di Hugging Face, protetti), llm.json e `llm\presets.ini` (il server dei modelli locali), ca-bundle.pem (i certificati, se le Impostazioni hanno un file PEM), components.txt (componenti scelti), `versions\` (versioni installate), `logs\`, services.json e servers.json (database e server web, con le porte di ogni versione e di ogni pool), terminals.json, frameworks.json, root.txt (ultima cartella del kit), safe-directory.txt (l'ultimo safe.directory scritto dal kit). |
| &emsp;&emsp;&emsp;`app\db\` | Password di amministratore dei database e accessi di mysql. Protetta: la legge solo il tuo utente. |
| &emsp;&emsp;&emsp;`app\diagnostics\` | Gli zip di «Esporta diagnostica» (atollo-diagnostica-\<data-ora\>.zip), da allegare a una segnalazione: segreti mascherati, restano sul PC. |
| &emsp;&emsp;`apps\<sistema>\` | I programmi, per sistema (win32-x64, darwin-arm64…), una cartella per componente e per versione. |
| &emsp;&emsp;`projects\<progetto>\` | I dati del kit per ogni progetto: dev-settings.json (versioni, estensioni, preferiti), ports.json (le porte delle sue azioni), configurazioni di Apache e Caddy (le tue direttive in `apache\conf.d\*.conf` e `caddy\conf.d\*.caddy` valgono sia per il server tutto suo sia in un pool), `private\` (password e copie dei database, protetta). |
| &emsp;&emsp;`db\<mysql\|mongodb>\<versione>\` | I dati dei database, una cartella per versione. |
| &emsp;&emsp;`models\gguf\<proprietario>--<nome>\` | I modelli locali (file GGUF), una cartella per repository di Hugging Face, una sola copia per Windows, macOS e Linux; atollo.json dice che cosa ha scaricato il kit. |
| &emsp;&emsp;`servers\<pool>\` | Le configurazioni generate dei server web condivisi. |
| &emsp;&emsp;`agents\` | Ciò che gli agenti condividono: AGENTS.md, `skills\`, mcp.json, e lo stato dell'allineamento in `.sync\`. |
| &emsp;&emsp;`antigravity\` | L'accesso di Antigravity conservato nel kit (keyring.json). |
| &emsp;&emsp;`home\` | La «cartella utente» dei programmi del kit: .claude, .codex, .gemini, .pi (profili degli agenti), .gitconfig, .npmrc, AppData (su Windows). |
| &emsp;&emsp;`gh\` | L'accesso di GitHub CLI (hosts.yml). |
| &emsp;&emsp;`vscodium-server\` | Impostazioni ed estensioni del server dell'editor. |
| &emsp;&emsp;`kit\` | File che il kit riscrive a ogni avvio: profilo della PowerShell, comandi di supporto (agy, mysql, gradle). |
| &emsp;&emsp;`cache\` | Cache dei gestori di pacchetti (npm, pip, uv, Composer, Go, NuGet…) e delle risposte di GitHub. Si può cancellare. |
| &emsp;&emsp;`python-user\, npm-global\, composer\, go\, gradle\…` | Pacchetti e strumenti installati globalmente con pip, npm, Composer, Go e gli altri. |
| &emsp;&emsp;`tmp\` | File temporanei, quando le Impostazioni li vogliono nel kit. |
| &emsp;&emsp;`webview2\` | I dati della finestra dell'app. |

#### Che cosa è tuo e che cosa è del kit

| Puoi modificare a mano | Meglio lasciare al kit |
| --- | --- |
| Tutto in projects; `data\agents` (istruzioni, skill, mcp.json); `data\home\.gitconfig` | `data\app`, `data\apps`, `data\kit`, `data\servers`, `data\db`: li gestisce il kit, e alcuni vengono riscritti a ogni avvio |

> [!WARNING]
> **Attenzione**
>
> Non cancellare a mano le cartelle dei database (`data\db`) o le cartelle private dei progetti: perderesti dati e password. Per togliere un componente usa la pagina Strumenti; per pulire i dati di un progetto usa «Dati senza progetto».

<details>
<summary>Sotto il cofano</summary>

- Cartelle protette (solo il tuo utente, su NTFS; permessi 700 su macOS e Linux): `data\app\db`, `data\app\secrets`, `data\projects\<progetto>\private`, `data\agents\claude`, `data\tmp` quando è la cartella temporanea di Claude.
- Su exFAT e FAT32 i permessi non esistono: il kit funziona, ma quelle cartelle non sono protette.
- I file di configurazione vengono scritti in modo sicuro: prima un file temporaneo, poi la sostituzione, così un'interruzione non lascia file a metà.
- `data\kit` viene riscritta a ogni avvio: non metterci file tuoi.

</details>

Argomenti collegati: [8.1 Portabile e isolato: che cosa vuol dire](#81-portabile-e-isolato-che-cosa-vuol-dire), [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza), [2.7 Eliminare un progetto e i «Dati senza progetto»](#27-eliminare-un-progetto-e-i-dati-senza-progetto)

### 8.3 Processi in background

*Che cosa parte con l'app, che cosa si ferma quando la chiudi, e perché non restano programmi accesi a tua insaputa.*

> **In breve**
>
> - Quando apri Atollo partono solo le cose che hai chiesto: i database e i server con «Avvia con Atollo».
> - Ogni programma avviato dal kit è legato all'app: se l'app si chiude, anche nel modo peggiore, si chiudono con lei.
> - Una sola finestra per cartella del kit: aprirla di nuovo porta in primo piano quella esistente.

#### All'avvio

1. **Cartelle** — Crea projects e data se mancano e controlla di poter scrivere.
2. **Posto** — Si accorge se il kit è stato spostato e lo annota.
3. **Ambiente** — Legge le Impostazioni e costruisce l'ambiente del kit.
4. **Finestra** — Una sola per cartella; poi apre la finestra.
5. **Sottofondo** — Avvia ciò che è segnato «Avvia con Atollo», allinea gli agenti e Git, controlla aggiornamenti.

- I server di database e i profili dei server web con «Avvia con Atollo» partono subito dopo l'apertura.
- I componenti obbligatori mancanti e quelli da reinstallare vengono installati da soli; al primo avvio no: lì si scarica tutto solo con «Installa» della pagina «Benvenuto».
- Istruzioni, skill e server MCP degli agenti vengono allineati; se il kit è stato spostato, i percorsi vengono aggiornati.
- Git segue la cartella del kit (safe.directory) e l'account GitHub: se «Chi firma i commit» è vuoto, nome e indirizzo privato vengono dall'account.
- Dopo 10 secondi, le versioni offerte da ogni fonte, per i menu che ne scaricano una (al massimo una volta al giorno).
- Dopo 20 secondi, se attivo, il controllo degli aggiornamenti di Atollo (spegnere «Aggiornamento automatico» prima lo ferma).

> [!NOTE]
> **Da sapere**
>
> Aprire la pagina Database non avvia i database: partono con «Avvia», con «Avvia con Atollo» o quando un progetto ne ha bisogno (per creare il suo database o fare una copia).

#### Mentre lavori

| Che cosa | Quando parte | Quando si ferma |
| --- | --- | --- |
| Azioni dei progetti | Con «Avvia» o ▶ | Con «Ferma», o alla chiusura dell'app |
| Server web condivisi | Con il primo progetto che li usa, dalla pagina Server Web o con «Avvia con Atollo» | Con l'ultimo progetto, salvo se avviati dalla pagina Server Web o con «Mantieni attivo anche senza progetti»; con «Ferma»; alla chiusura |
| Database | Con «Avvia», «Avvia con Atollo» o quando un progetto li chiede | Con «Ferma», o alla chiusura (arresto ordinato) |
| Editor | Con il primo «Editor» | Alla chiusura dell'app |
| Console e agenti | Con «Terminale», «Apri», i loghi degli agenti | Quando le chiudi, o alla chiusura dell'app |
| Server dei modelli locali | Con «Avvia», «Avvia con Atollo» o aprendo Pi su un modello locale | Con «Ferma», «Ferma tutto», o alla chiusura; un modello lascia la memoria dopo dieci minuti senza richieste |

Chiudere una console non spegne i database: restano accesi finché non li fermi o chiudi l'app. Il pannello Attività (dalla barra di stato) elenca le azioni dei progetti, ogni versione di database avviata, i pool attivi e il server dei modelli locali; «Ferma tutto» li ferma dopo una conferma.

#### Alla chiusura

Se qualcosa è aperto, Atollo chiede «Chiudere Atollo?» con l'elenco; «Chiudi tutto» ferma le azioni, i server web e il server dei modelli locali e spegne con ordine i database, poi esce. Prima di staccare l'unità usa [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza).

<details>
<summary>Sotto il cofano</summary>

- Su Windows tutti i programmi del kit vivono in un Job Object con l'opzione «chiudi alla chiusura»: se l'app termina in qualsiasi modo, Windows chiude anche loro. Editor e azioni hanno un Job loro, dentro quello dell'app.
- Su macOS e Linux un piccolo processo di guardia si accorge se l'app muore e chiude i gruppi di processi del kit (SIGTERM, poi SIGKILL dopo 5 secondi).
- L'istanza unica usa un identificativo ricavato dalla cartella del kit (un mutex su Windows, un lock su macOS e Linux): due kit diversi non si disturbano.
- Ogni porta ha un solo proprietario: un pool (di base da 8081 a 8999), una versione di database, un'azione di un progetto (di base da 7100 a 7999), il server dei modelli locali (9931 e seguenti); da 7000 a 7099 sono dell'editor. Una porta già presa viene rifiutata con il nome di chi la tiene (per esempio «La porta 3306 è già del database MySQL 8.4: scegline un'altra.»), e la porta automatica di un nuovo pool le evita tutte.
- Ogni versione di un database tiene la sua porta, anche quando cambi la versione del server; due avvii della stessa versione fanno partire un solo server, sulla stessa porta.
- Browser ed Esplora file vengono aperti fuori dal Job: restano aperti anche dopo la chiusura di Atollo.
- Alla chiusura, se c'è un aggiornamento pronto, l'eseguibile viene sostituito.

</details>

Argomenti collegati: [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza), [8.1 Portabile e isolato: che cosa vuol dire](#81-portabile-e-isolato-che-cosa-vuol-dire), [3.1 I server di database](#31-i-server-di-database)

### 8.4 Da dove arrivano gli strumenti

*Le fonti ufficiali di ogni componente, le verifiche crittografiche, il download a mano e la tabella completa delle fonti.*

> **In breve**
>
> - Ogni componente arriva dalla fonte ufficiale del suo produttore, in HTTPS: niente mirror, niente copie di terzi.
> - Ogni file viene verificato con la sua impronta crittografica prima di essere usato; se non coincide, viene scartato.
> - Se un download non riesce, puoi scaricare il file con il browser e consegnarlo al kit, che lo verifica allo stesso modo.

#### Il percorso di un download

1. **Chiede** — Qual è l'ultima versione e qual è la sua impronta, alla fonte ufficiale.
2. **Scarica** — In un file .part nella cartella di lavoro; tre tentativi.
3. **Verifica** — Calcola l'impronta del file e la confronta con quella attesa.
4. **Estrae** — zip, tar.gz, tar.xz, 7z o file singolo.
5. **Sostituisce** — Mette la cartella nuova al posto della vecchia, tutta insieme.
6. **Prova** — Esegue il programma per controllare che risponda.

| Verifica | Che cosa vuol dire | Esempi |
| --- | --- | --- |
| SHA-256, SHA-512, SHA3-256 | L'impronta del file deve essere quella pubblicata dal produttore (o fissata nel catalogo del kit). | Python, Node.js, Go, Maven, Caddy, SQLite, l'editor |
| npm sha512 | L'impronta di integrità del registro npm. | Claude Code, Codex |
| Authenticode (solo Windows) | In più, la firma digitale del produttore sul programma, con la verifica della revoca. | Claude Code, Visual C++ runtime |

> [!TIP]
> **Consiglio**
>
> Ogni parte di un componente (per esempio PHP e il suo Composer) viene scaricata e verificata prima di toccare quella installata: se qualcosa va storto, la versione che avevi resta com'era.

#### Quando un download non riesce: «a mano»

Se dopo tre tentativi il download fallisce (proxy che blocca, sito irraggiungibile), sotto la riga del componente compare un riquadro:

1. Fai clic su «Scaricalo dal browser: \<nome del file\>»: si apre l'indirizzo esatto nel browser.
2. Quando il file è scaricato, trascinalo nel riquadro o usa «sfoglia…».
3. Il kit lo conserva e lo usa solo se la sua impronta è quella pubblicata; l'installazione riparte da sola («Aggiunto …: nuovo tentativo.»).

Il riquadro compare solo quando l'impronta del file è nota. I file consegnati e non usati vengono cancellati dopo 30 giorni.

#### GitHub e i limiti dell'API

Molti componenti sono pubblicati su GitHub. Senza accesso, l'API di GitHub consente 60 richieste l'ora; con l'account del kit ([6.1 L'account GitHub del kit](#61-laccount-github-del-kit)) o con una variabile GITHUB_TOKEN del PC, 5.000. Il kit conserva le risposte e, se il limite è raggiunto, legge l'ultima versione direttamente dalle pagine di GitHub.

#### Altri servizi che l'app contatta

| Servizio | Perché |
| --- | --- |
| `api.github.com/repos/nabre/atollo/releases` | Gli aggiornamenti di Atollo. |
| `open-vsx.org` | Le estensioni dell'editor. |
| `api.anthropic.com · chatgpt.com · cloudcode-pa.googleapis.com` | I limiti dei piani degli agenti, con l'accesso salvato, solo in lettura e solo con la pagina Agenti aperta. |
| `huggingface.co` | I modelli locali: la ricerca dei file GGUF e i download, che huggingface.co reindirizza alla sua CDN. Solo quando scarichi un modello; il token, se c'è, va solo a huggingface.co. |
| `github.com/ggml-org/llama.cpp · github.com/earendil-works/pi` | Le release di llama.cpp e di Pi, come per gli altri componenti pubblicati su GitHub (sono nella tabella delle fonti qui sotto). |
| `github.com/nabre/atollo/issues` | «Segnala un problema» apre qui una segnalazione nuova nel browser, con versione e sistema nell'indirizzo. Lo apri tu, nel tuo browser: l'app non manda niente, e nulla arriva a GitHub finché non crei la segnalazione. |
| `registry.npmjs.org · pypi.org · packagist.org…` | Solo quando lo chiedono i comandi dei progetti e dei framework. |

#### Tabella completa delle fonti (39 componenti e 5 parti)

Per ogni componente: da dove il kit legge le versioni, da dove scarica il file, come lo verifica e dove lo mette. Le colonne descrivono Windows; dove macOS e Linux usano un'altra fonte, è scritta sotto l'indirizzo.

| Componente | Versioni da | File da | Verifica | In `data\apps\<sistema>\` |
| --- | --- | --- | --- | --- |
| [PowerShell 7 (pwsh)](https://github.com/PowerShell/PowerShell) | api.github.com/repos/PowerShell/PowerShell/releases/latest | PowerShell-\<v\>-win-(x64\|arm64).zip | SHA-256 — GitHub Release asset.digest | `pwsh\<linea>\pwsh.exe` |
| [GitHub CLI (gh)](https://cli.github.com) | api.github.com/repos/cli/cli/releases/latest | gh_\<v\>_windows_(amd64\|arm64).zip | SHA-256 — GitHub Release asset.digest | `gh\<linea>\bin\gh.exe` |
| [Editor (VSCodium Reh Web)](https://vscodium.com) | Catalogo integrato (versione fissata 1.135.06055, D-58) | github.com/VSCodium/vscodium/releases/download/\<v\>/vscodium-reh-web-win32-x64-\<v\>.tar.gz<br>macOS e Linux: vscodium-reh-web-\<target\>-\<v\>.tar.gz, SHA-256 fissato per target (nativo anche su ARM64) | SHA-256 — SHA-256 cablato in components.json | `vscodium-server\` |
| [ConPTY (conpty.dll + OpenConsole.exe)](https://github.com/microsoft/terminal) | Catalogo integrato (versione fissata 1.25.260930003, D-148) | api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/\<v\>/microsoft.windows.console.conpty.\<v\>.nupkg<br>Solo Windows: su macOS e Linux il terminale usa il PTY del sistema | Authenticode + SHA — SHA-256 del pacchetto cablato in components.json (SHA-512 di NuGet); conpty.dll e OpenConsole.exe firmati Microsoft Corporation | `conpty\` |
| [Claude Code](https://claude.com/product/claude-code) | registry.npmjs.org/@anthropic-ai/claude-code-win32-\<arch\>/latest | Campo dist.tarball del pacchetto npm | Authenticode + SHA — Integrità npm sha512 + firma Authenticode Anthropic | `claude\<linea>\claude.exe` |
| [Codex (OpenAI)](https://github.com/openai/codex) | registry.npmjs.org/@openai/codex/latest → /\<ver\>-win32-\<arch\> | Campo dist.tarball del pacchetto specifico per architettura | npm sha512 — Integrità npm dist.integrity (sha512-...) | `codex\<linea>\bin\codex.exe` |
| [Antigravity (Google)](https://antigravity.google) | antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_(amd64\|arm64).json | Campo url del manifesto JSON ufficiale<br>macOS e Linux: manifests/\<os\>_\<arch\>.json, archivio .tar.gz | SHA-512 — SHA-512 (128 hex) dal campo sha512 del manifesto | `antigravity\<linea>\agy.exe (via data\kit\bin\agy.cmd)` |
| [Pi (Earendil)](https://pi.dev) | api.github.com/repos/earendil-works/pi/releases/latest | github.com/earendil-works/pi/releases/download/v\<v\>/pi-windows-\<arch\>.zip<br>macOS e Linux: pi-\<os\>-\<arch\>.tar.gz della stessa release | SHA-256 — Digest SHA-256 di GitHub (o SHA256SUMS della release) | `pi\<linea>\pi.exe` |
| [llama.cpp (llama-server)](https://github.com/ggml-org/llama.cpp) | build bNNNNN fissata nel catalogo di Atollo (D-151) | github.com/ggml-org/llama.cpp/releases/download/\<b\>/llama-\<b\>-bin-win-vulkan-\<arch\>.zip<br>Linux: llama-\<b\>-bin-ubuntu-vulkan-\<arch\>.tar.gz; macOS: llama-\<b\>-bin-macos-\<arch\>.tar.gz (Metal) | SHA-256 — SHA-256 per piattaforma fissato nel catalogo (dal digest di GitHub) | `llama\llama-server.exe` |
| [Git + Git Bash (PortableGit)](https://git-scm.com) | api.github.com/repos/git-for-windows/git/releases/latest | PortableGit-\<v\>-(64-bit\|arm64).7z.exe (estratto in Rust)<br>macOS e Linux: GitHub desktop/dugite-native (dugite-native-v\<v\>-\<sha\>-\<sistema\>.tar.gz, .sha256 dell'asset), segue il kit | SHA-256 — GitHub Release asset.digest | `git\<linea>\cmd\git.exe e git\<linea>\bin\bash.exe` |
| [Python + pip](https://www.python.org) | raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json | cpython-3.\<minor\>.\<patch\>+\<date\>-\<triple\>-install_only_stripped.tar.gz | SHA-256 — SHA-256 dall'elenco ufficiale SHA256SUMS | `python\<linea>\python.exe (+ pip)` |
| [Node.js + npm](https://nodejs.org) | nodejs.org/dist/index.json (LTS pari \>= 18 + ultima major) | nodejs.org/dist/\<v\>/node-\<v\>-win-\<arch\>.zip | SHA-256 — SHA-256 da nodejs.org/dist/\<v\>/SHASUMS256.txt | `node\<linea>\node.exe` |
| [uv (uv, uvx, uvw)](https://docs.astral.sh/uv/) | api.github.com/repos/astral-sh/uv/releases/latest | uv-\<triple\>.zip | SHA-256 — GitHub Release asset.digest | `uv\<linea>\uv.exe, uvx.exe, uvw.exe` |
| [PHP (Thread Safe)](https://www.php.net) | windows.php.net/downloads/releases/releases.json (+ archivi EOL 8.1..5.6) | windows.php.net/downloads/releases/php-\<v\>-Win32-vs\*-x64.zip<br>macOS e Linux: static-php-cli «bulk», archivi cli e fpm con SHA-256 fissato nel catalogo (8.4 e 8.5) | SHA-256 — SHA-256 da releases.json (o tabella EOL verificata) | `php\<linea>\php.exe` |
| [↳ Parte: Composer](https://getcomposer.org) | getcomposer.org/versions (canale stable o 2.2 LTS per PHP \< 7.2.5) | getcomposer.org/download/\<v\>/composer.phar | SHA-256 — SHA-256 da \<url\>.sha256sum | `php\<linea>\composer.phar (+ composer.cmd)` |
| [↳ Parte: Bundle CA SSL (curl)](https://curl.se) | curl.se/ca/cacert.pem.sha256 | curl.se/ca/cacert.pem | SHA-256 — SHA-256 da cacert.pem.sha256 | `php\<linea>\extras\ssl\cacert.pem` |
| [↳ Parte: Visual C++ Runtime](https://learn.microsoft.com) | Estratto dal CAB `vcRuntimeMinimum_amd64\cab1.cab` se assente | aka.ms/vc14/vc_redist.x64.exe<br>Solo Windows | Authenticode + SHA — Firma Authenticode O=Microsoft Corporation su exe e vcruntime140.dll | `tools\vcruntime\ (copiato in PHP, Apache, MongoDB)` |
| [Apache httpd](https://httpd.apache.org) | apachelounge.com/download/ (scraping versione e checksum) | apachelounge.com/download/VS\<nn\>/binaries/httpd-\<v\>-Win64-VS\<nn\>.zip<br>macOS e Linux: build del kit, github.com/nabre/atollo/releases/download/httpd-v\<v\>/httpd-\<v\>-\<target\>.tar.gz, SHA-256 fissato per target nel catalogo, PHP con php-fpm (D-116) | SHA-256 — SHA-256 da \<url\>.txt (SHA256-Checksum for:) | `apache\<linea>\bin\httpd.exe` |
| [Caddy](https://caddyserver.com) | api.github.com/repos/caddyserver/caddy/releases?per_page=100 | Asset caddy_\<v\>_windows_amd64.zip dalla release GitHub ufficiale | SHA-256 — GitHub Release asset.digest (dalla 2.10.1), altrimenti SHA-512 da caddy_\<v\>_checksums.txt | `caddy\<linea>\caddy.exe` |
| [Rust (rustup, cargo, clippy)](https://www.rust-lang.org) | static.rust-lang.org/dist/channel-rust-stable.toml (\[pkg.rustc\]) | static.rust-lang.org/rustup/dist/\<triple\>/rustup-init.exe<br>Host gnu su Windows x64, gnullvm su Windows ARM64 (con C/C++), msvc dalla pagina di Rust in Strumenti («Dettagli»); macOS e Linux: rustup con l'host nativo | SHA-256 — SHA-256 da \<url\>.sha256 | `rust\cargo\bin\rustc.exe e rustup.exe` |
| [Go SDK](https://go.dev) | go.dev/dl/?mode=json (&include=all) | go.dev/dl/go\<v\>.windows-(amd64\|arm64).zip | SHA-256 — SHA-256 dal campo sha256 del JSON ufficiale | `go\<linea>\bin\go.exe` |
| [Java JDK (Eclipse Temurin)](https://adoptium.net) | api.adoptium.net/v3/info/available_releases + /v3/assets/latest/\<major\>/hotspot | Pacchetto .zip restituito dall'API Adoptium | SHA-256 — SHA-256 da binary.package.checksum | `java\<linea>\bin\java.exe` |
| [↳ Parte: Apache Maven](https://maven.apache.org) | repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml | apache-maven-\<v\>-bin.zip | SHA-512 — SHA-512 da \<url\>.sha512 | `maven\bin\mvn.cmd` |
| [↳ Parte: Gradle](https://gradle.org) | services.gradle.org/versions/current | Campo downloadUrl del JSON ufficiale | SHA-256 — SHA-256 da checksumUrl del JSON | `gradle\bin\gradle.bat (via data\kit\bin\gradle.cmd)` |
| [Kotlin Compiler](https://kotlinlang.org) | api.github.com/repos/JetBrains/kotlin/releases?per_page=100 | Asset kotlin-compiler-\<v\>.zip | SHA-256 — GitHub asset.digest o \<asset\>.sha256 | `kotlin\<linea>\bin\kotlinc.bat` |
| [Android SDK + Command-line Tools](https://developer.android.com/studio) | developer.android.com/studio (tabella commandlinetools-win-\*_latest.zip) | dl.google.com/android/repository/commandlinetools-win-\<build\>_latest.zip | SHA-256 — SHA-256 pubblicato nella tabella ufficiale Android | `android\cmdline-tools\latest\bin\sdkmanager.bat e adb.exe` |
| [.NET SDK](https://dotnet.microsoft.com) | builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json | Archivio .zip per rid=win-x64\|win-arm64 | SHA-512 — SHA-512 dal campo hash di releases.json | `dotnet\<linea>\dotnet.exe (con junction SDK condivise)` |
| [Dart SDK](https://dart.dev) | storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION | dartsdk-windows-\<arch\>-release.zip | SHA-256 — SHA-256 da \<url\>.sha256sum | `dart\<linea>\bin\dart.exe` |
| [Zig](https://ziglang.org) | ziglang.org/download/index.json | Campo tarball per (x86_64\|aarch64)-windows | SHA-256 — SHA-256 dal campo shasum del JSON | `zig\<linea>\zig.exe` |
| [Julia](https://julialang.org) | julialang-s3.julialang.org/bin/versions.json | Archivio .zip per os=winnt, arch=x86_64 | SHA-256 — SHA-256 dal campo sha256 di versions.json | `julia\<linea>\bin\julia.exe` |
| [Bun](https://bun.sh) | api.github.com/repos/oven-sh/bun/releases | Asset bun-windows-(x64\|aarch64).zip | SHA-256 — GitHub asset.digest o SHASUMS256.txt | `bun\<linea>\bun.exe` |
| [Deno](https://deno.com) | api.github.com/repos/denoland/deno/releases | Asset deno-(x86_64\|aarch64)-pc-windows-msvc.zip | SHA-256 — GitHub asset.digest o \<asset\>.sha256sum | `deno\<linea>\deno.exe` |
| [Typst](https://typst.app) | api.github.com/repos/typst/typst/releases | Asset typst-(x86_64\|aarch64)-pc-windows-msvc.zip | SHA-256 — GitHub Release asset.digest | `typst\<linea>\typst.exe` |
| [TinyTeX (LaTeX)](https://yihui.org/tinytex/) | api.github.com/repos/rstudio/tinytex-releases/releases/latest | Asset TinyTeX-1-windows-v\*.exe (7z SFX estratto in Rust)<br>macOS e Linux: TinyTeX-1-\<os\>-\<arch\>-v\*.tar.xz (universale su macOS) | SHA-256 — GitHub Release asset.digest | `tinytex\bin\windows\tlmgr.bat e pdftex.exe` |
| [pnpm + yarn (corepack)](https://pnpm.io) | registry.npmjs.org/pnpm/latest e /corepack/latest | Installato con npm-cli.js del Node.js del kit | npm sha512 — Integrità del registro npm | `data\npm-global\pnpm.cmd e yarn.cmd` |
| [CMake + Ninja](https://cmake.org) | GitHub API: Kitware/CMake e ninja-build/ninja (releases/latest) | cmake-\*-windows-(x86_64\|arm64).zip e ninja-win(arm64).zip | SHA-256 — GitHub Release asset.digest | `cmake\<linea>\bin\cmake.exe e tools\ninja.exe` |
| [C/C++ (llvm-mingw)](https://github.com/mstorsjo/llvm-mingw) | api.github.com/repos/mstorsjo/llvm-mingw/releases/latest | Asset llvm-mingw-\*-ucrt-(x86_64\|aarch64).zip<br>macOS e Linux: shim cc, c++, gcc, g++, clang sullo zig cc del kit (richiede Zig) e GNU make da conda-forge, non rilocabile | SHA-256 — GitHub Release asset.digest | `cc\bin\clang.exe (+ make.cmd)` |
| [Ruby + Bundler](https://www.ruby-lang.org) | api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30 | Asset rubyinstaller-\<v\>-(x64\|arm).7z<br>macOS e Linux: conda-forge ruby (3.2 e successive) con micromamba, non rilocabile | SHA-256 — GitHub Release asset.digest | `ruby\<linea>\bin\ruby.exe` |
| [R (conda-forge + OpenBLAS)](https://www.r-project.org) | api.anaconda.org/package/conda-forge/r-base + mamba-org/micromamba-releases | micromamba-win-64 + pacchetti r-base e libblas=\*=\*openblas<br>Windows ARM64: i pacchetti win-64 in emulazione x64, da micromamba nativo (--platform win-64). macOS e Linux: gli stessi pacchetti per osx-\* e linux-\*, lanciatori in kit-bin, non rilocabile | SHA-256 — SHA-256 di micromamba + SHA-256 pacchetti conda-forge | `r\<linea>\bin\R.cmd e Rscript.cmd` |
| [MySQL Community Server](https://www.mysql.com) | Linee 8.4 e 9.7 LTS fissate nel catalogo (sha256ByLine) | cdn.mysql.com/Downloads/MySQL-\<linea\>/mysql-\<v\>-winx64.zip | SHA-256 — SHA-256 fissato per linea e target dopo la firma GPG di Oracle | `mysql\<linea>\bin\mysqld.exe e mysql.exe` |
| [MongoDB + mongosh + Database Tools](https://www.mongodb.com) | downloads.mongodb.org/current.json, GitHub mongodb-js/mongosh, tools/db/release.json | Archivi .zip Windows x64 di mongod, mongosh e mongodump/mongorestore<br>macOS: archivi del fornitore; Linux: server da conda-forge (non rilocabile), mongosh generico, Database Tools per distribuzione | SHA-256 — SHA-256 dai JSON MongoDB + asset.digest GitHub per mongosh | `mongodb\<linea>\bin\mongod.exe, mongosh\, mongodb-tools\` |
| [phpMyAdmin (per MySQL)](https://www.phpmyadmin.net) | www.phpmyadmin.net/home_page/version.json | files.phpmyadmin.net/phpMyAdmin/\<v\>/phpMyAdmin-\<v\>-all-languages.zip | SHA-256 — SHA-256 dal file .sha256 ufficiale + controllo sintattico php -l | `phpmyadmin\<linea>\index.php` |
| [mongo-express (per MongoDB)](https://github.com/mongo-express/mongo-express) | versione fissata nel catalogo di Atollo (build del kit, D-147) | github.com/nabre/atollo/releases/download/mongo-express-v\<v\>/mongo-express-\<v\>.zip | SHA-256 — SHA-256 fissato nel catalogo (o il .sha256 della release del kit); build in CI dal commit fissato di mongo-express | `mongo-express\<linea>\app.js` |
| [SQLite CLI](https://sqlite.org) | sqlite.org/download.html (righe CSV PRODUCT,...) | sqlite.org/\<anno\>/sqlite-tools-win-(x64\|arm64)-\<v\>.zip<br>Linux ARM64: conda-forge sqlite (sqlite.org non pubblica gli strumenti), non rilocabile; Windows ARM64: lo zip x64 se manca quello arm64 | SHA3-256 — SHA3-256 pubblicato nella riga PRODUCT della pagina ufficiale | `sqlite\<linea>\sqlite3.exe` |

<details>
<summary>Sotto il cofano</summary>

- Cartella di lavoro: `<temporanei del sistema>\Atollo\<id del kit>\setup`, oppure `data\tmp\setup` se il disco di sistema ha meno di 4 GB liberi o non si può scrivere.
- Tre tentativi a 5 e 10 secondi di distanza; il download riparte da capo a ogni tentativo.
- Connessioni con il proxy delle Impostazioni e con i certificati del sistema più quelli di «Certificati aggiuntivi (file PEM)»: l'ispezione HTTPS di un firewall funziona se il suo certificato è installato nel PC o è in quel file.
- La versione installata viene scritta in `data\app\versions\<sistema>\<componente>.txt`.

</details>

Argomenti collegati: [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale), [6.1 L'account GitHub del kit](#61-laccount-github-del-kit)

### 8.5 La riga di comando

*I comandi dell'eseguibile di Atollo per preparare, controllare e sincronizzare il kit senza aprire la finestra.*

> **In breve**
>
> - L'eseguibile di Atollo è anche un programma a riga di comando: con un comando, fa il lavoro e esce, senza aprire la finestra.
> - Utile per preparare un kit per un altro sistema, controllare i componenti, o vedere l'ambiente.
> - Funziona anche mentre l'app è aperta.

#### Come si usa

Apri una console nella cartella del kit. Su Windows, in PowerShell, aggiungi | Out-Host perché l'output compaia nella console (l'eseguibile è un programma con finestra); nel Prompt dei comandi non serve. Su macOS e Linux usa il percorso del programma (per esempio ./Atollo.AppImage).

**Esempio**

```
cd D:\Atollo
.\Atollo.exe components | Out-Host
.\Atollo.exe prepare --only node,php --line php=8.3 | Out-Host
.\Atollo.exe check | Out-Host
```

#### I comandi

| Comando | Che cosa fa |
| --- | --- |
| `env [--project <nome>]` | Stampa l'ambiente del kit (segreti mascherati); con un progetto, con le sue versioni. |
| `components [--target <sistema>]` | Elenca i componenti: scelto o no, gruppo, versione installata o perché non è disponibile. |
| `versions [<id>]` | Senza id: le versioni su disco degli strumenti installati. Con un id: chiede alla fonte le versioni offerte e le salva. |
| `check` | Chiede a ogni componente installato la sua versione e segna ✓ o ✗. |
| `check --kit` | La salute del kit: disco, spazio, percorso, permessi, librerie, come «Salute del kit» nelle Impostazioni. Una riga per controllo (√, ! o x) con sotto che cosa fare; finisce con 1 se un controllo è un errore. |
| `prepare [opzioni]` | Installa, aggiorna e toglie i componenti secondo la scelta. Le opzioni sono nella tabella sotto. |
| `sync [--project <cartella>] [--verbose]` | Allinea istruzioni, skill e server MCP degli agenti. |
| `serve <cartella> <porta>` | Serve una cartella come sito statico (il server di file del kit). |
| `keyring restore \| save` | Usato dal kit attorno ad Antigravity: rimette o salva il suo accesso. |
| `models` | I modelli locali: quelli del catalogo e quelli scaricati. |
| `models pull <modello>` | Scarica un id del catalogo o un modello di Hugging Face (proprietario/nome:quantizzazione); riprende se interrotto. |
| `models rm <id>` | Cancella un modello scaricato. |
| `llm start \| stop \| status` | Il server dei modelli locali (chat e agenti): lo avvia, lo ferma o dice se risponde. status non manda la chiave a chi ascolta sulla porta. |
| `diagnostics` | Crea lo zip da allegare a una segnalazione (segreti mascherati) in `data\app\diagnostics`, come «Esporta diagnostica» nelle Impostazioni, e ne stampa il percorso. |
| `version · help` | La versione (anche --version, -V) e l'aiuto (anche --help, -h, /?). |

| Opzione di prepare | Effetto |
| --- | --- |
| `--only a,b` | Solo questi componenti (più ciò di cui hanno bisogno), senza togliere gli altri. |
| `--skip a,b` | Tutti tranne questi (gli obbligatori non si saltano). |
| `--update-all` | Aggiorna anche ciò che è già installato. |
| `--force` | Reinstalla anche se la versione è già quella. |
| `--line id=x.y` | Aggiunge quella versione accanto alle altre (si può ripetere). |
| `--target <sistema>` | Prepara per un altro sistema: win32-x64, win32-arm64, darwin-arm64, darwin-x64, linux-x64, linux-arm64. |

> [!WARNING]
> **Attenzione**
>
> Senza --only, prepare toglie i componenti non scelti, anche dalle cartelle degli altri sistemi. Se non sei sicuro, guarda prima components.

<details>
<summary>Sotto il cofano</summary>

| Codice di uscita | Significato |
| --- | --- |
| `0` | Tutto bene. |
| `1` | prepare o check: qualcosa non è riuscito. |
| `2` | Comando sconosciuto (viene stampato anche l'aiuto). |
| `3` | prepare: è già in corso un'altra preparazione dello stesso kit. |

Dalla riga di comando prepare usa GITHUB_TOKEN o GH_TOKEN della console, se ci sono, per alzare il limite dell'API di GitHub; non usa l'account del kit. Se un download fallisce stampa l'indirizzo da scaricare a mano e la cartella in cui metterlo.

</details>

Argomenti collegati: [5.1 La pagina Strumenti](#51-la-pagina-strumenti), [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella), [8.1 Portabile e isolato: che cosa vuol dire](#81-portabile-e-isolato-che-cosa-vuol-dire)

## 9. Aiuto

### 9.1 Quando qualcosa non va

*Su un PC nuovo, prima la «Salute del kit». Poi i problemi più comuni, il messaggio che vedi e che cosa fare: download, porte occupate, server che non partono, accessi scaduti. E come segnalare un problema con lo zip della diagnostica.*

> **In breve**
>
> - Cerca il messaggio che vedi nella casella qui sotto: la guida ne raccoglie i più comuni, con il rimedio.
> - Quasi ogni errore ha un suggerimento al passaggio del mouse e una riga di output o di log che dice di più.
> - Il log dell'app è in `data\app\logs\app.log`; quelli dei server accanto.

#### Salute del kit

Su un PC nuovo, o con il kit su un'altra chiavetta, guarda prima Impostazioni › «Il kit» › «Salute del kit»: una riga per controllo, verde se va bene, gialla per un avviso, rossa per un errore, e sotto che cosa fare. «Controlla di nuovo» rifà i controlli dopo una correzione.

- Il disco. FAT32 tiene file fino a 4 GB (niente modelli locali né copie di database grandi) e niente collegamenti: per un kit su chiavetta è meglio exFAT, o NTFS se resta su Windows. Su macOS e Linux un disco montato noexec (o FAT32 con showexec su Linux) non lascia partire i programmi: la riga dà il comando per rimontarlo.
- Lo spazio libero: sotto 5 GB un avviso, sotto 1 GB un errore (installazioni, download e copie dei database non riescono). Libera spazio o sposta il kit su un disco più grande.
- Il percorso del kit: &, %, !, ^, apici e lettere accentate o fuori dall'alfabeto latino confondono cmd, PowerShell, Maven e Gradle. Sposta il kit in una cartella con solo lettere, cifre, trattini e spazi.
- File resi privati da un altro account, spesso su un altro PC: il kit non li legge (impostazioni, chiavi, accessi degli agenti). La riga dà il comando per riprenderli, da eseguire in un terminale da amministratore: takeown e icacls su Windows, sudo chown su macOS e Linux. L'app non lo esegue da sé perché servono i diritti di amministratore.
- Su Linux, le librerie del sistema: libaio e libnuma per MySQL, libicu per .NET, libvulkan1 per la scheda video dei modelli locali (senza, restano sulla CPU). La riga dà il comando per installarle, per esempio con apt.
- Dalla riga di comando gli stessi controlli li fa `.\Atollo.exe check --kit | Out-Host`, che finisce con 1 se uno è un errore.

#### Download e installazione

**Un componente resta in «errore» con «non riuscito» e un messaggio di rete o di certificato.**

Spesso è la rete: un proxy aziendale o l'ispezione HTTPS. Compila Impostazioni › Rete e riprova. Se non basta, usa il riquadro «Scaricalo dal browser» sotto la riga e consegna il file al kit.

Approfondisci: [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale)

**«in uso: chiudi i programmi del kit e riprova»**

Un programma del kit (un progetto avviato, una console, l'editor, un database) sta usando i file da sostituire. Fermalo, chiudi le console e riprova. Se non sai che cosa è aperto, la pagina «Rimuovi l'unità» lo elenca.

Approfondisci: [7.1 Rimuovere l'unità in sicurezza](#71-rimuovere-lunità-in-sicurezza)

**«C'è già un'installazione in corso.» oppure «Un'installazione è già in corso»**

Aspetta che finisca (la barra di stato mostra il passo) o premi «Annulla» nella barra «Applica».

**Un componente è «da aggiornare» (triangolo giallo).**

Su macOS e Linux, dopo uno spostamento del kit, alcuni componenti vanno reinstallati: il kit lo fa da solo all'avvio. Se resta così, premi «Aggiorna tutto» in fondo alla pagina Strumenti.

Approfondisci: [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella)

#### Progetti e siti

**«Porta già assegnata a …», «La porta … è già del server/database/progetto …: scegline un'altra.», «Le porte da 7000 a 7099 sono dell'editor» o un programma che dice che la porta è occupata.**

Una porta è di una cosa sola: un'azione di un progetto, un pool della pagina Server Web, una versione di un database o l'editor (7000-7099). Il messaggio dice chi ce l'ha già: «Porta già assegnata a …» un'altra azione, «già del server …» un pool, «già del database …» una versione di MySQL o MongoDB, «già del progetto …» un'azione. Vale per la porta di un'azione, di un pool e di una versione del database. Se invece la porta è di un programma del PC, lo dice quel programma. Per un'azione svuota il campo «Porta» per tornare alla porta automatica; altrimenti scegline un'altra tra 1024 e 65535.

Approfondisci: [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte)

**Un'azione si ferma subito con «errore (codice 1)».**

Leggi le ultime righe dell'output: di solito dicono che cosa manca. Casi frequenti: dipendenze non installate (esegui prima npm install o composer install), un componente mancante (sezione «Componenti» › «Installa ciò che manca»), una versione sbagliata (sezione «Versioni»).

Approfondisci: [2.4 Componenti e versioni di un progetto](#24-componenti-e-versioni-di-un-progetto)

**«Azione non trovata: premi Aggiorna»**

Il progetto è cambiato (per esempio panel.json). Premi «Aggiorna» nella pagina del progetto.

**«panel.json non è un JSON valido: ignorato.»**

C'è un errore di sintassi nel file (una virgola in più, una virgoletta mancante). Correggilo nell'editor e premi «Aggiorna».

Approfondisci: [2.3 Avviare un progetto: azioni e porte](#23-avviare-un-progetto-azioni-e-porte)

**«Il server condiviso … non è partito: guarda la pagina Server Web.»**

Apri la pagina Server Web, scegli il pool e guarda l'errore. Spesso la porta è occupata: cambia porta al profilo, oppure scegli «Server tutto suo» nella sezione «Sito web» del progetto.

Approfondisci: [3.3 Siti web e indirizzi .localhost](#33-siti-web-e-indirizzi-localhost)

**Il sito si apre ma mostra file vecchi o una pagina bianca.**

Controlla la cartella in «Servi come sito web la cartella» (per molti framework PHP è public) e ricarica la pagina del browser ignorando la cache (Ctrl+F5).

**L'editor non si accorge dei file cambiati.**

Su Windows succede se una cartella del percorso del kit inizia con una cifra, con x o con u: l'app lo segnala con un avviso. Sposta il kit in un percorso diverso.

Approfondisci: [1.2 Il primo avvio](#12-il-primo-avvio)

#### Database

**«La porta … è già usata da un altro programma: cambiala qui sotto.»**

Sul PC gira un altro MySQL o MongoDB (installato a parte) sulla stessa porta. Cambia la «Porta» nella pagina del server (per esempio 3307) e avvialo di nuovo; poi riscrivi la connessione nel .env dei progetti con «Scrivi in .env».

Approfondisci: [3.1 I server di database](#31-i-server-di-database)

**Il server resta in «Errore» dopo l'avvio.**

La pagina del server mostra l'ultimo errore e il percorso del log (`data\app\logs`). Su Linux MySQL può chiedere le librerie libaio e libnuma: il messaggio dice il comando per installarle.

**Il progetto non si collega al database.**

Controlla che il server sia «Avviato», poi usa «Scrivi in .env» nella sezione «Database» del progetto: il .env avrà utente, password e porta giusti. Se il progetto usa un'altra versione del server, la porta è diversa (3307…).

Approfondisci: [3.2 Il database di un progetto](#32-il-database-di-un-progetto)

#### Agenti, Git e GitHub

**«accesso scaduto» negli agenti.**

Apri l'agente in un progetto: rinnova l'accesso, o te lo fa rifare.

Approfondisci: [4.1 Gli agenti: Claude Code, Codex, Antigravity, Pi](#41-gli-agenti-claude-code-codex-antigravity-pi)

**Git dice «Please tell me who you are» al primo commit.**

Scrivi nome ed e-mail in Impostazioni › «Account GitHub» › «Chi firma i commit», oppure accedi a GitHub: se sono vuoti, li riempie l'account. In un terminale del kit funziona anche git config --global user.name e user.email.

Approfondisci: [6.1 L'account GitHub del kit](#61-laccount-github-del-kit)

**«GitHub ha rifiutato l'accesso: accedi di nuovo.»**

Il token non è più valido (revocato o scaduto). In Impostazioni › «Account GitHub» premi «Esci» e poi «Accedi a GitHub».

Approfondisci: [6.1 L'account GitHub del kit](#61-laccount-github-del-kit)

#### Avvio dell'app

**«Il disco del kit è in sola lettura: impossibile scrivere nella cartella …»**

La chiavetta è protetta da scrittura (a volte c'è un interruttore) o montata in sola lettura. Il kit deve poter scrivere nella sua cartella.

**macOS: «macOS ha aperto Atollo da una copia temporanea in sola lettura (App Translocation)».**

Succede con le app scaricate e non autenticate. Il messaggio mostra un comando xattr: eseguilo nel Terminale e riapri Atollo.

Approfondisci: [7.2 Su un altro PC, un'altra lettera, un'altra cartella](#72-su-un-altro-pc-unaltra-lettera-unaltra-cartella)

**Linux: il disco è montato con noexec, o manca WebKitGTK.**

Il messaggio spiega che cosa fare: rimonta il disco con il permesso di esecuzione, o installa i pacchetti indicati.

**Windows: manca WebView2.**

WebView2 è il componente di Windows che disegna la finestra; c'è già in Windows 11 e quasi sempre in Windows 10. Se manca, Atollo mostra il collegamento per installarlo.

**Avviso: «Su questo PC sono aperti programmi di un'altra copia del kit».**

Solo un'informazione: un altro kit è aperto. Le due copie non condividono niente.

#### Se il problema non è qui

- Leggi l'output dell'azione (sezione «Avvio») o il log del server: l'ultima riga di solito dice la causa.
- Chiudi Atollo e riaprilo: molte cose (ambiente, agenti, file del kit) vengono rigenerate a ogni avvio.
- Controlla i componenti con «Verifica» nella pagina Strumenti, o con `.\Atollo.exe check | Out-Host`.
- Se ancora non va, segnalalo: la sezione qui sotto dice come.

#### Segnalare un problema

In Impostazioni › «Il kit» › «Segnalare un problema» ci sono due pulsanti, da usare in quest'ordine.

- «Esporta diagnostica» crea sul PC lo zip atollo-diagnostica-\<data-ora\>.zip in `data\app\diagnostics`: versione e sistema, disco del kit (file system, tipo, spazio libero), RAM, ambiente del kit, la verifica dei programmi, i componenti, le porte e la parte finale di ogni log. «Apri la cartella» lo mostra. Dalla riga di comando fa lo stesso `.\Atollo.exe diagnostics | Out-Host`.
- Prima di scrivere lo zip, ogni segreto che il kit conserva (le chiavi di `data\app\secrets`, il token di Claude Code, le password dei database, gli accessi degli agenti, il token di GitHub) diventa \*\*\* ovunque compaia; la cartella del tuo profilo diventa ~, il tuo nome utente e il nome del PC \*\*\*. Aprilo e guardalo comunque prima di allegarlo: un progetto può scrivere nei log cose che il kit non conosce.
- «Segnala un problema» apre nel browser una segnalazione nuova su github.com/nabre/atollo con il modulo dei problemi: versione di Atollo e sistema (kit, build di Windows, file system e tipo del disco) sono già scritti. Racconta che cosa è successo e i passi per rifarlo, e trascina lo zip nella segnalazione.
- L'app non manda niente: lo zip resta sul PC finché non lo alleghi tu, e la segnalazione arriva a GitHub solo quando premi «Create» (serve un account GitHub). Da lì valgono le regole sulla privacy di GitHub, e la segnalazione è pubblica.

Argomenti collegati: [9.2 Domande frequenti](#92-domande-frequenti), [6.3 Rete aziendale: proxy, certificati, rete locale](#63-rete-aziendale-proxy-certificati-rete-locale), [8.3 Processi in background](#83-processi-in-background)

### 9.2 Domande frequenti

*Risposte brevi alle domande che tornano più spesso.*

> **In breve**
>
> - Risposte brevi; ognuna rimanda all'argomento che spiega di più.

#### Domande frequenti

**Funziona senza Internet?**

Sì, per tutto ciò che è già installato: progetti, database, siti, editor, console. Servono Internet i download dei componenti, i comandi che scaricano pacchetti (npm, Composer, pip) e gli agenti IA, che parlano con i server del fornitore.

**Quanto costa?**

Atollo è gratuito e open source (licenza MIT), come i componenti che scarica. Gli agenti IA richiedono un account o una chiave API del fornitore, che può essere a pagamento.

**Il mio codice va da qualche parte?**

Il kit non manda niente: niente telemetria, e gli strumenti hanno la telemetria spenta. Quando usi un agente IA, però, il codice che gli fai leggere viene inviato al fornitore del modello (Anthropic, OpenAI, Google), secondo le sue condizioni. Il riassunto è in Impostazioni › «Privacy», con il link «Leggi l'informativa sulla privacy».

**Come disinstallo Atollo?**

Chiudi l'app e cancella la cartella del kit (dopo aver messo al sicuro i progetti che ti servono). Non ci sono voci di registro né programmi installati da togliere. La cartella temporanea del sistema viene pulita alla chiusura.

**Posso tenere i progetti fuori da projects?**

La pagina Progetti mostra solo le cartelle dentro projects. Per lavorare con gli strumenti del kit su una cartella qualsiasi del PC usa la pagina Terminali ([3.4 Terminali del kit](#34-terminali-del-kit)), che apre console ovunque.

**Posso usare un altro editor?**

I progetti sono cartelle normali: puoi aprirle con qualsiasi editor. Ma gli strumenti del kit (Node, PHP…) e le versioni del progetto sono pronti solo nelle console e nell'editor del kit, perché è il kit a preparare il loro ambiente.

**Atollo sostituisce Docker?**

No, e non lo include. Usa programmi nativi per ogni sistema, più leggeri e senza amministratore. Se un progetto ha un compose.yaml o un Dockerfile, il kit li legge solo per suggerire le versioni giuste (PHP, Node, MySQL…).

**Posso avere due kit?**

Sì: ogni cartella è un kit indipendente, con i suoi progetti, accessi e impostazioni, e due kit possono essere aperti insieme. Lo stesso kit, invece, si apre una volta sola.

**Funziona su un disco di rete?**

Sì, ma è più lento: i programmi leggono molti piccoli file. Con «Temporanei dei programmi» su «Automatico», i file temporanei vanno comunque sul disco del PC.

**Quanto spazio serve?**

La scelta iniziale occupa circa 1,8 GB. Ogni linguaggio aggiunge da qualche decina a qualche centinaio di MB (l'Android SDK e TinyTeX di più), ogni versione in più altrettanto. I progetti con le loro dipendenze (node_modules, vendor) crescono nel tempo. La pagina Strumenti mostra la dimensione di ogni componente.

**Gli strumenti si aggiornano da soli?**

Atollo sì (se lasci attivo l'aggiornamento automatico); i componenti no, li aggiorni tu dalla pagina Strumenti. Vedi [7.3 Aggiornamenti e copie di sicurezza](#73-aggiornamenti-e-copie-di-sicurezza).

**Perché la pagina Agenti non apre gli agenti?**

Perché un agente lavora sempre in un progetto: si apre dalla pagina Progetti, con il pulsante che porta il logo dell'agente. La pagina Agenti serve per accesso, limiti e cartella condivisa.

> [!TIP]
> **Consiglio**
>
> Non trovi la risposta? Prova [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va) o la casella di ricerca della guida.

Argomenti collegati: [9.1 Quando qualcosa non va](#91-quando-qualcosa-non-va), [9.3 Glossario](#93-glossario), [1.1 Benvenuto in Atollo](#11-benvenuto-in-atollo)

### 9.3 Glossario

*Le parole della guida spiegate in modo semplice, dalla A alla Z.*

> **In breve**
>
> - Le parole che tornano nella guida e nell'app, spiegate senza dare niente per scontato.

#### Dalla A alla Z

- **.env** — Un file di testo nel progetto con impostazioni e segreti (come la password del database), una riga CHIAVE=valore ciascuna. Non va messo in Git.
- **Agente IA** — Un assistente a riga di comando che lavora nella cartella del progetto: legge, modifica, esegue comandi. Nel kit: Claude Code, Codex, Antigravity, Pi.
- **Aggiunta** — Nella procedura «Da una base», un pezzo in più da aggiungere alla base (Tailwind, React, un database, le prove).
- **Alias di percorso** — Un indirizzo del sito che serve una cartella diversa, per esempio /media → storage/uploads.
- **Azione** — Un comando del progetto con un nome, che si avvia e si ferma dalla sezione «Avvio».
- **Base** — Il punto di partenza di un progetto creato con un framework o un CMS: Laravel, Vite, Django, WordPress…
- **Cartella del kit** — La cartella che contiene l'eseguibile di Atollo, projects e data.
- **Chiave API** — Un codice segreto che permette di usare un servizio (per esempio un modello IA) pagando a consumo, senza abbonamento.
- **Clonare** — Copiare un repository Git sul tuo computer, con tutta la sua storia.
- **Commit** — Un salvataggio nella storia di Git: un insieme di modifiche con un messaggio.
- **Componente** — Un programma che il kit sa installare: un linguaggio, un agente, un database, uno strumento.
- **Console (terminale)** — Una finestra in cui si scrivono comandi. Quella del kit è PowerShell 7 con gli strumenti del kit; su macOS e Linux, se PowerShell non è installato, è bash.
- **Copia (backup)** — Un file con tutto il contenuto di un database, per rimetterlo com'era.
- **Estensione** — Un componente aggiuntivo dell'editor che lo rende capace di capire un linguaggio o di fare cose nuove.
- **exFAT, FAT32, NTFS** — Formati dei dischi. NTFS (Windows) permette di proteggere le cartelle; exFAT è letto da Windows, macOS e Linux; FAT32 non accetta file oltre 4 GB.
- **Framework** — Una struttura pronta per costruire un tipo di applicazione (Laravel per i siti PHP, Django per Python…).
- **GGUF** — Il formato dei file dei modelli locali che usa llama.cpp: un file (a volte in parti) con i pesi del modello compressi secondo una «quantizzazione», per esempio Q4_K_M.
- **Git** — Il programma che tiene la storia delle modifiche di un progetto.
- **GitHub, GitHub CLI** — GitHub è il servizio online che ospita repository Git; GitHub CLI (gh) è il suo programma a riga di comando.
- **Impronta (hash)** — Un codice calcolato dal contenuto di un file: se il file cambia anche di un byte, l'impronta cambia. Serve a verificare che un download sia quello giusto.
- **localhost, 127.0.0.1** — Il nome e l'indirizzo con cui un computer chiama sé stesso. Un nome che finisce in .localhost porta sempre a questo PC.
- **MCP (server MCP)** — Model Context Protocol: un modo standard per dare agli agenti nuovi strumenti, attraverso un programma o un servizio.
- **Modelli locali** — Modelli linguistici che girano sul PC, senza rete e senza account, con il server llama.cpp del kit. Si scaricano dalla pagina «Modelli locali» e li usano Pi e la chat nel browser.
- **Open VSX** — Il catalogo libero di estensioni usato da VSCodium.
- **PATH** — L'elenco di cartelle in cui il sistema cerca i programmi quando scrivi un comando.
- **Pi** — L'agente da terminale di Earendil (Mario Zechner), il quarto del kit: lavora con i modelli locali, senza account, o con un provider in cloud scelto con /login.
- **Pool** — Un server web (Caddy o Apache) condiviso da più progetti, ognuno con il suo nome .localhost e la sua cartella conf.d.
- **Porta** — Un numero che distingue i servizi sullo stesso computer: http://localhost:7100 e http://localhost:7101 sono due siti diversi. Nel kit ogni porta è di una cosa sola (un'azione, un pool, una versione di un database); 7000-7099 sono dell'editor.
- **Profilo server** — Un pool creato da te nella pagina Server Web, con le versioni e la porta che scegli.
- **Progetto** — Una cartella dentro projects. Il kit ne riconosce il tipo dai file.
- **Proxy** — Un server intermedio da cui passa il traffico Internet, comune nelle reti aziendali.
- **Push, pull** — Inviare i commit al repository remoto (push) e riceverne (pull).
- **Repository** — Un progetto gestito con Git, con la sua storia. Può stare sul tuo computer e su GitHub.
- **Skill** — Una procedura scritta che un agente carica quando serve, in una cartella con un file SKILL.md.
- **Token** — Un codice segreto che dimostra un accesso già fatto (GitHub, Claude). Per i modelli IA, «token» è anche l'unità con cui si contano testo e consumi.
- **Versione minore** — Le prime due parti di una versione: di PHP 8.4.3 la versione minore è 8.4. Il kit tiene una cartella per versione minore.
- **VirtualHost** — Un sito dentro un server web, riconosciuto dal nome con cui lo chiami.
- **VSCodium** — La versione libera di Visual Studio Code: l'editor del kit.

Argomenti collegati: [1.1 Benvenuto in Atollo](#11-benvenuto-in-atollo), [9.2 Domande frequenti](#92-domande-frequenti)

La stessa guida è nell'app, pagina Guida, nelle quattro lingue.
