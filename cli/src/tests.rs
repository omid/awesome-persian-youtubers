use crate::{read_json_file, read_string_file, Channel};
use tokio::fs;

#[tokio::test]
pub async fn read_json_file_test() {
    let content = r#"[{ "id": "1", "name": "ali","category": "programming"}]"#;
    let file = "test1.json5";
    fs::write(file, content).await.unwrap();
    let res: Vec<Channel> = read_json_file(file).await.unwrap();
    assert_eq!(res.len(), 1);
    fs::remove_file(file).await.unwrap();
}

#[tokio::test]
pub async fn read_file_test() {
    let content = "just for test file";
    let file = "test2.json5";
    fs::write(file, content).await.unwrap();
    let res: String = read_string_file(file).await.unwrap();
    assert_eq!(res.len(), content.len());
    fs::remove_file(file).await.unwrap();
}
