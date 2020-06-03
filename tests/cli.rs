use assert_cmd::Command;
use predicates::prelude::*;

use hypothesis::annotations::Annotation;

fn create_annotation(text: &str) -> color_eyre::Result<String> {
    let mut cmd = Command::cargo_bin("hypothesis")?;
    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    let output = cmd
        .env("HYPOTHESIS_NAME", &dotenv::var("USERNAME")?)
        .env("HYPOTHESIS_KEY", &dotenv::var("DEVELOPER_KEY")?)
        .arg("annotations")
        .arg("create")
        .arg(&format!("--text={}", text))
        .arg(&format!("--group={}", group_id))
        .arg("www.example.com")
        .assert();
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    let stdout = stdout.split("annotation ").last();
    assert!(stdout.is_some());
    Ok(stdout.unwrap().trim().to_string())
}

#[test]
fn add_and_delete_annotation() -> color_eyre::Result<()> {
    dotenv::dotenv()?;
    let id = create_annotation("test text");
    assert!(id.is_ok());
    let id = id.unwrap();
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &dotenv::var("USERNAME")?)
        .env("HYPOTHESIS_KEY", &dotenv::var("DEVELOPER_KEY")?)
        .arg("annotations")
        .arg("delete")
        .arg(&id)
        .assert()
        .stdout(predicate::str::starts_with("Deleted").and(predicate::str::contains(&id)));
    Ok(())
}

#[test]
fn update_annotation() -> color_eyre::Result<()> {
    dotenv::dotenv()?;
    let username = dotenv::var("USERNAME")?;
    let key = dotenv::var("DEVELOPER_KEY")?;
    let id = create_annotation("test text")?;
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("update")
        .arg("--text=\"test text 2\"")
        .arg(&id)
        .assert()
        .stdout(predicate::str::starts_with("Updated").and(predicate::str::contains(&id)));
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("delete")
        .arg(&id)
        .assert()
        .success();
    Ok(())
}

#[test]
fn search_annotations() -> color_eyre::Result<()> {
    dotenv::dotenv()?;
    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or("__world__".into());
    let username = dotenv::var("USERNAME")?;
    let key = dotenv::var("DEVELOPER_KEY")?;
    let ids = (0..4)
        .map(|i| create_annotation(&format!("test text {}", i)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut cmd = Command::cargo_bin("hypothesis")?;
    let output = cmd
        .env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("search")
        .arg("--limit=200")
        .arg(&format!("--group={}", group_id))
        .assert();
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    let mut count = 0;
    for annotation in serde_json::Deserializer::from_str(&stdout).into_iter::<Annotation>() {
        if ids.contains(&annotation?.id) {
            count += 1;
        }
    }
    assert_eq!(count, 4);
    for id in ids {
        let mut cmd = Command::cargo_bin("hypothesis")?;
        cmd.env("HYPOTHESIS_NAME", &username)
            .env("HYPOTHESIS_KEY", &key)
            .arg("annotations")
            .arg("delete")
            .arg(id)
            .assert()
            .success();
    }
    Ok(())
}

#[test]
fn fetch_annotation() -> color_eyre::Result<()> {
    dotenv::dotenv()?;
    let username = dotenv::var("USERNAME")?;
    let key = dotenv::var("DEVELOPER_KEY")?;
    let id = create_annotation("test text")?;
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("fetch")
        .arg(&id)
        .assert()
        .success();
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("delete")
        .arg(id)
        .assert()
        .success();
    Ok(())
}
