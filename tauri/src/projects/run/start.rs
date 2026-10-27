//! Starting an action: its command, its port, its log, until it runs or ends.

use std::io::Read;
use std::path::Path;
use std::process::{Child, Stdio};
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::common::i18n::t as tr;
use crate::common::ports::responds;
use crate::common::time::now_ms;
use crate::detect::{self, Action, Kind};
use crate::env::KitEnv;
use crate::platform::jobs::{self, Job};

use super::output::{env_on_network, on_network};
use super::{key_of, Entry, Event, RunInfo, Runner, State, PORT_FROM};

impl Runner {
    /// The `sbx exec` of `action` in `vm`, the line shown in its log, what
    /// stops it there ([`VmRun`]) and the lines its log starts with (what
    /// the VM serves differently).
    #[allow(clippy::too_many_arguments)]
    fn in_vm(
        &self,
        vm: &crate::isolation::Vm,
        id: &str,
        dir: &Path,
        cwd: &Path,
        action: &Action,
        env: &KitEnv,
        port: Option<u16>,
        lan: bool,
        fill: &dyn Fn(&str) -> String,
        lang: &str,
    ) -> Result<(std::process::Command, String, VmRun, Vec<String>)> {
        use crate::isolation::vm::{exec_args, guest_path, run_tag, tracked, web_line, Web};
        // Started by `levels::prepare` (D-179): its tools, databases and
        // ports ready; here only the action's port.
        if let Some(p) = port {
            vm.publish(env, p, lan)?;
        }
        let mut notes = Vec::new();
        let rel = cwd
            .strip_prefix(dir)
            .ok()
            .map(|r| r.to_string_lossy().replace('\\', "/"))
            .filter(|r| !r.is_empty());
        let mut vars: Vec<(String, String)> = vec![
            ("BROWSER".into(), "none".into()),
            ("PYTHONUNBUFFERED".into(), "1".into()),
            ("HOST".into(), "0.0.0.0".into()),
        ];
        if let Some(p) = port {
            vars.push(("PORT".into(), p.to_string()));
            vars.push(("ROCKET_PORT".into(), p.to_string()));
        }
        // The database's way (`DB_HOST`, `DB_PORT`…) is in the tools' file
        // (D-171); the kit's tools' variables come before the action's.
        for (k, v) in &action.env {
            vars.push((k.clone(), fill(v)));
        }
        let php_line = crate::isolation::php_line(&self.kit, id);
        let served = action
            .static_dir
            .as_ref()
            .or(action.apache_dir.as_ref())
            .or(action.caddy_dir.as_ref());
        // The site's path aliases of the project itself, as the VM sees
        // them; another project's folder is outside this VM (D-175).
        let mut aliases = Vec::new();
        if served.is_some() {
            let windows = self.kit.target().is_windows();
            for a in crate::projects::alias::resolve(dir, &self.kit.projects(), &action.aliases) {
                match a
                    .in_project()
                    .zip(guest_path(windows, dir))
                    .map(|(rel, root)| format!("{root}/{rel}"))
                {
                    Some(inside) => aliases.push((a.url.clone(), inside)),
                    None => notes.push(tr(
                        lang,
                        "l'alias {0} punta a un altro progetto, che non è nella VM di questo: non servito",
                        &[&a.url],
                    )),
                }
            }
        }
        if action.caddy_dir.is_some() {
            notes.push(tr(
                lang,
                "nella VM il sito usa Apache con PHP {0} al posto di Caddy: le regole proprie di Caddy non valgono",
                &[&php_line],
            ));
        }
        let web = |folder: &str, web: Web| {
            web_line(
                &action.id,
                &web,
                folder,
                port.unwrap_or(PORT_FROM),
                &aliases,
            )
        };
        let line = if let Some(folder) = &action.static_dir {
            web(folder, Web::Static)
        } else if let Some(folder) = action.apache_dir.as_ref().or(action.caddy_dir.as_ref()) {
            web(folder, Web::Php { line: php_line })
        } else {
            fill(&action.command)
        };
        let tag = run_tag(&action.id);
        let args = exec_args(
            &vm.name,
            rel.as_deref(),
            &tracked(&tag, &line),
            &vars,
            vm.tools(),
        );
        let shown = format!("sbx exec {} {line}", vm.name);
        let run = VmRun {
            vm: vm.clone(),
            env: env.clone(),
            tag,
            container: served.map(|_| crate::isolation::vm::web_container(&action.id)),
            port,
            lan,
        };
        Ok((vm.command(env, &args), shown, run, notes))
    }
}

/// What stops an action in its VM: ending its `sbx exec` on the PC leaves
/// it running there (tried with sbx 0.49.2), so the stop goes into the VM
/// and the port goes back.
pub(super) struct VmRun {
    vm: crate::isolation::Vm,
    env: KitEnv,
    tag: String,
    container: Option<String>,
    port: Option<u16>,
    lan: bool,
}

impl VmRun {
    /// The sandbox it runs in.
    pub(super) fn sandbox(&self) -> &str {
        &self.vm.name
    }

    /// Its port published on the PC and the address (`0.0.0.0` with the
    /// local network, else `127.0.0.1`).
    pub(super) fn published(&self) -> Option<(u16, &'static str)> {
        self.port
            .map(|p| (p, if self.lan { "0.0.0.0" } else { "127.0.0.1" }))
    }

    /// Stops it in the VM, off the caller's thread (`sbx exec` takes a
    /// moment, up to the five seconds of a process that ignores `TERM`).
    pub(super) fn stop(self) {
        std::thread::spawn(move || {
            self.vm.stop_run(
                &self.env,
                &self.tag,
                self.container.as_deref(),
                self.port,
                self.lan,
            );
        });
    }
}

/// The value of `key` in the `.env` of `dir` (`KEY=value`, `export KEY=`,
/// quotes taken off), when set there.
pub(crate) fn dotenv_value(dir: &Path, key: &str) -> Option<String> {
    let text = crate::platform::safe_path::read_text(dir, &dir.join(".env"))?;
    text.lines().rev().find_map(|line| {
        let line = line.trim();
        let line = line.strip_prefix("export ").unwrap_or(line);
        let (k, v) = line.split_once('=')?;
        (!k.starts_with('#') && k.trim() == key).then(|| {
            let v = v.trim();
            v.strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(v)
                .to_owned()
        })
    })
}

impl Runner {
    /// Starts `action` of `project` (folder `dir`), on the PC with `env`,
    /// or in the project's microVM when `vm` (D-167, started by
    /// `levels::prepare`; `env` then `sbx`'s): the command line, or the
    /// container of its web server, through `sbx exec`, its port published
    /// on the PC's `127.0.0.1`. Already running: the same process, not a
    /// second one. Called only by `levels::start::start_action` (D-179).
    pub fn start_in(
        self: &Arc<Self>,
        id: &str,
        dir: &Path,
        action: &Action,
        env: &KitEnv,
        lang: &str,
        vm: Option<&crate::isolation::Vm>,
    ) -> Result<RunInfo> {
        let key = key_of(id, &action.id);
        // A second start of the action (a double click during
        // `assign_port`) waits for the first, then finds it running
        // instead of replacing it (BUG-V6-10).
        let gate = self.start_gate(&key);
        let _starting = gate.lock().unwrap_or_else(PoisonError::into_inner);
        if self.alive(&key) {
            return self
                .entries()
                .get(&key)
                .map(|e| e.info.clone())
                .ok_or_else(|| anyhow!("gone"));
        }
        let port = if action.kind == Kind::Web {
            Some(self.assign_port(&key, lang)?)
        } else {
            None
        };
        let fill = |s: &str| s.replace("{port}", &port.map(|p| p.to_string()).unwrap_or_default());
        // Settings > "Permetti ai server dei progetti..." (ATOLLO_ALLOW_LAN).
        let lan = env.get("ATOLLO_ALLOW_LAN") == Some("1");
        let cwd = match &action.folder {
            Some(f) => detect::inside_dir(dir, f)?,
            None => dir.to_path_buf(),
        };

        let job = Job::new()?;
        // The FastCGI server of a project's Caddy (or of its Apache off
        // Windows), watched with the server.
        let mut php: Option<Child> = None;
        // The FastCGI server of a project's Caddy, kept until the action
        // ends: php-fpm's private socket folder goes with its last copy.
        let mut fcgi_kept: Option<crate::projects::fastcgi::FastCgi> = None;
        let mut vm_run = None;
        let mut vm_notes = Vec::new();
        let (mut cmd, shown) = if let Some(vm) = vm {
            let (cmd, shown, run, notes) =
                self.in_vm(vm, id, dir, &cwd, action, env, port, lan, &fill, lang)?;
            vm_run = Some(run);
            vm_notes = notes;
            (cmd, shown)
        } else if let Some(folder) = &action.static_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            // The AppImage, not its inner program: `kit_command` clears
            // APPIMAGE, and the image's mount is not this program's to keep.
            let exe = crate::kit::app_exe().ok_or_else(|| anyhow!("current exe"))?;
            let mut cmd = jobs::kit_command(&exe, env);
            cmd.arg("serve")
                .arg(&root)
                .arg(port.unwrap_or(PORT_FROM).to_string());
            (
                cmd,
                format!("Atollo.exe serve {folder} {}", port.unwrap_or_default()),
            )
        } else if let Some(folder) = &action.apache_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            // Off Windows PHP is php-fpm (D-116), in the action's job and
            // watched with the server like Caddy's; on Windows the module.
            let fcgi = if self.kit.os().is_windows() {
                None
            } else {
                let run_dir = crate::projects::apache::dir(&self.kit, id)?;
                crate::projects::fastcgi::spawn_fastcgi_if_present(
                    &self.kit,
                    env,
                    &job,
                    &run_dir,
                    &format!("project {id}"),
                )
                .map(|(fcgi, child)| {
                    php = Some(child);
                    fcgi
                })
            };
            fcgi_kept = fcgi.clone();
            let cmd = crate::projects::apache::command(
                &self.kit,
                id,
                dir,
                &root,
                port.unwrap_or(PORT_FROM),
                fcgi.as_ref(),
                lan,
                &action.domains,
                &action.aliases,
                env,
                lang,
            )?;
            let shown = if self.kit.os().is_windows() {
                format!("httpd.exe -f data\\projects\\{id}\\apache\\httpd.conf ({folder})")
            } else {
                format!("httpd -f data/projects/{id}/apache/httpd.conf -DFOREGROUND ({folder})")
            };
            (cmd, shown)
        } else if let Some(folder) = &action.caddy_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            // Its PHP (php-cgi, or php-fpm off Windows) lives in the action's
            // job too, watched with the server.
            let run_dir = crate::projects::caddy::dir(&self.kit, id)?;
            let fcgi = crate::projects::fastcgi::spawn_fastcgi_if_present(
                &self.kit,
                env,
                &job,
                &run_dir,
                &format!("project {id}"),
            )
            .map(|(fcgi, child)| {
                php = Some(child);
                fcgi
            });
            // php-fpm's socket folder lives as long as the action.
            fcgi_kept = fcgi.clone();
            let cmd = crate::projects::caddy::command(
                &self.kit,
                id,
                dir,
                &root,
                port.unwrap_or(PORT_FROM),
                fcgi.as_ref(),
                lan,
                &action.domains,
                &action.aliases,
                env,
                lang,
            )?;
            (
                cmd,
                format!("caddy run --config data/projects/{id}/caddy/Caddyfile ({folder})"),
            )
        } else {
            let command = on_network(fill(&action.command), lan);
            (jobs::shell_command(&command, env), command)
        };
        cmd.current_dir(&cwd)
            .env("BROWSER", "none")
            .env("PYTHONUNBUFFERED", "1")
            .env("ATOLLO_LANG", lang)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(p) = port {
            cmd.env("PORT", p.to_string())
                .env("ROCKET_PORT", p.to_string());
        }
        // The port of the MySQL version its database is on
        // (D-102), unless a `.env` sets another one: dotenv loaders
        // keep a variable already there, and the user's own port would be
        // lost.
        if let Some(p) = crate::db::services::project_db_port(&self.kit, id) {
            let p = p.to_string();
            let own = [dir, cwd.as_path()]
                .into_iter()
                .filter_map(|d| dotenv_value(d, "DB_PORT"))
                .any(|v| v != p);
            if !own {
                cmd.env("DB_PORT", p);
                // `mysql` of its scripts logs in on that version too
                // (`ATOLLO_MYSQL_CNF`).
                for (name, value) in crate::db::services::project_client_env(&self.kit, id) {
                    cmd.env(name, value);
                }
            }
        }
        for (k, v) in &action.env {
            cmd.env(k, env_on_network(k, fill(v), lan));
        }

        let url = (action.kind == Kind::Web)
            .then(|| fill(action.url.as_deref().unwrap_or("http://localhost:{port}/")));
        let mut info = RunInfo {
            key: key.clone(),
            project: id.to_owned(),
            action: action.id.clone(),
            label: action.label.clone(),
            kind: action.kind,
            command: shown.clone(),
            port,
            url,
            state: if action.kind == Kind::Web {
                State::Starting
            } else {
                State::Running
            },
            start: now_ms(),
            end: None,
            code: None,
            pid: None,
            skipped_aliases: if action.apache_dir.is_some() {
                crate::projects::alias::skipped(dir, &self.kit.projects(), &action.aliases)
            } else if action.caddy_dir.is_some() {
                crate::projects::caddy::skipped_aliases(dir, &self.kit.projects(), &action.aliases)
            } else {
                Vec::new()
            },
        };
        let mut header = vec![format!("> {shown}")];
        header.push(match port {
            Some(p) => format!(
                "  {}   {}",
                tr(lang, "cartella: {0}", &[&cwd.display().to_string()]),
                tr(lang, "porta: {0}", &[&p.to_string()])
            ),
            None => format!(
                "  {}",
                tr(lang, "cartella: {0}", &[&cwd.display().to_string()])
            ),
        });
        header.extend(vm_notes.iter().map(|n| format!("[Atollo] {n}")));
        header.push(String::new());

        let run = self.next_run();
        let spawned = jobs::spawn_in(&mut cmd, &job);
        let mut child: Child = match spawned {
            Ok(c) => c,
            Err(e) => {
                info.state = State::Error;
                info.end = Some(now_ms());
                header.push(format!(
                    "[!] {}",
                    tr(lang, "Avvio non riuscito: {0}", &[&format!("{e:#}")])
                ));
                let seq = {
                    let mut entries = self.entries();
                    entries.insert(
                        key.clone(),
                        Entry {
                            info: info.clone(),
                            lines: header.clone().into(),
                            job: None,
                            stopped: false,
                            actual_port: None,
                            detected_url: None,
                            pool: None,
                            vm_run: None,
                            run,
                        },
                    );
                    self.number_lines(header.len())
                };
                (self.notify)(Event::Log {
                    key: key.clone(),
                    lines: header,
                    seq,
                });
                (self.notify)(Event::State(info.clone()));
                return Ok(info);
            }
        };
        info.pid = Some(child.id());
        self.entries().insert(
            key.clone(),
            Entry {
                info: info.clone(),
                lines: std::collections::VecDeque::new(),
                job: Some(job),
                stopped: false,
                actual_port: None,
                detected_url: None,
                pool: None,
                vm_run,
                run,
            },
        );
        self.append(&key, header, false);
        (self.notify)(Event::State(info.clone()));

        let mut readers = Vec::new();
        for stream in [
            child
                .stdout
                .take()
                .map(|s| Box::new(s) as Box<dyn Read + Send>),
            child
                .stderr
                .take()
                .map(|s| Box::new(s) as Box<dyn Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let (me, key) = (Arc::clone(self), key.clone());
            readers.push(std::thread::spawn(move || {
                me.read_stream(&key, run, stream)
            }));
        }
        let (me, key2, lang2) = (Arc::clone(self), key.clone(), lang.to_owned());
        std::thread::spawn(move || {
            let _fcgi = fcgi_kept;
            let code = match php {
                Some(php) => me.wait_with_php(&key2, run, &mut child, php, &lang2),
                None => child.wait().ok().and_then(|s| s.code()),
            };
            for r in readers {
                let _ = r.join();
            }
            me.finish(&key2, run, code, &lang2);
        });
        if action.kind == Kind::Web {
            let (me, key3, lang3) = (Arc::clone(self), key.clone(), lang.to_owned());
            std::thread::spawn(move || me.watch_port(&key3, &lang3));
        }
        Ok(info)
    }

    pub(super) fn watch_port(&self, key: &str, lang: &str) {
        loop {
            std::thread::sleep(Duration::from_millis(700));
            let (state, port, actual) = match self.entries().get(key) {
                Some(e) => (e.info.state, e.info.port, e.actual_port),
                None => return,
            };
            if state != State::Starting {
                return;
            }
            if port.is_some_and(responds) {
                self.ready(key, None, lang);
                return;
            }
            if let Some(a) = actual.filter(|a| responds(*a)) {
                self.ready(key, Some(a), lang);
                return;
            }
        }
    }

    pub(super) fn ready(&self, key: &str, actual: Option<u16>, lang: &str) {
        let (info, line) = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            if e.info.state != State::Starting {
                return;
            }
            let mut line = Vec::new();
            if let Some(a) = actual {
                line.push(format!(
                    "[Atollo] {}",
                    tr(
                        lang,
                        "il server usa la porta {0} invece della {1}",
                        &[&a.to_string(), &e.info.port.unwrap_or_default().to_string()]
                    )
                ));
                e.info.url = e.detected_url.clone();
            }
            e.info.state = State::Running;
            line.push(format!(
                "[Atollo] {}",
                tr(
                    lang,
                    "server pronto: {0}",
                    &[e.info.url.as_deref().unwrap_or_default()]
                )
            ));
            (e.info.clone(), line)
        };
        self.append(key, line, false);
        (self.notify)(Event::State(info));
    }

    /// The run `run` of `key` ended with `code`. An entry that is now
    /// another run's (the action started again, or served by a pool) is
    /// left as it is: never closed by the end of the run it replaced
    /// (BUG-V6-10).
    pub(super) fn finish(&self, key: &str, run: u64, code: Option<i32>, lang: &str) {
        let info = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key).filter(|e| e.run == run) else {
                return;
            };
            e.job = None;
            // Ended on the PC: what it ran in the VM too, and its port back.
            if let Some(vm_run) = e.vm_run.take() {
                vm_run.stop();
            }
            e.info.code = code;
            e.info.end = Some(now_ms());
            e.info.state = if e.stopped {
                State::Stopped
            } else if code == Some(0) {
                State::Finished
            } else {
                State::Error
            };
            e.info.clone()
        };
        let line = match code {
            Some(c) => tr(lang, "processo terminato, codice {0}", &[&c.to_string()]),
            None => tr(lang, "processo terminato", &[]),
        };
        self.append_to(key, Some(run), vec![format!("[{line}]")], false);
        (self.notify)(Event::State(info));
    }
}
