// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
use bee_orm::Model;

#[derive(Model)]
#[allow(dead_code)]
struct User {
    id: i32,
    name: String,
    age: i32,
}

#[test]
fn test_table_name() {
    assert_eq!(User::table_name(), "users");
}

#[test]
fn test_query_select_all() {
    let sql = User::query().to_sql();
    assert_eq!(sql, "SELECT * FROM users");
}

#[test]
fn test_query_with_filter() {
    let sql = User::query().filter("age > 18").to_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE age > 18");
}

#[test]
fn test_query_with_multiple_filters() {
    let sql = User::query().filter("age > 18").filter("name LIKE 'A%'").to_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE age > 18 AND name LIKE 'A%'");
}

#[test]
fn test_query_with_order_by() {
    let sql = User::query().order_by("id DESC").to_sql();
    assert_eq!(sql, "SELECT * FROM users ORDER BY id DESC");
}

#[test]
fn test_query_with_limit_offset() {
    let sql = User::query().limit(10).offset(20).to_sql();
    assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 20");
}

#[test]
fn test_query_combined() {
    let sql = User::query().filter("age > 18").order_by("id DESC").limit(10).offset(5).to_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE age > 18 ORDER BY id DESC LIMIT 10 OFFSET 5");
}
