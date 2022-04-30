#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::{thread, time};

use hypothesis::annotations::Annotation;

fn create_annotation(
    text: &str,
    username: &str,
    key: &str,
    group_id: &str,
) -> color_eyre::Result<String> {
    let mut cmd = Command::cargo_bin("hypothesis")?;
    let output = cmd
        .env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
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
    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or_else(|_| "__world__".into());
    let username = dotenv::var("HYPOTHESIS_NAME")?;
    let key = dotenv::var("HYPOTHESIS_KEY")?;

    // Create a new annotation
    let id = create_annotation("test annotation comment", &username, &key, &group_id);
    assert!(id.is_ok());
    let id = id.unwrap();

    // Fetch created annotation
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("fetch")
        .arg(&id)
        .assert()
        .stdout(
            predicate::str::contains(&id).and(predicate::str::contains("test annotation comment")),
        );

    // Delete annotation
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
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
    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or_else(|_| "__world__".into());
    let username = dotenv::var("HYPOTHESIS_NAME")?;
    let key = dotenv::var("HYPOTHESIS_KEY")?;

    // Create a new annotation
    let id = create_annotation("test annotation comment", &username, &key, &group_id);
    assert!(id.is_ok());
    let id = id.unwrap();

    // Update annotation
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("update")
        .arg("--text=\"test text 2\"")
        .arg(&id)
        .assert()
        .stdout(predicate::str::starts_with("Updated").and(predicate::str::contains(&id)));

    // Fetch updated annotation
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("annotations")
        .arg("fetch")
        .arg(&id)
        .assert()
        .stdout(predicate::str::contains(&id).and(predicate::str::contains("test text 2")));

    // Delete annotation
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
    let group_id = dotenv::var("TEST_GROUP_ID").unwrap_or_else(|_| "__world__".into());
    let username = dotenv::var("HYPOTHESIS_NAME")?;
    let key = dotenv::var("HYPOTHESIS_KEY")?;
    let ids = (0..4)
        .map(|i| create_annotation(&format!("test text {}", i), &username, &key, &group_id))
        .collect::<Result<Vec<_>, _>>()?;

    let duration = time::Duration::from_millis(500);
    thread::sleep(duration);
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

fn create_group(
    name: &str,
    description: &str,
    username: &str,
    key: &str,
) -> color_eyre::Result<String> {
    let mut cmd = Command::cargo_bin("hypothesis")?;
    let output = cmd
        .env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("groups")
        .arg("create")
        .arg(name)
        .arg(description)
        .assert();
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    let stdout = stdout.split("group ").last();
    assert!(stdout.is_some());
    Ok(stdout.unwrap().trim().to_string())
}

#[test]
fn create_and_leave_group() -> color_eyre::Result<()> {
    dotenv::dotenv()?;
    let username = dotenv::var("HYPOTHESIS_NAME")?;
    let key = dotenv::var("HYPOTHESIS_KEY")?;

    // Create a new group
    let group_id = create_group("test_name", "test description with spaces", &username, &key);
    assert!(group_id.is_ok());
    let group_id = group_id.unwrap();

    // Fetch created group
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("groups")
        .arg("fetch")
        .arg(&group_id)
        .assert()
        .stdout(predicate::str::contains(&group_id).and(predicate::str::contains("test_name")));

    // Leave group
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("groups")
        .arg("leave")
        .arg(&group_id)
        .assert()
        .stdout(predicate::str::starts_with(format!(
            "Left group {}",
            &group_id
        )));
    Ok(())
}

#[test]
fn update_group() -> color_eyre::Result<()> {
    let username = dotenv::var("HYPOTHESIS_NAME")?;
    let key = dotenv::var("HYPOTHESIS_KEY")?;

    // Create a new group
    let group_id = create_group("test_name", "test description with spaces", &username, &key);
    assert!(group_id.is_ok());
    let group_id = group_id.unwrap();

    // Update group
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("groups")
        .arg("update")
        .arg(&group_id)
        .arg("--name=test_group_2")
        .arg("--description=\"new description\"")
        .assert()
        .stdout(predicate::str::starts_with(format!(
            "Updated group {}",
            &group_id
        )));

    // Check that update worked
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("groups")
        .arg("fetch")
        .arg(&group_id)
        .assert()
        .stdout(predicate::str::contains(&group_id).and(predicate::str::contains("test_group_2")));

    // Leave group
    let mut cmd = Command::cargo_bin("hypothesis")?;
    cmd.env("HYPOTHESIS_NAME", &username)
        .env("HYPOTHESIS_KEY", &key)
        .arg("groups")
        .arg("leave")
        .arg(&group_id)
        .assert()
        .stdout(predicate::str::starts_with(format!(
            "Left group {}",
            group_id
        )));
    Ok(())
}
