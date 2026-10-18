use super::*;
use crate::projects::frameworks::{check_env, official};

fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}
#[test]
fn combinations() {
    let p = official();
    let laravel_react = plan(p, "laravel", &ids(&["react", "tailwind", "nativephp"])).unwrap();
    assert!(laravel_react.create[0].contains("laravel/react-starter-kit"));
    assert_eq!(laravel_react.steps[0], "npm install");
    assert!(laravel_react
        .steps
        .iter()
        .any(|s| s.contains("nativephp/desktop")));
    assert!(laravel_react
        .actions
        .iter()
        .any(|a| a.name == "App desktop"));
    assert!(laravel_react.notes.len() >= 2);

    let electron_vue = plan(p, "electron", &ids(&["vue"])).unwrap();
    assert!(electron_vue.create[0].contains("@quick-start/electron"));
    let start = electron_vue
        .actions
        .iter()
        .find(|a| a.name == "Avvia")
        .unwrap();
    assert_eq!(
        start.command, "npm run dev",
        "the addition replaces the action"
    );
    assert_eq!(electron_vue.actions.len(), 2);

    let tauri = plan(p, "tauri", &[]).unwrap();
    assert_eq!(tauri.needs, ["rust", "node"]);

    // A CMS has its database from the base, and an administrator in .env.
    for cms in ["wordpress", "typo3", "drupal"] {
        let site = plan(p, cms, &[]).unwrap();
        assert_eq!(site.database.as_deref(), Some("mysql"), "{cms}");
        assert_eq!(site.needs, ["php", "mysql"], "{cms}");
        assert!(
            site.env.iter().any(|e| e.value == "{password}"),
            "{cms}: administrator's password"
        );
        assert!(
            site.steps.iter().all(|s| !s.contains("DB_PASSWORD=")),
            "{cms}: passwords only through $env"
        );
    }
    let statamic = plan(p, "statamic", &[]).unwrap();
    assert_eq!(statamic.database, None, "content in files");
    let mut base_db = p.to_vec();
    base_db
        .iter_mut()
        .find(|b| b.id == "laravel")
        .unwrap()
        .database = Some("mongodb".into());
    assert!(
        plan(&base_db, "laravel", &ids(&["mysql"])).is_err(),
        "the base's database and another"
    );

    let mysql = plan(p, "laravel", &ids(&["mysql", "bootstrap"])).unwrap();
    assert_eq!(mysql.database.as_deref(), Some("mysql"));
    assert!(mysql.needs.contains(&"mysql".to_owned()));
    assert!(
        plan(p, "laravel", &ids(&["mysql", "mongodb"])).is_err(),
        "one database"
    );
    for base in ["laravel", "symfony", "vite", "nextjs", "electron", "tauri"] {
        for css in ["tailwind", "bootstrap", "bulma"] {
            plan(p, base, &ids(&[css])).unwrap_or_else(|_| panic!("{css} with {base}"));
        }
    }
    assert!(plan(p, "symfony", &ids(&["vite-bundle"]))
        .unwrap()
        .needs
        .contains(&"node".to_owned()));

    let web_app = plan(p, "vite", &ids(&["react", "tailwind", "capacitor"])).unwrap();
    assert!(web_app.needs.contains(&"android".to_owned()));
    assert!(web_app.steps.iter().any(|s| s.contains("cap add android")));

    let tauri_android = plan(p, "tauri", &ids(&["react", "capacitor"])).unwrap();
    assert!(tauri_android
        .steps
        .iter()
        .any(|s| s.contains("tauri android init")));
    assert!(tauri_android.needs.contains(&"android".to_owned()));

    let mobile = plan(p, "laravel", &ids(&["nativephp-mobile"])).unwrap();
    assert_eq!(mobile.needs, ["php", "node", "java", "android"]);

    assert_eq!(
        plan(p, "laravel", &ids(&["react", "vue"])).unwrap_err().0,
        "{0} e {1} non vanno insieme."
    );
    assert_eq!(
        plan(p, "symfony", &ids(&["react"])).unwrap_err().0,
        "{0} non si usa con questa base."
    );
    assert!(plan(p, "react", &[]).is_err(), "an addition is not a base");
    assert!(plan(p, "vite", &ids(&["nothing"])).is_err());
}

#[test]
fn env_values() {
    assert_eq!(env_value("{name}", "shop", "pw"), "shop");
    assert_eq!(env_value("{name}", "my shop", "pw"), "\"my shop\"");
    assert_eq!(
        env_value("{package}", "my shop", "pw"),
        "com.example.myshop"
    );
    assert_eq!(env_value("{password}", "shop", "Dk-1"), "Dk-1");
    assert_eq!(env_raw("{name} {password}", "my shop", "pw"), "my shop pw");
    let pw = password_of(&"0".repeat(64));
    assert_eq!(pw.len(), 26);
    for want in [
        char::is_ascii_uppercase,
        char::is_ascii_lowercase,
        char::is_ascii_digit,
        char::is_ascii_punctuation,
    ] {
        assert!(pw.chars().any(|c| want(&c)), "{pw}");
    }
    let p = official();
    let plan = plan(p, "laravel", &ids(&["nativephp"])).unwrap();
    let keys: Vec<&str> = plan.env.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, ["APP_NAME", "NATIVEPHP_APP_ID"]);
    let bad = vec![EnvVar {
        key: "1A".into(),
        value: String::new(),
    }];
    assert!(check_env(&bad).is_err());
}

#[test]
fn filled_lines() {
    assert_eq!(
        fill(
            "npm create vite@9 {name} -- --identifier {package}",
            "it's mine"
        ),
        "npm create vite@9 'it''s mine' -- --identifier com.example.itsmine"
    );
}

#[test]
fn package_ids() {
    assert_eq!(package_of("My App 2"), "com.example.myapp2");
    assert_eq!(package_of("2048"), "com.example.a2048");
    assert_eq!(package_of("è"), "com.example.a");
}
