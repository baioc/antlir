use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;

use anyhow::{Context, Result};
use itertools::Itertools;
use rayon::iter::*;
use structopt::{clap, StructOpt};
use postgres::{Client, NoTls};

// we declare all modules here so that they may refer to each other using `super::<mod>`
mod buck_test;
mod pyunit;
mod rust;

use buck_test::{Test, TestResult, test_name};

#[derive(StructOpt, Debug)]
#[structopt(
    about = "A custom buck test runner for Antlir's CI",
    setting = clap::AppSettings::AllowLeadingHyphen, // allows ignored options
)]
struct Options {
    /// Path to JSON-encoded test descriptions. Passed in by buck test
    #[structopt(long = "buck-test-info")]
    spec: PathBuf,

    /// Lists unit tests and exits without running them
    #[structopt(long, short)]
    list: bool,

    /// Path to generated test report in JUnit XML format
    #[structopt(long = "xml")]
    report: Option<PathBuf>,

    /// Connection string of the DB used in stateful test runs
    #[structopt(long = "state")]
    conn: Option<String>,

    /// Commit SHA-1 used to update the test DB on stateful runs
    #[structopt(long = "commit", requires("conn"))]
    revision: Option<String>,

    /// Forces auto-disabled tests to run
    #[structopt(long = "run-disabled")]
    run_disabled: bool,

    /// Maximum number of times a failing unit test will be retried
    #[structopt(long = "max-retries", short = "r", default_value = "0")]
    retries: u32,

    /// Maximum number of concurrent tests. Passed in by buck test
    #[structopt(long = "jobs", default_value = "1")]
    threads: usize,

    /// Warns on any further options for forward compatibility with buck
    #[structopt(hidden = true)]
    ignored: Vec<String>,
}

fn main() -> Result<()> {
    // parse command line
    let options = Options::from_args();
    if options.ignored.len() > 0 {
        eprintln!(
            "Warning: Unimplemented options were ignored: {:?}\n",
            options.ignored
        );
    }

    // validate and collect tests which are not auto-excluded
    let tests: Vec<Test> =
        buck_test::read(&options.spec)?
        .into_iter()
        .map(|spec| buck_test::validate(spec).unwrap())
        .flatten()
        .collect();

    // don't run anything when just listing
    if options.list {
        for test in tests {
           println!("{}", test_name(&test.target, &test.unit));
        }
        exit(0);
    }

    // connect to DB when it is provided
    let db = match options.conn {
        None => None,
        Some(ref uri) => Some(
            Client::connect(&uri, NoTls)
                .with_context(|| format!("Couldn't connect to specified test DB at '{}'", uri))?
        )
    };
    let disabled = query_disabled(db);

    // run tests in parallel (retries share the same thread)
    let total = tests.len();
    eprintln!("Found {} tests...", total);
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(options.threads)
        .build_global();
    let mut tests: Vec<TestResult> = tests
        .into_par_iter()
        .map(|test| {
            let run = !disabled.contains(&(test.target.clone(), test.unit.clone()));
            let test =  if run {
                test.run(options.retries)
            } else {
                TestResult {
                    target: test.target,
                    unit: test.unit,
                    attempts: 0,
                    passed: false,
                    duration: Duration::ZERO,
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                    contacts: test.contacts,
                }
            };

            let name = test_name(&test.target, &test.unit);
            if test.passed {
                print!("[PASS] {} ({} ms)", name, test.duration.as_millis());
                if test.attempts > 1 {
                    print!(" ({} attempts needed)\n", test.attempts);
                } else {
                    print!("\n");
                }
            } else if test.attempts == 0 {
                println!("[SKIP] {}", name);
            } else {
                println!("[FAIL] {} ({} ms)", name, test.duration.as_millis());
            }

            return test;
        })
        .collect();

    // collect and print results
    let mut passed = 0;
    let mut skipped = 0;
    let mut errors: Vec<String> = Vec::new();
    for test in tests.iter_mut() {
        if test.passed {
            passed += 1;
        } else if test.attempts == 0 {
            skipped += 1;
        } else {
            let mut message = format!(
                "\nTest {} failed after {} unsuccessful attempts:\n",
                test_name(&test.target, &test.unit), test.attempts
            );
            for line in test.stderr.split("\n") {
                let line = format!("    {}\n", line);
                message.push_str(&line);
            }
            if test.contacts.len() > 0 {
                let contacts = format!("Please report this to {:?}\n", test.contacts);
                message.push_str(&contacts);
            }
            errors.push(message);
        }
    }
    let failed = errors.len();
    for error in errors {
        eprintln!("{}", error);
    }
    println!("Out of {} tests, {} passed, {} failed, {} were skipped", total, passed, failed, skipped);

    // generate test report
    match options.report {
        None => (),
        Some(path) => report(tests, path)?,
    }

    exit(failed as i32);
}

// Refer to https://llg.cubic.org/docs/junit/
fn report<P: AsRef<Path>>(tests: Vec<TestResult>, path: P) -> Result<()> {
    let path = path.as_ref();
    let file = File::create(&path).with_context(|| {
        format!(
            "Couldn't generate report at specified path {}",
            path.display()
        )
    })?;
    let mut xml = BufWriter::new(&file);

    let failures = tests.iter().filter(|test| !test.passed && test.attempts > 0).count();
    writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(xml, r#"<testsuites tests="{}" failures="{}">"#, tests.len(), failures)?;

    // we group unit tests from the same buck target as a JUnit "testsuite"
    let suites = tests.into_iter().into_group_map_by(|test| test.target.clone());
    for (target, cases) in suites {
        let failures = cases.iter().filter(|test| !test.passed && test.attempts > 0).count();
        let skipped = cases.iter().filter(|test| !test.passed && test.attempts == 0).count();
        writeln!(xml, r#"  <testsuite name="{}" tests="{}" failures="{}" skipped="{}">"#,
                      target, cases.len(), failures, skipped)?;

        for test in cases {
            write!(xml, r#"    <testcase classname="{}" name="{}" time="{}""#,
                        &test.target, test.unit.as_ref().unwrap_or(&test.target), test.duration.as_millis() as f32 / 1e3)?;
            if test.passed {
                writeln!(xml, " />")?;
            } else {
                writeln!(xml, r#">"#)?;
                writeln!(xml, r#"      <failure>Test failed after {} unsuccessful attempts</failure>"#, test.attempts)?;
                writeln!(xml, r#"      <system-out>{}</system-out>"#,
                              xml_escape_text(test.stdout))?;
                writeln!(xml, r#"      <system-err>{}</system-err>"#,
                              xml_escape_text(test.stderr))?;
                writeln!(xml, r#"    </testcase>"#)?;
            }
        }

        writeln!(xml, r#"  </testsuite>"#)?;
    }

    writeln!(xml, r#"</testsuites>"#)?;

    eprintln!("Test report written to {}", path.display());
    return Ok(());
}

fn xml_escape_text(unescaped: String) -> String {
    return unescaped.replace("<", "&lt;").replace("&", "&amp;");
}

fn query_disabled(db: Option<Client>) -> HashSet<(String, Option<String>)> {
    match db {
        None => HashSet::new(),
        Some(mut db) =>
            db.query("SELECT target, test FROM tests WHERE disabled = true", &[])
            .unwrap()
            .into_iter()
            .map(|row| {
                let target = row.get("target");
                let test = row.get("test");
                let test = if test == "" { None } else { Some(test) };
                return (target, test);
            })
            .collect(),
    }
}
