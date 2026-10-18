use super::*;

fn run(dump: &str) -> Result<(String, Report)> {
    let mut out = Vec::new();
    let report = filter_dump(dump.as_bytes(), &mut out, None)?;
    Ok((String::from_utf8(out).unwrap(), report))
}

fn ok(dump: &str) -> String {
    run(dump).unwrap().0
}

#[test]
fn mariadb_comments_go_with_the_statements_they_leave_empty() {
    let dump = "/*M!999999\\- enable the sandbox mode */ \n\
                /*!999999\\- enable the sandbox mode */\n\
                -- MariaDB dump 10.19  Distrib 11.4.5-MariaDB\n\
                /*!40101 SET NAMES utf8mb4 */;\n\
                /*M!100616 SET @OLD_NOTE_VERBOSITY=@@NOTE_VERBOSITY, NOTE_VERBOSITY=0 */;\n\
                SELECT 1 /*M!100100 , 2 */;\n";
    let (out, report) = run(dump).unwrap();
    assert_eq!(
        out,
        "-- MariaDB dump 10.19  Distrib 11.4.5-MariaDB\n\
         /*!40101 SET NAMES utf8mb4 */;\n\
         SELECT 1 ;\n"
    );
    assert_eq!(report.mariadb_comments, 4);
}

#[test]
fn collations_and_charsets_become_mysqls() {
    let out = ok("CREATE DATABASE /*!32312 IF NOT EXISTS*/ `shop` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_uca1400_ai_ci */;\n\
                  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;\n\
                  `a` varchar(9) COLLATE utf8mb4_uca1400_as_cs,\n\
                  `b` varchar(9) COLLATE uca1400_ai_ci,\n\
                  `c` varchar(9) COLLATE utf8mb4_nopad_bin,\n\
                  `d` varchar(9) DEFAULT _utf8mb3'x',\n");
    assert_eq!(
        out,
        "CREATE DATABASE /*!32312 IF NOT EXISTS*/ `shop` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci */;\n\
         ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;\n\
         `a` varchar(9) COLLATE utf8mb4_0900_as_cs,\n\
         `b` varchar(9) COLLATE utf8mb4_0900_ai_ci,\n\
         `c` varchar(9) COLLATE utf8mb4_bin,\n\
         `d` varchar(9) DEFAULT _utf8mb4'x',\n"
    );
    assert_eq!(
        mysql_name("utf8mb4_uca1400_ai_cs").as_deref(),
        Some("utf8mb4_0900_as_cs")
    );
    assert_eq!(
        mysql_name("utf8mb4_uca1400_as_ci").as_deref(),
        Some("utf8mb4_0900_as_ci")
    );
    assert_eq!(mysql_name("utf8mb4_general_ci"), None);
    assert_eq!(mysql_name("utf8mb4"), None);
}

#[test]
fn aria_options_go_and_the_table_is_innodb() {
    let out = ok(
        ") ENGINE=Aria DEFAULT CHARSET=utf8mb4 PAGE_CHECKSUM=1 ROW_FORMAT=PAGE TRANSACTIONAL=1;\n\
                  ) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=latin1 ROW_FORMAT=DYNAMIC;\n",
    );
    assert_eq!(
        out,
        ") ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\n\
         ) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=latin1 ROW_FORMAT=DYNAMIC;\n"
    );
    let (_, report) = run(") ENGINE=Aria PAGE_CHECKSUM=0 TRANSACTIONAL = 0;\n").unwrap();
    assert_eq!((report.engines, report.table_options), (1, 2));
}

/// The rows' data never changes: the same words inside strings, quoted
/// names and comments stay as they are, also over several lines in a
/// routine's body and with escaped quotes.
#[test]
fn data_and_names_stay_as_they_are() {
    let rows = "INSERT INTO `t` VALUES (1,'utf8mb3 PAGE_CHECKSUM=1 /*M!1 x */ it\\'s','ENGINE=Aria'),(2,\"uca1400_ai_ci\",'a''b /*M!');\n\
                DELIMITER ;;\n\
                CREATE PROCEDURE `p`() SELECT 'line one\n\
                CREATE SEQUENCE s; TRANSACTIONAL=1 \\\\' ;;\n\
                DELIMITER ;\n\
                -- utf8mb3 in a comment, CREATE SEQUENCE x\n\
                /* plain utf8mb3_general_ci */ SELECT 1;\n\
                INSERT INTO `t` VALUES ('caf\u{e9}');\n";
    let (out, report) = run(rows).unwrap();
    assert_eq!(out, rows);
    assert_eq!(report, Report::default());
    // Bytes that are not UTF-8 (a latin1 column) pass as they are.
    let mut out = Vec::new();
    let latin1 = b"INSERT INTO `t` VALUES ('caf\xe9');\r\n";
    filter_dump(&latin1[..], &mut out, None).unwrap();
    assert_eq!(out, latin1, "byte for byte, CRLF kept");
}

#[test]
fn sql_modes_keep_what_mysql_knows() {
    let out = ok("/*!50003 SET sql_mode = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION' */ ;\n\
                  /*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;\n\
                  /*!50003 SET sql_mode = 'SIMULTANEOUS_ASSIGNMENT,ORACLE' */ ;\n\
                  INSERT INTO `t` VALUES ('NO_AUTO_CREATE_USER');\n");
    assert_eq!(
        out,
        "/*!50003 SET sql_mode = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION' */ ;\n\
         /*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;\n\
         /*!50003 SET sql_mode = '' */ ;\n\
         INSERT INTO `t` VALUES ('NO_AUTO_CREATE_USER');\n"
    );
}

/// D-117: MySQL has no sequences; the filter reads the whole dump and names
/// each one with its database.
#[test]
fn sequences_stop_the_filter_with_their_names() {
    let dump = "USE `shop`;\n\
                CREATE SEQUENCE `orders_seq` start with 1 minvalue 1 maxvalue 9223372036854775806 increment by 1 cache 1000 nocycle ENGINE=InnoDB;\n\
                INSERT INTO `t` VALUES ('CREATE SEQUENCE `no`');\n\
                USE `blog`;\n\
                CREATE OR REPLACE SEQUENCE IF NOT EXISTS `ids`;\n\
                CREATE TABLE `sequence` (`id` int);\n";
    let err = run(dump).unwrap_err();
    let seq = err.downcast_ref::<Sequences>().expect("Sequences");
    assert_eq!(seq.0, ["shop.orders_seq", "blog.ids"]);
    let text = err.to_string();
    assert!(
        text.contains("CREATE SEQUENCE") && text.contains("shop.orders_seq"),
        "{text}"
    );
    // The dump of one database names it without `USE`.
    let mut out = Vec::new();
    let err = filter_dump(&b"CREATE SEQUENCE `s`;\n"[..], &mut out, Some("shop")).unwrap_err();
    assert_eq!(err.downcast_ref::<Sequences>().unwrap().0, ["shop.s"]);
}

/// A routine's body (DELIMITER ;;) with a string over two lines; the
/// DEFINER goes (the restoring account becomes it).
#[test]
fn a_trigger_of_a_dump() {
    let dump = "DELIMITER ;;\n\
                /*!50003 CREATE*/ /*!50017 DEFINER=`shop`@`localhost`*/ /*!50003 TRIGGER `t_bi` BEFORE INSERT ON `t` FOR EACH ROW SET NEW.note = 'two\n\
                lines' */;;\n\
                DELIMITER ;\n\
                /*!50003 SET sql_mode              = @saved_sql_mode */ ;\n";
    let (out, report) = run(dump).unwrap();
    assert_eq!(out, dump.replace(" DEFINER=`shop`@`localhost`", ""));
    assert_eq!(report.definers, 1);
    let view = ok(
        "/*!50013 DEFINER='root'@'localhost' SQL SECURITY DEFINER */\n\
                   CREATE DEFINER=CURRENT_USER() PROCEDURE `p`() SELECT 1;\n\
                   CREATE DEFINER = `a``b`@`%` EVENT `e` ON SCHEDULE EVERY 1 DAY DO SELECT 1;\n",
    );
    assert_eq!(
        view,
        "/*!50013 SQL SECURITY DEFINER */\n\
         CREATE PROCEDURE `p`() SELECT 1;\n\
         CREATE EVENT `e` ON SCHEDULE EVERY 1 DAY DO SELECT 1;\n"
    );
}

fn uncertain(dump: &str) -> Uncertain {
    let err = run(dump).expect_err(dump);
    err.downcast_ref::<Uncertain>()
        .unwrap_or_else(|| panic!("{err:#}"))
        .clone()
}

/// Security review of D-117: wherever the filter and MySQL could disagree on
/// where a string ends, the filter stops instead of writing what MySQL
/// would run as SQL.
#[test]
fn adversarial_dumps_stop_the_filter() {
    // NO_BACKSLASH_ESCAPES: `'a\'` is a whole string to MySQL, not here.
    let u = uncertain(
        "/*!40101 SET SQL_MODE='NO_BACKSLASH_ESCAPES' */;\n\
         INSERT INTO `t` VALUES ('a\\');DROP DATABASE x;-- ');\n",
    );
    assert_eq!(u.line, 1, "{u}");
    assert!(u.to_string().contains("NO_BACKSLASH_ESCAPES"));
    // Without it the same row is one string to both, data that passes.
    let row = "INSERT INTO `t` VALUES ('a\\');DROP DATABASE x;-- ');\n";
    assert_eq!(ok(row), row);
    // A string open at the end of a row's line (rows are one line each).
    let u = uncertain("INSERT INTO `t` VALUES ('a\\');\nDROP DATABASE x;\n");
    assert!(u.to_string().contains("open at the end of the line"), "{u}");
    // A MariaDB comment that does not close on its line.
    let u = uncertain("/*M!100616 SET NOTE_VERBOSITY=0\n*/ DROP DATABASE x;\n");
    assert!(u.to_string().contains("/*M!"), "{u}");
    // A comment or a name left open at the end of the dump.
    uncertain("SELECT 1; /* not closed\n");
    uncertain("DELIMITER ;;\nCREATE PROCEDURE `p`() SELECT 'open\n");
    // A DEFINER it cannot read.
    uncertain("CREATE DEFINER=`a\\`@`b` VIEW `v` AS SELECT 1;\n");
    // Well escaped, the same texts are data and pass untouched.
    let rows = "INSERT INTO `t` VALUES ('\\';DROP DATABASE x;--'),('/*M! x */'),('*/ DROP DATABASE y; /*M!');\n";
    assert_eq!(ok(rows), rows);
}

/// Security review of D-117: any bytes in the dump, never a panic (a
/// release build aborts on one, half way through a move).
#[test]
fn any_bytes_never_panic() {
    let mut dump: Vec<u8> = "ÀÀÀÀÀÀ DELIMITER ;;\n".as_bytes().to_vec();
    dump.extend_from_slice(b"\xff\xfe\xc3 DELIMITER ;;\n\xe2\x82 SELECT 1;\n");
    dump.extend_from_slice("ÀÀÀÀ\u{e9} 'é' /*M! x */;\n".as_bytes());
    let mut out = Vec::new();
    filter_dump(&dump[..], &mut out, None).unwrap();
    assert!(out.starts_with("ÀÀÀÀÀÀ DELIMITER ;;\n".as_bytes()));
    // Not a DELIMITER line: a string open at the end of a line stops.
    let mut out = Vec::new();
    let bad = b"\xc3\xc3\xc3\xc3\xc3 DELIMITER ;; 'x\nSELECT 1;\n";
    let err = filter_dump(&bad[..], &mut out, None).unwrap_err();
    assert!(err.downcast_ref::<Uncertain>().is_some(), "{err:#}");
    // A real one, after spaces, still counts.
    let fine = "  delimiter ;;\nCREATE PROCEDURE `p`() SELECT 'a\nb' ;;\nDELIMITER ;\n";
    let mut out = Vec::new();
    filter_dump(fine.as_bytes(), &mut out, None).unwrap();
    assert_eq!(out, fine.as_bytes());
}
