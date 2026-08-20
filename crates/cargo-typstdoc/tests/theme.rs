use std::ffi::OsString;
use std::path::PathBuf;

use cargo_typstdoc::theme;

fn directory(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join("typstdoc-theme-tests").join(name);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn args(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn written(args: &[OsString]) -> String {
    let path = args
        .iter()
        .find_map(|arg| {
            let arg = arg.to_str()?;
            arg.strip_prefix("--extend-css=")
                .or_else(|| arg.ends_with(".css").then_some(arg))
        })
        .unwrap();
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn a_build_that_asked_for_no_stylesheet_gets_ours() {
    let directory = directory("none");
    let mut args = args(&["lib.rs"]);
    theme::extend(&mut args, &directory).unwrap();

    assert_eq!(args[1], OsString::from("--extend-css"));
    assert!(written(&args).contains("@font-face"));
}

#[test]
fn a_stylesheet_that_was_asked_for_is_kept() {
    let directory = directory("separate");
    let theirs = directory.join("theirs.css");
    std::fs::write(&theirs, "body { color: red; }").unwrap();

    let mut args = args(&["lib.rs", "--extend-css"]);
    args.push(theirs.clone().into_os_string());
    theme::extend(&mut args, &directory).unwrap();

    assert_ne!(args[2], theirs.into_os_string());
    let css = written(&args);
    assert!(css.starts_with("body { color: red; }\n"));
    assert!(css.contains("@font-face"));
}

#[test]
fn the_option_is_answered_in_the_form_it_was_given() {
    let directory = directory("joined");
    let theirs = directory.join("theirs.css");
    std::fs::write(&theirs, "body { color: red; }\n").unwrap();

    let mut args = args(&["lib.rs"]);
    args.push(format!("--extend-css={}", theirs.display()).into());
    theme::extend(&mut args, &directory).unwrap();

    assert_eq!(args.len(), 2);
    assert!(written(&args).contains("@font-face"));
}
