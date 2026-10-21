use super::*;

/// The starter the app writes (`runtime/mongo-express.js`).
const SOURCE: &str = include_str!("../../../runtime/mongo-express.js");

fn launch<'a>(root: &'a Path, password: &'a str, token: &'a str) -> Launch<'a> {
    Launch {
        node: Path::new("node"),
        starter: Path::new("data/kit/mongo-express.js"),
        work: root,
        root,
        host: Ipv4Addr::new(127, 9, 8, 7),
        port: 7401,
        db_port: 27017,
        password,
        token,
    }
}

/// The password of MongoDB's admin and the token reach the starter in its
/// environment only: the starter is the one argument.
#[test]
fn secrets_never_on_the_command_line() {
    let root = std::env::temp_dir();
    let (password, token) = ("s3cret-Pass", "ab".repeat(32));
    let cmd = command(&KitEnv::default(), &launch(&root, password, &token));
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args, ["data/kit/mongo-express.js"]);
    for a in &args {
        assert!(!a.contains(password) && !a.contains(&token), "{a}");
    }
    let env: std::collections::BTreeMap<String, String> = cmd
        .get_envs()
        .filter_map(|(k, v)| {
            Some((
                k.to_string_lossy().into_owned(),
                v?.to_string_lossy().into_owned(),
            ))
        })
        .collect();
    assert_eq!(env["ATOLLO_DB_PASSWORD"], password);
    assert_eq!(env["ATOLLO_ME_TOKEN"], token);
    assert_eq!(env["ATOLLO_ME_HOST"], "127.9.8.7");
    assert_eq!(env["ATOLLO_ME_PORT"], "7401");
    assert_eq!(env["ATOLLO_DB_PORT"], "27017");
    assert_eq!(env["NODE_ENV"], "production");
    assert_eq!(cmd.get_current_dir(), Some(root.as_path()));
}

/// The starter is written with the kit's files, and it keeps what the gate
/// promises: the secrets leave its environment before mongo-express is
/// loaded, mongo-express's own logins and settings from the environment
/// are off, the cookie is HttpOnly and SameSite=Strict, no shared host.
#[test]
fn starter_keeps_the_gate() {
    assert_eq!(STARTER, "mongo-express.js");
    for needle in [
        "ATOLLO_ME_TOKEN|ATOLLO_DB_PASSWORD|ME_CONFIG_",
        "delete env[name]",
        "env.NODE_ENV = 'production'",
        "config.useBasicAuth = false",
        "config.useOidcAuth = false",
        "HttpOnly; SameSite=Strict",
        "auth: { username: 'admin', password }",
        "timingSafeEqual",
    ] {
        assert!(SOURCE.contains(needle), "{needle}");
    }
    // The password is never part of the connection string it may print.
    assert!(!SOURCE.contains("admin:${"));
    assert!(!SOURCE.contains("localhost"), "no shared host accepted");
    let tmp = crate::kit::tests::TempDir::new("mongo-express-starter");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::detect());
    kit.prepare().unwrap();
    crate::setup::kit_files::write_all(&kit).unwrap();
    assert_eq!(
        fs::read_to_string(kit.kit_files().join(STARTER)).unwrap(),
        SOURCE
    );
}

/// The gate's answers, from the starter itself, with the Node.js of the PC
/// (skipped without one): 403 without the token, with a wrong one, with
/// another Host or for a Host without the port; the token in the address
/// gives the cookie; the cookie lets the request through.
#[test]
fn gate_answers() {
    let tmp = crate::kit::tests::TempDir::new("mongo-express-gate");
    let starter = tmp.0.join("mongo-express.js");
    fs::write(&starter, SOURCE).unwrap();
    let token = "c".repeat(64);
    let script = format!(
        r#"const {{ gate }} = require({starter:?});
const self = '127.9.8.7:7401', t = '{token}';
const ask = (url, headers) => gate({{ url, headers }}, self, t);
const out = [
  ask('/', {{ host: self }}),
  ask('/?atollo=wrong', {{ host: self }}),
  ask('/?atollo=' + t, {{ host: self }}),
  ask('/?atollo=' + t, {{ host: 'localhost:7401' }}),
  ask('/?atollo=' + t, {{ host: '127.0.0.1:7401' }}),
  ask('/?atollo=' + t, {{ host: '127.9.8.7' }}),
  ask('/db/admin/', {{ host: self, cookie: 'a=1; atollo_me=' + t }}),
  ask('/db/admin/', {{ host: '127.9.8.7:7401'.toUpperCase(), cookie: 'atollo_me=' + t }}),
  ask('/db/admin/', {{ host: self, cookie: 'atollo_me=' + t.slice(1) }}),
  ask('/db/admin/', {{ host: 'evil.test', cookie: 'atollo_me=' + t }}),
  gate({{ url: '/', headers: {{ host: self, cookie: 'atollo_me=' }} }}, self, ''),
];
console.log(out.join(','));"#,
        starter = starter.to_string_lossy()
    );
    let Ok(out) = std::process::Command::new("node")
        .args(["-e", &script])
        .output()
    else {
        eprintln!("no node on the PATH: skipped");
        return;
    };
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "deny,deny,enter,deny,deny,deny,pass,pass,deny,deny,deny"
    );
}

/// mongo-express of the kit given by ATOLLO_TEST_KIT on its MongoDB
/// already running at ATOLLO_TEST_MONGODB_PORT (an open Atollo): the gate,
/// then the databases listed as admin; stopped, it no longer answers.
#[test]
fn real_mongo_express_on_running_mongodb() {
    let (Ok(root), Ok(db_port)) = (
        std::env::var("ATOLLO_TEST_KIT"),
        std::env::var("ATOLLO_TEST_MONGODB_PORT"),
    ) else {
        eprintln!("ATOLLO_TEST_KIT or ATOLLO_TEST_MONGODB_PORT not set: skipped");
        return;
    };
    let kit = Kit::at(PathBuf::from(root), crate::kit::Arch::detect());
    let env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let password = crate::db::auth::load(&kit)
        .mongodb_admin
        .expect("MongoDB has a password of admin");
    let me = MongoExpress::default();
    let url = me
        .open(&kit, &env, db_port.parse().unwrap(), &password)
        .unwrap();
    // Opened again on the same version: the same server.
    assert_eq!(
        me.open(&kit, &env, db_port.parse().unwrap(), &password)
            .unwrap(),
        url
    );
    assert_logs_in(&url);
    let authority = url.trim_start_matches("http://").split('/').next().unwrap();
    me.stop_for(1);
    assert!(
        std::net::TcpStream::connect(authority).is_ok(),
        "another version: still on"
    );
    me.stop_for(db_port.parse().unwrap());
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(std::net::TcpStream::connect(authority).is_err(), "stopped");
}

/// The checks of a real mongo-express at `url`: 403 without the token, for
/// another Host, with a wrong token; the token gives the cookie and leaves
/// the address; with it the databases are listed and the assets served.
pub fn assert_logs_in(url: &str) {
    let authority = url
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap()
        .to_owned();
    let base = format!("http://{authority}");
    let (_, port) = authority.rsplit_once(':').unwrap();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .into();
    let get = |path: &str, cookie: &str| {
        let mut req = agent.get(&format!("{base}{path}"));
        if !cookie.is_empty() {
            req = req.header("Cookie", cookie);
        }
        let mut answer = req.call().unwrap();
        let code = answer.status().as_u16();
        (code, answer.body_mut().read_to_string().unwrap_or_default())
    };
    assert_eq!(get("/", "").0, 403, "no token, no mongo-express");
    assert_eq!(get("/?atollo=0", "").0, 403, "wrong token");
    for other in ["127.0.0.1", "localhost"] {
        let mut answer = agent
            .get(url)
            .header("Host", &format!("{other}:{port}"))
            .call()
            .unwrap();
        assert_eq!(answer.status().as_u16(), 403, "Host {other}");
        let _ = answer.body_mut().read_to_string();
    }
    let entry = agent.get(url).call().unwrap();
    assert_eq!(entry.status().as_u16(), 302);
    let set_cookie = entry
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_owned();
    assert!(
        set_cookie.contains("HttpOnly") && set_cookie.contains("SameSite=Strict"),
        "{set_cookie}"
    );
    let location = entry.headers().get("location").unwrap().to_str().unwrap();
    assert_eq!(location, "/", "the token leaves the address");
    let cookie = set_cookie.split(';').next().unwrap().to_owned();
    let (code, body) = get("/", &cookie);
    assert_eq!(code, 200, "{body}");
    assert!(body.contains("/db/admin/"), "databases listed: {body}");
    assert_eq!(get("/db/admin/", &cookie).0, 200);
}
